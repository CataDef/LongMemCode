//! Generate ~500 LongMemCode scenarios from a `.argosbundle`.
//!
//! Strategy: for each category in the LongMemCode taxonomy, pick a
//! number of concrete instances from the bundle proportional to the
//! category's weight (Completion 32 %, BugFix 22 %, Refactor 12 %,
//! TestGen 10 %, FeatureAdd 10 %, ApiDiscovery 14 %).
//!
//! Ground-truth comes from the bundle's own structural facts — see
//! `docs/METHODOLOGY.md` for the full discussion. A grep-compared
//! partition is generated separately (follow-up) to catch decoder
//! regressions against an independent source.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use neurogenesis_bundle::{Bundle, EdgeKind, SymbolKindRepr};
use serde::Serialize;

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lmc-scenario-gen: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let bundle_path = flag(&args, "--bundle").context("--bundle <path> required")?;
    let out_path = flag(&args, "--out").context("--out <path> required")?;
    let target: usize = flag(&args, "--count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);

    let bundle = Bundle::open(PathBuf::from(&bundle_path))
        .with_context(|| format!("open bundle at {bundle_path}"))?;

    // Weights must match docs/TAXONOMY.md.
    let budget = [
        ("Completion", 0.32f64),
        ("BugFix", 0.22),
        ("Refactor", 0.12),
        ("TestGen", 0.10),
        ("FeatureAdd", 0.10),
        ("ApiDiscovery", 0.14),
    ];

    let mut scenarios = Vec::new();
    for (cat, w) in &budget {
        let n = (*w * target as f64).round() as usize;
        match *cat {
            "Completion" => scenarios.extend(gen_completion(&bundle, n)),
            "BugFix" => scenarios.extend(gen_bug_fix(&bundle, n)),
            "Refactor" => scenarios.extend(gen_refactor(&bundle, n)),
            "TestGen" => scenarios.extend(gen_test_gen(&bundle, n)),
            "FeatureAdd" => scenarios.extend(gen_feature_add(&bundle, n)),
            "ApiDiscovery" => scenarios.extend(gen_api_discovery(&bundle, n)),
            _ => unreachable!(),
        }
    }

    let json = serde_json::to_string_pretty(&scenarios)?;
    std::fs::write(&out_path, json)?;
    eprintln!(
        "generated {} scenarios → {}",
        scenarios.len(),
        out_path
    );
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

// ── Scenario structs (written out as JSON) ────────────────────────

#[derive(Debug, Serialize)]
struct Scenario {
    id: String,
    category: &'static str,
    intent: String,
    query: serde_json::Value,
    expected: serde_json::Value,
    gold_source: &'static str,
}

// ── Category generators ───────────────────────────────────────────

/// Completion: lookup-by-bare-name for real symbols in the bundle.
/// Mix functions + classes/structs + methods to exercise all the
/// shapes scip-python / rust-analyzer produce.
fn gen_completion(bundle: &Bundle, n: usize) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for (idx, sym) in bundle.root().symbols.iter().enumerate() {
        if picked >= n {
            break;
        }
        // Skip File nodes (not retrieval pivots for completion).
        if matches!(sym.kind, SymbolKindRepr::File) {
            continue;
        }
        let bare = bundle.string(sym.bare_name);
        if bare.is_empty() {
            continue;
        }
        let stable = bundle.string(sym.stable_id).to_string();
        out.push(Scenario {
            id: format!("completion-{picked:03}"),
            category: "Completion",
            intent: format!("agent about to use `{bare}` — needs the canonical id"),
            query: serde_json::json!({
                "op": "lookup",
                "name": bare,
                "bare_name": true,
            }),
            expected: serde_json::json!({
                "kind": "contains",
                "required": [stable],
            }),
            gold_source: "scip_roundtrip",
        });
        picked += 1;
        let _ = idx;
    }
    out
}

