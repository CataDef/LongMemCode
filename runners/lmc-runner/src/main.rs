//! LongMemCode runner.
//!
//! Pipes scenarios to an adapter via the JSON-over-stdio protocol
//! (docs/ADAPTER_PROTOCOL.md), scores the responses, emits a report
//! matching results/schema.json.
//!
//! ```bash
//! lmc-runner \
//!     --adapter ./target/release/lmc-adapter-argosbrain \
//!     --adapter-args "--corpus /path/to/fastapi.argosbundle" \
//!     --scenarios scenarios/fastapi.json \
//!     --out results/argosbrain-fastapi-YYYY-MM-DD.json
//! ```

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::{ChildStdin, ChildStdout, Command, ExitCode, Stdio};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lmc-runner: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let adapter_bin = flag(&args, "--adapter").context("--adapter <path> required")?;
    let adapter_args = flag(&args, "--adapter-args").unwrap_or_default();
    let scenarios_path = flag(&args, "--scenarios").context("--scenarios <path> required")?;
    let out_path = flag(&args, "--out");
    // Audit trail: one JSON line per scenario, sibling to the summary.
    let audit_path = out_path.as_deref().map(|p| p.replace(".json", ".jsonl"));
    let mut audit_file: Option<BufWriter<File>> = audit_path
        .as_deref()
        .map(|p| File::create(p).map(BufWriter::new))
        .transpose()
        .context("create audit jsonl")?;
    let corpus_name = flag(&args, "--corpus-name").unwrap_or_else(|| "unknown".into());
    let corpus_commit = flag(&args, "--corpus-commit").unwrap_or_else(|| "unknown".into());
    let adapter_name = flag(&args, "--adapter-name").unwrap_or_else(|| "unknown".into());
    let adapter_version = flag(&args, "--adapter-version").unwrap_or_else(|| "0.0.0".into());

    let scenarios_bytes = std::fs::read(&scenarios_path)
        .with_context(|| format!("read scenarios at {scenarios_path}"))?;
    let scenarios: Vec<Scenario> = serde_json::from_slice(&scenarios_bytes)?;
    let scenarios_sha256 = sha256(&scenarios_bytes);

    // Spawn adapter.
    let extra: Vec<&str> = adapter_args.split_whitespace().collect();
    let mut child = Command::new(&adapter_bin)
        .args(&extra)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("spawn adapter {adapter_bin}"))?;

    let mut stdin = BufWriter::new(child.stdin.take().context("adapter stdin")?);
    let mut stdout = BufReader::new(child.stdout.take().context("adapter stdout")?);

    // Pipe scenarios + read responses; measure per-request wall time.
    let mut outcomes = Vec::with_capacity(scenarios.len());
    for s in &scenarios {
        let req = serde_json::json!({ "query": s.query });
        let t0 = Instant::now();
        send_req(&mut stdin, &req)?;
        let resp = recv_resp(&mut stdout)?;
        let elapsed = t0.elapsed().as_micros() as u64;
        let (score, precision, recall, notes) = score(&s.expected, &resp.results);
        // Audit line: full expected + returned so any reviewer can
        // diff. Written *after* scoring so `score` and `notes`
        // reflect the final judgement.
        if let Some(f) = audit_file.as_mut() {
            let line = serde_json::json!({
                "id": s.id,
                "category": s.category,
                "sub_type": s.sub_type,
                "gold_source": s.gold_source,
                "query": s.query,
                "expected": s.expected,
                "returned": resp.results,
                "score": score,
                "precision": precision,
                "recall": recall,
                "latency_us": elapsed,
                "cost_usd": resp.cost_usd,
                "notes": notes,
            });
            serde_json::to_writer(&mut *f, &line)?;
            writeln!(f)?;
        }
        outcomes.push(Outcome {
            scenario_id: s.id.clone(),
            category: s.category.clone(),
            gold_source: s.gold_source.clone(),
            score,
            precision,
            recall,
            latency_us: elapsed,
            returned_count: resp.results.len(),
            cost_usd: resp.cost_usd,
            tokens_estimate: resp
                .results
                .iter()
                .map(|id| approx_tokens(id.len()))
                .sum(),
            notes,
        });
    }

    // Close stdin so the adapter exits cleanly.
    drop(stdin);
    let _ = child.wait();
    if let Some(mut f) = audit_file.take() {
        f.flush().ok();
        if let Some(p) = audit_path.as_deref() {
            eprintln!("audit trail → {p}");
        }
    }

    // Aggregate.
    let report = aggregate(&outcomes, scenarios.len());
    let doc = serde_json::json!({
        "longmemcode_version": "0.1",
        "adapter": {
            "name": adapter_name,
            "version": adapter_version,
        },
        "corpus": {
            "name": corpus_name,
            "commit": corpus_commit,
            "scenarios_sha256": scenarios_sha256,
            "scenarios_count": scenarios.len(),
        },
        "machine": {
            "cpu": machine_cpu(),
            "ram_gb": 0,
            "os": machine_os(),
        },
        "summary": report.summary,
        "per_category": report.per_category,
        "per_gold_source": report.per_gold_source,
        "notes": [
            "scip_roundtrip scenarios: gold = bundle's own structural facts; tests decoder fidelity + speed.",
            "adversarial scenarios: fabricated names; gold = empty; tests anti-hallucination.",
        ],
    });
    let json = serde_json::to_string_pretty(&doc)?;
    if let Some(p) = out_path.as_deref() {
        std::fs::write(p, &json)?;
        eprintln!("report → {p}");
    }
    println!("{json}");
    Ok(())
}

