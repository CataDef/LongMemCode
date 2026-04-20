//! ArgosBrain adapter for LongMemCode.
//!
//! Speaks the JSON-over-stdio protocol described in
//! `docs/ADAPTER_PROTOCOL.md`: reads `{ "query": {...} }` lines on
//! stdin and emits `{ "results": [...], "cost_usd": 0.0 }` lines on
//! stdout in the same order. Wraps `neurogenesis_bundle::Bundle` —
//! no LLM hops, `cost_usd` is always 0.
//!
//! Usage:
//!
//! ```bash
//! lmc-adapter-argosbrain --corpus corpora/_work/fastapi/fastapi.argosbundle \
//!     < scenarios.jsonl > results.jsonl
//! ```

use std::io::{BufRead, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{anyhow, Context, Result};
use neurogenesis_bundle::{Bundle, EdgeKind, SymbolKindRepr};
use serde::{Deserialize, Serialize};

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lmc-adapter-argosbrain: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let corpus = flag(&args, "--corpus").context("--corpus <path> required")?;
    let bundle = Bundle::open(PathBuf::from(&corpus))
        .with_context(|| format!("open bundle at {corpus}"))?;

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for line in stdin.lock().lines() {
        let line = line.context("read stdin line")?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                emit_error(&mut out, &format!("parse request: {e}"))?;
                continue;
            }
        };
        let results = execute(&bundle, &req.query);
        let resp = Response {
            results,
            cost_usd: 0.0,
        };
        serde_json::to_writer(&mut out, &resp)?;
        writeln!(out)?;
        out.flush()?;
    }
    Ok(())
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

fn emit_error(out: &mut impl Write, msg: &str) -> Result<()> {
    let resp = Response {
        results: Vec::new(),
        cost_usd: 0.0,
    };
    serde_json::to_writer(
        &mut *out,
        &serde_json::json!({
            "results": resp.results,
            "cost_usd": resp.cost_usd,
            "error": msg,
        }),
    )?;
    writeln!(out)?;
    out.flush().map_err(|e| anyhow!(e))?;
    Ok(())
}

// ── Protocol types ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Request {
    query: Query,
}

#[derive(Debug, Serialize)]
struct Response {
    results: Vec<String>,
    cost_usd: f64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Query {
    Lookup {
        name: String,
        #[serde(default)]
        bare_name: bool,
        #[serde(default)]
        kind: Option<String>,
    },
    Callers {
        sym_stable_id: String,
    },
    Callees {
        sym_stable_id: String,
    },
    ContainedBy {
        sym_stable_id: String,
    },
    FileSymbols {
        file_path: String,
    },
    Implementors {
        sym_stable_id: String,
    },
    Orphans {
        #[serde(default)]
        kind: Option<String>,
    },
}

fn execute(bundle: &Bundle, q: &Query) -> Vec<String> {
    match q {
        Query::Lookup {
            name,
            bare_name,
            kind,
        } => {
            let cands: Vec<u32> = if *bare_name {
                bundle.lookup_bare_name(name).to_vec()
            } else if let Some(idx) = bundle.lookup_stable_id(name) {
                vec![idx]
            } else {
                Vec::new()
            };
            cands
                .into_iter()
                .filter(|idx| {
                    let Some(sym) = bundle.symbol(*idx) else {
                        return false;
                    };
                    let Some(want) = kind.as_deref() else {
                        return true;
                    };
                    format!("{:?}", sym.kind).eq_ignore_ascii_case(want)
                })
                .filter_map(|idx| {
                    bundle.symbol(idx).map(|s| bundle.string(s.stable_id).to_string())
                })
                .collect()
        }
        Query::Callers { sym_stable_id } => bundle
            .lookup_stable_id(sym_stable_id)
            .map(|idx| {
                bundle
                    .incoming(idx)
                    .iter()
                    .filter_map(|i| bundle.symbol(*i).map(|s| bundle.string(s.stable_id).to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        Query::Callees { sym_stable_id } => bundle
            .lookup_stable_id(sym_stable_id)
            .map(|idx| {
                bundle
                    .outgoing_of_kind(idx, EdgeKind::Calls)
                    .filter_map(|e| bundle.symbol(e.dst).map(|s| bundle.string(s.stable_id).to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        Query::ContainedBy { sym_stable_id } => bundle
            .lookup_stable_id(sym_stable_id)
            .map(|idx| {
                bundle
                    .outgoing_of_kind(idx, EdgeKind::Contains)
                    .filter_map(|e| bundle.symbol(e.dst).map(|s| bundle.string(s.stable_id).to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        Query::FileSymbols { file_path } => bundle
            .file_idx(file_path)
            .map(|f| {
                bundle
                    .symbols_in_file(f)
                    .iter()
                    .filter_map(|i| bundle.symbol(*i).map(|s| bundle.string(s.stable_id).to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        Query::Implementors { sym_stable_id } => bundle
            .lookup_stable_id(sym_stable_id)
            .map(|target| {
                bundle
                    .implementors_of(target)
                    .into_iter()
                    .filter_map(|idx| bundle.symbol(idx).map(|s| bundle.string(s.stable_id).to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        Query::Orphans { kind } => {
            let kf = kind.as_deref().and_then(|k| match k.to_ascii_lowercase().as_str() {
                "function" => Some(SymbolKindRepr::Function),
                "struct" => Some(SymbolKindRepr::Struct),
                "trait" => Some(SymbolKindRepr::Trait),
                "enum" => Some(SymbolKindRepr::Enum),
                "module" => Some(SymbolKindRepr::Module),
                _ => None,
            });
            bundle
                .orphans(kf)
                .into_iter()
                .filter_map(|idx| bundle.symbol(idx).map(|s| bundle.string(s.stable_id).to_string()))
                .collect()
        }
    }
}