/// BugFix: callers + callees + implementors of symbols with
/// non-trivial edge counts. Each scenario represents an agent
/// triaging a bug: "who is affected by X?" / "what does X invoke
/// downstream?"
fn gen_bug_fix(bundle: &Bundle, n: usize) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut idx_with_callers: Vec<(u32, usize)> = Vec::new();
    let mut idx_with_callees: Vec<(u32, usize)> = Vec::new();
    for idx in 0..bundle.root().symbols.len() as u32 {
        let cin = bundle.incoming(idx).len();
        let cout = bundle.outgoing_of_kind(idx, EdgeKind::Calls).count();
        if cin > 0 {
            idx_with_callers.push((idx, cin));
        }
        if cout > 0 {
            idx_with_callees.push((idx, cout));
        }
    }
    idx_with_callers.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    idx_with_callees.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

    let callers_n = n / 2;
    let callees_n = n - callers_n;

    for (i, (sidx, _)) in idx_with_callers.iter().take(callers_n).enumerate() {
        let Some(sym) = bundle.symbol(*sidx) else {
            continue;
        };
        let stable = bundle.string(sym.stable_id).to_string();
        let callers: Vec<String> = bundle
            .incoming(*sidx)
            .iter()
            .filter_map(|i| bundle.symbol(*i).map(|s| bundle.string(s.stable_id).to_string()))
            .take(5)
            .collect();
        out.push(Scenario {
            id: format!("bug_fix-callers-{i:03}"),
            category: "BugFix",
            intent: format!("bug suspected in `{}` — enumerate callers", bundle.string(sym.bare_name)),
            query: serde_json::json!({
                "op": "callers",
                "sym_stable_id": stable,
            }),
            expected: serde_json::json!({
                "kind": "contains",
                "required": callers,
            }),
            gold_source: "scip_roundtrip",
        });
    }
    for (i, (sidx, _)) in idx_with_callees.iter().take(callees_n).enumerate() {
        let Some(sym) = bundle.symbol(*sidx) else {
            continue;
        };
        let stable = bundle.string(sym.stable_id).to_string();
        let callees: Vec<String> = bundle
            .outgoing_of_kind(*sidx, EdgeKind::Calls)
            .filter_map(|e| bundle.symbol(e.dst).map(|s| bundle.string(s.stable_id).to_string()))
            .take(5)
            .collect();
        out.push(Scenario {
            id: format!("bug_fix-callees-{i:03}"),
            category: "BugFix",
            intent: format!("bug downstream of `{}` — enumerate callees", bundle.string(sym.bare_name)),
            query: serde_json::json!({
                "op": "callees",
                "sym_stable_id": stable,
            }),
            expected: serde_json::json!({
                "kind": "contains",
                "required": callees,
            }),
            gold_source: "scip_roundtrip",
        });
    }
    out
}

/// Refactor: methods-of a class, enum-variants, implementors,
/// cross-module callers. Covers the "rename blast radius" + "full
/// surface" patterns.
fn gen_refactor(bundle: &Bundle, n: usize) -> Vec<Scenario> {
    let mut out = Vec::new();

    let mut classes: Vec<u32> = Vec::new();
    for idx in 0..bundle.root().symbols.len() as u32 {
        let Some(sym) = bundle.symbol(idx) else {
            continue;
        };
        let has_contains = bundle.outgoing_of_kind(idx, EdgeKind::Contains).next().is_some();
        if has_contains && matches!(sym.kind, SymbolKindRepr::Struct | SymbolKindRepr::Trait | SymbolKindRepr::Enum) {
            classes.push(idx);
        }
    }

    for (i, cidx) in classes.iter().take(n).enumerate() {
        let Some(sym) = bundle.symbol(*cidx) else {
            continue;
        };
        let stable = bundle.string(sym.stable_id).to_string();
        let methods: Vec<String> = bundle
            .outgoing_of_kind(*cidx, EdgeKind::Contains)
            .filter_map(|e| bundle.symbol(e.dst).map(|s| bundle.string(s.stable_id).to_string()))
            .take(8)
            .collect();
        out.push(Scenario {
            id: format!("refactor-methods-{i:03}"),
            category: "Refactor",
            intent: format!("agent renaming / auditing `{}` — full method surface", bundle.string(sym.bare_name)),
            query: serde_json::json!({
                "op": "contained_by",
                "sym_stable_id": stable,
            }),
            expected: serde_json::json!({
                "kind": "contains",
                "required": methods,
            }),
            gold_source: "scip_roundtrip",
        });
    }
    out
}