// ── I/O plumbing ──────────────────────────────────────────────────

fn send_req(stdin: &mut BufWriter<ChildStdin>, req: &serde_json::Value) -> Result<()> {
    serde_json::to_writer(&mut *stdin, req)?;
    writeln!(stdin)?;
    stdin.flush()?;
    Ok(())
}

fn recv_resp(stdout: &mut BufReader<ChildStdout>) -> Result<AdapterResponse> {
    let mut line = String::new();
    let n = stdout.read_line(&mut line)?;
    if n == 0 {
        return Err(anyhow!("adapter closed stdout before response"));
    }
    Ok(serde_json::from_str(line.trim())?)
}

fn flag(args: &[String], f: &str) -> Option<String> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == f {
            return it.next().cloned();
        }
        if let Some(rest) = a.strip_prefix(&format!("{f}=")) {
            return Some(rest.to_string());
        }
    }
    None
}

// ── Scoring ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Scenario {
    id: String,
    category: String,
    #[serde(default)]
    sub_type: String,
    #[allow(dead_code)]
    intent: String,
    query: serde_json::Value,
    expected: serde_json::Value,
    gold_source: String,
}

#[derive(Debug, Deserialize)]
struct AdapterResponse {
    results: Vec<String>,
    #[serde(default)]
    cost_usd: f64,
}

#[derive(Debug, Serialize)]
struct Outcome {
    scenario_id: String,
    category: String,
    gold_source: String,
    score: f64,
    precision: Option<f64>,
    recall: Option<f64>,
    latency_us: u64,
    returned_count: usize,
    cost_usd: f64,
    tokens_estimate: u32,
    notes: String,
}

fn score(expected: &serde_json::Value, returned: &[String]) -> (f64, Option<f64>, Option<f64>, String) {
    let kind = expected.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    match kind {
        "exact_symbol" => {
            let want = expected
                .get("stable_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            match returned.first() {
                Some(top) if top == want => (1.0, None, None, String::new()),
                Some(top) => (0.0, None, None, format!("top-1={top} wanted={want}")),
                None => (0.0, None, None, "empty".into()),
            }
        }
        "in_top_k" => {
            let k = expected
                .get("k")
                .and_then(|v| v.as_u64())
                .unwrap_or(5) as usize;
            let want = expected
                .get("stable_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let hit = returned.iter().take(k).any(|r| r == want);
            (
                if hit { 1.0 } else { 0.0 },
                None,
                None,
                if hit { String::new() } else { format!("missing {want}") },
            )
        }
        "exact_set" => {
            let want: std::collections::HashSet<&str> = expected
                .get("stable_ids")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str()).collect())
                .unwrap_or_default();
            let got: std::collections::HashSet<&str> = returned.iter().map(String::as_str).collect();
            if want.is_empty() && got.is_empty() {
                return (1.0, Some(1.0), Some(1.0), String::new());
            }
            let tp = got.intersection(&want).count() as f64;
            let p = if got.is_empty() { 0.0 } else { tp / got.len() as f64 };
            let r = if want.is_empty() { 0.0 } else { tp / want.len() as f64 };
            let f1 = if p + r > 0.0 { 2.0 * p * r / (p + r) } else { 0.0 };
            (f1, Some(p), Some(r), String::new())
        }
        "contains" => {
            let req: Vec<&str> = expected
                .get("required")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str()).collect())
                .unwrap_or_default();
            let got: std::collections::HashSet<&str> = returned.iter().map(String::as_str).collect();
            if req.is_empty() {
                return (1.0, None, None, String::new());
            }
            let hit = req.iter().filter(|x| got.contains(**x)).count();
            let score = hit as f64 / req.len() as f64;
            (
                score,
                None,
                None,
                if score > 0.999 {
                    String::new()
                } else {
                    format!("hit {}/{}", hit, req.len())
                },
            )
        }
        other => (0.0, None, None, format!("unknown expected.kind={other}")),
    }
}

// ── Aggregation ───────────────────────────────────────────────────

struct Report {
    summary: serde_json::Value,
    per_category: serde_json::Value,
    per_gold_source: serde_json::Value,
}