/// TestGen: file-symbols on test modules / conftest / fixtures.
fn gen_test_gen(bundle: &Bundle, n: usize) -> Vec<Scenario> {
    let mut out = Vec::new();
    let files: Vec<&str> = bundle.root().files.iter().map(|s| s.as_str()).collect();
    let mut picked = 0usize;
    for file in files {
        if picked >= n {
            break;
        }
        let lower = file.to_ascii_lowercase();
        let is_test = lower.contains("test") || lower.ends_with("conftest.py") || lower.contains("fixture");
        if !is_test {
            continue;
        }
        let Some(fidx) = bundle.file_idx(file) else {
            continue;
        };
        let syms: Vec<String> = bundle
            .symbols_in_file(fidx)
            .iter()
            .filter_map(|i| bundle.symbol(*i).map(|s| bundle.string(s.stable_id).to_string()))
            .filter(|id| !id.is_empty())
            .take(10)
            .collect();
        if syms.is_empty() {
            continue;
        }
        out.push(Scenario {
            id: format!("test_gen-file-{picked:03}"),
            category: "TestGen",
            intent: format!("agent writing a test — needs existing symbols in `{file}`"),
            query: serde_json::json!({
                "op": "file_symbols",
                "file_path": file,
            }),
            expected: serde_json::json!({
                "kind": "contains",
                "required": syms,
            }),
            gold_source: "scip_roundtrip",
        });
        picked += 1;
    }
    // If bundle has no test-like files (small mini-corpora), fall
    // back to sampling generic files so TestGen still has a floor.
    if out.is_empty() {
        for (i, f) in bundle.root().files.iter().take(n).enumerate() {
            let Some(fidx) = bundle.file_idx(f) else { continue };
            let syms: Vec<String> = bundle
                .symbols_in_file(fidx)
                .iter()
                .filter_map(|j| bundle.symbol(*j).map(|s| bundle.string(s.stable_id).to_string()))
                .take(6)
                .collect();
            if syms.is_empty() {
                continue;
            }
            out.push(Scenario {
                id: format!("test_gen-file-{i:03}"),
                category: "TestGen",
                intent: format!("file-symbols fallback on `{f}`"),
                query: serde_json::json!({
                    "op": "file_symbols",
                    "file_path": f,
                }),
                expected: serde_json::json!({
                    "kind": "contains",
                    "required": syms,
                }),
                gold_source: "scip_roundtrip",
            });
        }
    }
    out
}

/// FeatureAdd: class lookup + implementors of popular traits /
/// base classes.
fn gen_feature_add(bundle: &Bundle, n: usize) -> Vec<Scenario> {
    let mut out = Vec::new();
    // Find traits / abstract classes with many implementors.
    let mut impl_targets: Vec<(u32, usize)> = Vec::new();
    for idx in 0..bundle.root().symbols.len() as u32 {
        let k = bundle.implementors_of(idx).len();
        if k > 0 {
            impl_targets.push((idx, k));
        }
    }
    impl_targets.sort_by_key(|(_, k)| std::cmp::Reverse(*k));
    for (i, (tidx, _)) in impl_targets.iter().take(n).enumerate() {
        let Some(sym) = bundle.symbol(*tidx) else {
            continue;
        };
        let stable = bundle.string(sym.stable_id).to_string();
        let impls: Vec<String> = bundle
            .implementors_of(*tidx)
            .into_iter()
            .filter_map(|i| bundle.symbol(i).map(|s| bundle.string(s.stable_id).to_string()))
            .take(6)
            .collect();
        out.push(Scenario {
            id: format!("feature_add-impls-{i:03}"),
            category: "FeatureAdd",
            intent: format!("agent extending `{}` — needs existing implementors", bundle.string(sym.bare_name)),
            query: serde_json::json!({
                "op": "implementors",
                "sym_stable_id": stable,
            }),
            expected: serde_json::json!({
                "kind": "contains",
                "required": impls,
            }),
            gold_source: "scip_roundtrip",
        });
    }
    // Pad with class lookups if not enough trait targets.
    if out.len() < n {
        for idx in 0..bundle.root().symbols.len() as u32 {
            if out.len() >= n {
                break;
            }
            let Some(sym) = bundle.symbol(idx) else {
                continue;
            };
            if !matches!(sym.kind, SymbolKindRepr::Struct | SymbolKindRepr::Trait) {
                continue;
            }
            let bare = bundle.string(sym.bare_name);
            if bare.is_empty() {
                continue;
            }
            let stable = bundle.string(sym.stable_id).to_string();
            out.push(Scenario {
                id: format!("feature_add-lookup-{:03}", out.len()),
                category: "FeatureAdd",
                intent: format!("agent scaffolding against `{bare}` — confirm id"),
                query: serde_json::json!({
                    "op": "lookup",
                    "name": bare,
                    "bare_name": true,
                }),
                expected: serde_json::json!({
                    "kind": "contains",
                    "required": [stable],
                }),
                gold_source: "scip_roundtrip",
            });
        }
    }
    out
}

/// ApiDiscovery: positive exact matches + adversarial (fabricated
/// ids must return empty). This is the Precision Layer; grep can't
/// validate the exact stable_id structure, only scip can.
fn gen_api_discovery(bundle: &Bundle, n: usize) -> Vec<Scenario> {
    let mut out = Vec::new();
    let positive_n = n / 2;
    let adversarial_n = n - positive_n;

    for (i, sym) in bundle.root().symbols.iter().take(positive_n).enumerate() {
        if matches!(sym.kind, SymbolKindRepr::File) {
            continue;
        }
        let id = bundle.string(sym.stable_id).to_string();
        if id.is_empty() {
            continue;
        }
        out.push(Scenario {
            id: format!("api_discovery-positive-{i:03}"),
            category: "ApiDiscovery",
            intent: format!("precision-layer: exact symbol exists? ({})", bundle.string(sym.bare_name)),
            query: serde_json::json!({
                "op": "lookup",
                "name": id,
                "bare_name": false,
            }),
            expected: serde_json::json!({
                "kind": "exact_symbol",
                "stable_id": id,
            }),
            gold_source: "scip_roundtrip",
        });
    }

    // Adversarial: take real ids and mangle them into non-existent
    // siblings. A real SCIP stable_id can't be guessed by an LLM, so
    // an invented one must NOT resolve.
    let adversarials = [
        "this_symbol_does_not_exist",
        "QuantumTeleportManager",
        "overrideAllWithMars",
        "__teleport__",
        "fabricated_helper_that_never_existed",
    ];
    for i in 0..adversarial_n {
        let name = adversarials[i % adversarials.len()];
        out.push(Scenario {
            id: format!("api_discovery-adversarial-{i:03}"),
            category: "ApiDiscovery",
            intent: format!("precision-layer: fabricated name `{name}` must NOT resolve"),
            query: serde_json::json!({
                "op": "lookup",
                "name": name,
                "bare_name": true,
            }),
            expected: serde_json::json!({
                "kind": "exact_set",
                "stable_ids": [],
            }),
            gold_source: "adversarial",
        });
    }
    out
}