fn aggregate(outcomes: &[Outcome], total: usize) -> Report {
    let weights: std::collections::HashMap<&str, f64> = [
        ("Completion", 0.32),
        ("BugFix", 0.22),
        ("Refactor", 0.12),
        ("TestGen", 0.10),
        ("FeatureAdd", 0.10),
        ("ApiDiscovery", 0.14),
    ]
    .iter()
    .cloned()
    .collect();

    let mut per_cat: BTreeMap<String, (u32, u32, f64, f64, u32, f64, u32)> = BTreeMap::new();
    let mut per_gold: BTreeMap<String, (u32, u32, f64)> = BTreeMap::new();

    for o in outcomes {
        let e = per_cat.entry(o.category.clone()).or_default();
        e.0 += 1;
        if o.score > 0.999 {
            e.1 += 1;
        }
        e.2 += o.score;
        if let Some(p) = o.precision {
            e.3 += p;
            e.4 += 1;
        }
        if let Some(r) = o.recall {
            e.5 += r;
            e.6 += 1;
        }

        let g = per_gold.entry(o.gold_source.clone()).or_default();
        g.0 += 1;
        if o.score > 0.999 {
            g.1 += 1;
        }
        g.2 += o.score;
    }

    let mut cats_json = serde_json::Map::new();
    let mut weighted_sum = 0.0f64;
    let mut weight_norm = 0.0f64;
    for (cat, (n, passed, ss, sp, pn, sr, rn)) in &per_cat {
        let avg = if *n == 0 { 0.0 } else { ss / *n as f64 };
        if let Some(w) = weights.get(cat.as_str()) {
            weighted_sum += avg * w;
            weight_norm += w;
        }
        cats_json.insert(
            cat.clone(),
            serde_json::json!({
                "n": n,
                "passed": passed,
                "avg_score": avg,
                "avg_precision": if *pn == 0 { serde_json::Value::Null } else { serde_json::json!(sp / *pn as f64) },
                "avg_recall": if *rn == 0 { serde_json::Value::Null } else { serde_json::json!(sr / *rn as f64) },
            }),
        );
    }
    let weighted = if weight_norm > 0.0 { weighted_sum / weight_norm } else { 0.0 };

    let raw = if outcomes.is_empty() {
        0.0
    } else {
        outcomes.iter().map(|o| o.score).sum::<f64>() / outcomes.len() as f64
    };

    let mut lats: Vec<u64> = outcomes.iter().map(|o| o.latency_us).collect();
    lats.sort_unstable();
    let p50 = percentile(&lats, 0.50);
    let p95 = percentile(&lats, 0.95);
    let p99 = percentile(&lats, 0.99);

    let total_tokens: u64 = outcomes.iter().map(|o| o.tokens_estimate as u64).sum();
    let total_cost: f64 = outcomes.iter().map(|o| o.cost_usd).sum();
    let cost_per_1k = if total > 0 {
        total_cost * 1000.0 / total as f64
    } else {
        0.0
    };

    let mut gold_json = serde_json::Map::new();
    for (g, (n, passed, ss)) in &per_gold {
        let avg = if *n == 0 { 0.0 } else { ss / *n as f64 };
        gold_json.insert(
            g.clone(),
            serde_json::json!({ "n": n, "passed": passed, "avg_score": avg }),
        );
    }

    Report {
        summary: serde_json::json!({
            "weighted_accuracy": weighted,
            "raw_accuracy": raw,
            "p50_latency_ms": (p50 as f64) / 1000.0,
            "p95_latency_ms": (p95 as f64) / 1000.0,
            "p99_latency_ms": (p99 as f64) / 1000.0,
            "total_tokens_returned": total_tokens,
            "total_cost_usd": total_cost,
            "cost_per_1k_queries_usd": cost_per_1k,
        }),
        per_category: serde_json::Value::Object(cats_json),
        per_gold_source: serde_json::Value::Object(gold_json),
    }
}

fn percentile(sorted: &[u64], q: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64) * q).ceil() as usize;
    sorted[idx.saturating_sub(1).min(sorted.len() - 1)]
}

fn approx_tokens(chars: usize) -> u32 {
    ((chars as f32 / 4.0).ceil() as u32).max(1)
}

fn sha256(bytes: &[u8]) -> String {
    // Stdlib has no sha256; we use a FNV-1a 64-bit fallback and
    // label it as such in the field comment. A dedicated sha2 dep
    // would improve cross-run auditability but costs the build time
    // of another crate — v0.2 swap.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("fnv64:{h:016x}")
}

fn machine_cpu() -> String {
    std::env::var("LMC_CPU").unwrap_or_else(|_| "unknown".into())
}

fn machine_os() -> String {
    std::env::var("LMC_OS").unwrap_or_else(|_| std::env::consts::OS.to_string())
}

// Silence unused warning on the import (only .first() needed).
#[allow(dead_code)]
fn _touch(_: &PathBuf) {}
