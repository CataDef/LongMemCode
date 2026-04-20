//! Per-sub-type scenario generators. Each public function returns
//! a Gen function pointer list for one category. Generators operate
//! on the Bundle's public API only — never on private state — so
//! the scenarios they produce stay in sync with what adapters see.

use crate::{Gen, Scenario};
use neurogenesis_bundle::{Bundle, EdgeKind, SymbolKindRepr};
use serde_json::json;

// ── Shared helpers ────────────────────────────────────────────────

fn stable_id(bundle: &Bundle, idx: u32) -> Option<String> {
    bundle.symbol(idx).map(|s| bundle.string(s.stable_id).to_string())
}

fn bare_name<'a>(bundle: &'a Bundle, idx: u32) -> &'a str {
    bundle
        .symbol(idx)
        .map(|s| bundle.string(s.bare_name))
        .unwrap_or("")
}

fn classes(bundle: &Bundle) -> Vec<u32> {
    (0..bundle.root().symbols.len() as u32)
        .filter(|i| {
            bundle
                .symbol(*i)
                .map(|s| matches!(s.kind, SymbolKindRepr::Struct | SymbolKindRepr::Trait))
                .unwrap_or(false)
        })
        .collect()
}

fn enums(bundle: &Bundle) -> Vec<u32> {
    (0..bundle.root().symbols.len() as u32)
        .filter(|i| {
            bundle
                .symbol(*i)
                .map(|s| matches!(s.kind, SymbolKindRepr::Enum))
                .unwrap_or(false)
        })
        .collect()
}

fn functions(bundle: &Bundle) -> Vec<u32> {
    (0..bundle.root().symbols.len() as u32)
        .filter(|i| {
            bundle
                .symbol(*i)
                .map(|s| matches!(s.kind, SymbolKindRepr::Function))
                .unwrap_or(false)
        })
        .collect()
}

pub fn completion_subtypes() -> Vec<(&'static str, Gen)> {
    vec![
        ("lookup_class_by_name", gen_lookup_class_by_name),
        ("lookup_method_on_type", gen_lookup_method_on_type),
        ("signature_recall", gen_signature_recall),
        ("import_path_resolution", gen_import_path_resolution),
        ("cross_module_sibling", gen_cross_module_sibling),
        ("builder_chain_recall", gen_builder_chain_recall),
    ]
}

// Stubs — implemented in completion.rs via include; kept here as
// declarations so generators.rs stays the single entry point.
fn gen_lookup_class_by_name(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    for (i, idx) in classes(bundle).into_iter().take(n).enumerate() {
        let bare = bare_name(bundle, idx);
        if bare.is_empty() { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{i:03}"),
            category: cat.into(),
            sub_type: sub.into(),
            intent: format!("Agent importing class `{bare}` — needs the canonical id"),
            query: json!({"op":"lookup","name":bare,"bare_name":true,"kind":"struct"}),
            expected: json!({"kind":"contains","required":[id]}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_lookup_method_on_type(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for cls in classes(bundle) {
        if picked >= n { break; }
        for edge in bundle.outgoing_of_kind(cls, EdgeKind::Contains) {
            if picked >= n { break; }
            let Some(method_id) = stable_id(bundle, edge.dst) else { continue };
            let method_bare = bare_name(bundle, edge.dst);
            if method_bare.is_empty() { continue; }
            out.push(Scenario {
                id: format!("{sub}-{picked:03}"),
                category: cat.into(),
                sub_type: sub.into(),
                intent: format!("Agent calling method `{method_bare}` on its type"),
                query: json!({"op":"lookup","name":method_bare,"bare_name":true}),
                expected: json!({"kind":"contains","required":[method_id]}),
                gold_source: "scip_roundtrip".into(),
            });
            picked += 1;
        }
    }
    out
}

fn gen_signature_recall(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    for (i, idx) in functions(bundle).into_iter().take(n).enumerate() {
        let Some(id) = stable_id(bundle, idx) else { continue };
        let bare = bare_name(bundle, idx);
        out.push(Scenario {
            id: format!("{sub}-{i:03}"),
            category: cat.into(),
            sub_type: sub.into(),
            intent: format!("Agent verifying signature of `{bare}` before calling it"),
            query: json!({"op":"lookup","name":id,"bare_name":false}),
            expected: json!({"kind":"exact_symbol","stable_id":id}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_import_path_resolution(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let Some(sym) = bundle.symbol(idx) else { continue };
        if matches!(sym.kind, SymbolKindRepr::File) { continue; }
        let file = bundle.root().files.get(sym.file as usize).cloned().unwrap_or_default();
        if file.is_empty() { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"),
            category: cat.into(),
            sub_type: sub.into(),
            intent: format!("Agent needs the file path where `{}` is defined", bare_name(bundle, idx)),
            query: json!({"op":"lookup","name":id,"bare_name":false}),
            expected: json!({"kind":"exact_symbol","stable_id":id}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_cross_module_sibling(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    for (i, file) in bundle.root().files.iter().take(n).enumerate() {
        let Some(fidx) = bundle.file_idx(file) else { continue };
        let syms: Vec<String> = bundle
            .symbols_in_file(fidx)
            .iter()
            .filter_map(|j| stable_id(bundle, *j))
            .take(5)
            .collect();
        if syms.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{i:03}"),
            category: cat.into(),
            sub_type: sub.into(),
            intent: format!("Agent discovering siblings in `{file}`"),
            query: json!({"op":"file_symbols","file_path":file}),
            expected: json!({"kind":"contains","required":syms}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

pub fn bug_fix_subtypes() -> Vec<(&'static str, Gen)> {
    vec![
        ("callers_of_symbol", gen_callers_of_symbol),
        ("callees_of_symbol", gen_callees_of_symbol),
        ("override_detection", gen_override_detection),
        ("multi_hop_impact", gen_multi_hop_impact),
        ("exception_type_discovery", gen_exception_type_discovery),
        ("class_field_invariants", gen_class_field_invariants),
        ("error_wrapping_pattern", gen_error_wrapping_pattern),
    ]
}

fn gen_callers_of_symbol(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut v: Vec<(u32, usize)> = (0..bundle.root().symbols.len() as u32)
        .map(|i| (i, bundle.incoming(i).len()))
        .filter(|(_, c)| *c > 0)
        .collect();
    v.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    let mut out = Vec::new();
    for (i, (idx, _)) in v.iter().take(n).enumerate() {
        let Some(id) = stable_id(bundle, *idx) else { continue };
        let callers: Vec<String> = bundle.incoming(*idx).iter().filter_map(|j| stable_id(bundle, *j)).take(5).collect();
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Bug suspected in `{}` — enumerate callers", bare_name(bundle, *idx)),
            query: json!({"op":"callers","sym_stable_id":id}),
            expected: json!({"kind":"contains","required":callers}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_callees_of_symbol(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut v: Vec<(u32, usize)> = (0..bundle.root().symbols.len() as u32)
        .map(|i| (i, bundle.outgoing_of_kind(i, EdgeKind::Calls).count()))
        .filter(|(_, c)| *c > 0)
        .collect();
    v.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    let mut out = Vec::new();
    for (i, (idx, _)) in v.iter().take(n).enumerate() {
        let Some(id) = stable_id(bundle, *idx) else { continue };
        let callees: Vec<String> = bundle.outgoing_of_kind(*idx, EdgeKind::Calls)
            .filter_map(|e| stable_id(bundle, e.dst)).take(5).collect();
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Bug downstream of `{}` — enumerate callees", bare_name(bundle, *idx)),
            query: json!({"op":"callees","sym_stable_id":id}),
            expected: json!({"kind":"contains","required":callees}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_override_detection(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut v: Vec<(u32, usize)> = (0..bundle.root().symbols.len() as u32)
        .map(|i| (i, bundle.implementors_of(i).len()))
        .filter(|(_, c)| *c > 0)
        .collect();
    v.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    let mut out = Vec::new();
    for (i, (idx, _)) in v.iter().take(n).enumerate() {
        let Some(id) = stable_id(bundle, *idx) else { continue };
        let impls: Vec<String> = bundle.implementors_of(*idx).into_iter()
            .filter_map(|j| stable_id(bundle, j)).take(5).collect();
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Agent refactoring `{}` — find every override", bare_name(bundle, *idx)),
            query: json!({"op":"implementors","sym_stable_id":id}),
            expected: json!({"kind":"contains","required":impls}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_multi_hop_impact(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    // Depth-1 callers for now (depth-2 simulation would need runner
    // composition). Document as an approximation in notes.
    let mut v: Vec<(u32, usize)> = (0..bundle.root().symbols.len() as u32)
        .map(|i| (i, bundle.incoming(i).len()))
        .filter(|(_, c)| *c > 1)
        .collect();
    v.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    let mut out = Vec::new();
    for (i, (idx, _)) in v.iter().take(n).enumerate() {
        let Some(id) = stable_id(bundle, *idx) else { continue };
        let callers: Vec<String> = bundle.incoming(*idx).iter().filter_map(|j| stable_id(bundle, *j)).take(5).collect();
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Multi-hop impact starting from `{}` (depth-1 approximation)", bare_name(bundle, *idx)),
            query: json!({"op":"callers","sym_stable_id":id}),
            expected: json!({"kind":"contains","required":callers}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_exception_type_discovery(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    for (i, idx) in (0..bundle.root().symbols.len() as u32)
        .filter(|i| {
            let b = bare_name(bundle, *i);
            b.ends_with("Error") || b.ends_with("Exception")
        })
        .take(n)
        .enumerate()
    {
        let Some(id) = stable_id(bundle, idx) else { continue };
        let bare = bare_name(bundle, idx).to_string();
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Agent triaging error — locate `{bare}` type"),
            query: json!({"op":"lookup","name":bare,"bare_name":true}),
            expected: json!({"kind":"contains","required":[id]}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_class_field_invariants(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    for (i, cls) in classes(bundle).into_iter().take(n).enumerate() {
        let Some(id) = stable_id(bundle, cls) else { continue };
        let members: Vec<String> = bundle
            .outgoing_of_kind(cls, EdgeKind::Contains)
            .filter_map(|e| stable_id(bundle, e.dst))
            .take(5)
            .collect();
        if members.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Agent understanding invariants of `{}` — members", bare_name(bundle, cls)),
            query: json!({"op":"contained_by","sym_stable_id":id}),
            expected: json!({"kind":"contains","required":members}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_error_wrapping_pattern(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    // Approximation: signatures containing "Result" or "Option" —
    // read from the signature string pool.
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let Some(sym) = bundle.symbol(idx) else { continue };
        let sig = bundle.string(sym.signature);
        if !(sig.contains("Result<") || sig.contains("Option<") || sig.contains("-> Result") || sig.contains("-> Option")) {
            continue;
        }
        let Some(id) = stable_id(bundle, idx) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Example of error-wrapping pattern: `{}`", bare_name(bundle, idx)),
            query: json!({"op":"lookup","name":id,"bare_name":false}),
            expected: json!({"kind":"exact_symbol","stable_id":id}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_builder_chain_recall(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for cls in classes(bundle) {
        if picked >= n { break; }
        let builders: Vec<String> = bundle
            .outgoing_of_kind(cls, EdgeKind::Contains)
            .filter_map(|e| stable_id(bundle, e.dst))
            .filter(|s| s.contains("with_") || s.contains("builder"))
            .take(3)
            .collect();
        if builders.is_empty() { continue; }
        let Some(class_id) = stable_id(bundle, cls) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"),
            category: cat.into(),
            sub_type: sub.into(),
            intent: format!("Agent chaining builder calls on `{}`", bare_name(bundle, cls)),
            query: json!({"op":"contained_by","sym_stable_id":class_id}),
            expected: json!({"kind":"contains","required":builders}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

// ── Refactor (6) ──────────────────────────────────────────────────

pub fn refactor_subtypes() -> Vec<(&'static str, Gen)> {
    vec![
        ("methods_of_class", gen_methods_of_class),
        ("call_site_enumeration", gen_call_site_enumeration),
        ("enum_variant_list", gen_enum_variant_list),
        ("trait_implementors", gen_trait_implementors),
        ("cross_module_callers", gen_cross_module_callers),
        ("dead_export_detection", gen_dead_export_detection),
    ]
}

fn gen_methods_of_class(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    for (i, cls) in classes(bundle).into_iter().take(n).enumerate() {
        let Some(id) = stable_id(bundle, cls) else { continue };
        let methods: Vec<String> = bundle.outgoing_of_kind(cls, EdgeKind::Contains)
            .filter_map(|e| stable_id(bundle, e.dst)).take(6).collect();
        if methods.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Rename audit on `{}` — every method", bare_name(bundle, cls)),
            query: json!({"op":"contained_by","sym_stable_id":id}),
            expected: json!({"kind":"contains","required":methods}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_call_site_enumeration(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    gen_callers_of_symbol(bundle, n, cat, sub)
}

fn gen_enum_variant_list(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    for (i, e) in enums(bundle).into_iter().take(n).enumerate() {
        let Some(id) = stable_id(bundle, e) else { continue };
        let variants: Vec<String> = bundle.outgoing_of_kind(e, EdgeKind::Contains)
            .filter_map(|x| stable_id(bundle, x.dst)).take(8).collect();
        if variants.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Extending enum `{}` — list variants", bare_name(bundle, e)),
            query: json!({"op":"contained_by","sym_stable_id":id}),
            expected: json!({"kind":"contains","required":variants}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_trait_implementors(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    gen_override_detection(bundle, n, cat, sub)
}

fn gen_cross_module_callers(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let Some(sym) = bundle.symbol(idx) else { continue };
        let incoming = bundle.incoming(idx);
        if incoming.is_empty() { continue; }
        let target_file = sym.file;
        let cross: Vec<String> = incoming.iter()
            .filter_map(|j| {
                let caller = bundle.symbol(*j)?;
                if caller.file != target_file { stable_id(bundle, *j) } else { None }
            })
            .take(5)
            .collect();
        if cross.is_empty() { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Cross-module callers of `{}`", bare_name(bundle, idx)),
            query: json!({"op":"callers","sym_stable_id":id}),
            expected: json!({"kind":"contains","required":cross}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_dead_export_detection(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let orphans_ids: Vec<String> = bundle.orphans(Some(SymbolKindRepr::Function))
        .into_iter().filter_map(|i| stable_id(bundle, i)).collect();
    let mut out = Vec::new();
    for (i, id) in orphans_ids.into_iter().take(n).enumerate() {
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: "Agent cleaning dead exports — symbol has no callers".into(),
            query: json!({"op":"orphans","kind":"function"}),
            expected: json!({"kind":"contains","required":[id]}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

// ── TestGen (5) ───────────────────────────────────────────────────

pub fn test_gen_subtypes() -> Vec<(&'static str, Gen)> {
    vec![
        ("existing_tests_in_file", gen_existing_tests_in_file),
        ("fixture_discovery", gen_fixture_discovery),
        ("test_to_prod_mapping", gen_test_to_prod_mapping),
        ("mock_pattern", gen_mock_pattern),
        ("assertion_idiom", gen_assertion_idiom),
    ]
}

fn gen_existing_tests_in_file(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for file in &bundle.root().files {
        if picked >= n { break; }
        let low = file.to_ascii_lowercase();
        if !(low.contains("test") || low.contains("spec")) { continue; }
        let Some(fidx) = bundle.file_idx(file) else { continue };
        let syms: Vec<String> = bundle.symbols_in_file(fidx).iter()
            .filter_map(|i| stable_id(bundle, *i)).take(8).collect();
        if syms.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Agent writing a test — check existing in `{file}`"),
            query: json!({"op":"file_symbols","file_path":file}),
            expected: json!({"kind":"contains","required":syms}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_fixture_discovery(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for file in &bundle.root().files {
        if picked >= n { break; }
        let low = file.to_ascii_lowercase();
        if !(low.contains("conftest") || low.contains("fixture")) { continue; }
        let Some(fidx) = bundle.file_idx(file) else { continue };
        let syms: Vec<String> = bundle.symbols_in_file(fidx).iter()
            .filter_map(|i| stable_id(bundle, *i)).take(8).collect();
        if syms.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Find shared fixtures in `{file}`"),
            query: json!({"op":"file_symbols","file_path":file}),
            expected: json!({"kind":"contains","required":syms}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_test_to_prod_mapping(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let b = bare_name(bundle, idx);
        if !b.starts_with("test_") { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Map test `{b}` back to its production target"),
            query: json!({"op":"lookup","name":b,"bare_name":true}),
            expected: json!({"kind":"contains","required":[id]}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_mock_pattern(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let b = bare_name(bundle, idx);
        if !(b.to_ascii_lowercase().contains("mock") || b.to_ascii_lowercase().contains("stub")) { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Mock pattern example: `{b}`"),
            query: json!({"op":"lookup","name":b,"bare_name":true}),
            expected: json!({"kind":"contains","required":[id]}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_assertion_idiom(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let b = bare_name(bundle, idx);
        if !b.starts_with("assert") { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Assertion idiom: `{b}`"),
            query: json!({"op":"lookup","name":b,"bare_name":true}),
            expected: json!({"kind":"contains","required":[id]}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

// ── FeatureAdd (5) ────────────────────────────────────────────────

pub fn feature_add_subtypes() -> Vec<(&'static str, Gen)> {
    vec![
        ("nearest_feature_template", gen_nearest_feature_template),
        ("plugin_extension_point", gen_plugin_extension_point),
        ("di_wiring_registration", gen_di_wiring_registration),
        ("schema_partner_file", gen_schema_partner_file),
        ("public_api_surface", gen_public_api_surface),
    ]
}

fn gen_nearest_feature_template(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    // Approximation: treat a random sample of public-kind classes as
    // “similar feature” anchors; real nearest-feature needs embeddings.
    let mut out = Vec::new();
    for (i, cls) in classes(bundle).into_iter().take(n).enumerate() {
        let Some(id) = stable_id(bundle, cls) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Agent scaffolding mirrored on `{}`", bare_name(bundle, cls)),
            query: json!({"op":"lookup","name":id,"bare_name":false}),
            expected: json!({"kind":"exact_symbol","stable_id":id}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_plugin_extension_point(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    gen_override_detection(bundle, n, cat, sub)
}

fn gen_di_wiring_registration(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let b = bare_name(bundle, idx).to_ascii_lowercase();
        if !(b.starts_with("register") || b.contains("_register") || b == "register" || b == "add_route") { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        let callers: Vec<String> = bundle.incoming(idx).iter().filter_map(|j| stable_id(bundle, *j)).take(5).collect();
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Find wiring call-sites for `{}`", bare_name(bundle, idx)),
            query: json!({"op":"callers","sym_stable_id":id}),
            expected: json!({"kind":"contains","required":callers}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_schema_partner_file(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for file in &bundle.root().files {
        if picked >= n { break; }
        let low = file.to_ascii_lowercase();
        if !(low.contains("schema") || low.contains("model") || low.contains("migration")) { continue; }
        let Some(fidx) = bundle.file_idx(file) else { continue };
        let syms: Vec<String> = bundle.symbols_in_file(fidx).iter()
            .filter_map(|i| stable_id(bundle, *i)).take(5).collect();
        if syms.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Schema partner file: `{file}`"),
            query: json!({"op":"file_symbols","file_path":file}),
            expected: json!({"kind":"contains","required":syms}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_public_api_surface(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let b = bare_name(bundle, idx);
        if b.starts_with('_') || b.is_empty() { continue; }
        let Some(sym) = bundle.symbol(idx) else { continue };
        if !matches!(sym.kind, SymbolKindRepr::Function | SymbolKindRepr::Struct | SymbolKindRepr::Trait | SymbolKindRepr::Enum) { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Public API surface: `{b}`"),
            query: json!({"op":"lookup","name":id,"bare_name":false}),
            expected: json!({"kind":"exact_symbol","stable_id":id}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}


// ── ApiDiscovery + Ambiguity (12) ─────────────────────────────────

pub fn api_discovery_subtypes() -> Vec<(&'static str, Gen)> {
    vec![
        ("exact_symbol_exists", gen_exact_symbol_exists),
        ("exact_symbol_absent", gen_exact_symbol_absent),
        ("fake_type_absent", gen_fake_type_absent),
        ("typo_detection", gen_typo_detection),
        ("bare_name_collision", gen_bare_name_collision),
        ("signature_arity_check", gen_signature_arity_check),
        ("naming_convention_query", gen_naming_convention_query),
        ("disambiguate_by_type", gen_disambiguate_by_type),
        ("disambiguate_by_kind", gen_disambiguate_by_kind),
        ("disambiguate_by_module", gen_disambiguate_by_module),
        ("scope_restricted_lookup", gen_scope_restricted_lookup),
        ("fuzzy_match_ranking", gen_fuzzy_match_ranking),
    ]
}

fn gen_exact_symbol_exists(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let Some(sym) = bundle.symbol(idx) else { continue };
        if matches!(sym.kind, SymbolKindRepr::File) { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        if id.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: "Precision-layer: symbol exists, return the exact id".into(),
            query: json!({"op":"lookup","name":id,"bare_name":false}),
            expected: json!({"kind":"exact_symbol","stable_id":id}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_exact_symbol_absent(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let fakes = ["fake::DoesNotExist#fabricated()", "scip-python fake pkg 0.0.0 fake/Nothing#",
                 "does_not_exist_qualified_name", "::fabricated::path::", "Nothing at all"];
    let mut out = Vec::new();
    for i in 0..n {
        let name = fakes[i % fakes.len()];
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: "Precision-layer: fabricated full id must return empty".into(),
            query: json!({"op":"lookup","name":name,"bare_name":false}),
            expected: json!({"kind":"exact_set","stable_ids":[]}),
            gold_source: "adversarial".into(),
        });
    }
    let _ = bundle;
    out
}

fn gen_fake_type_absent(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let fakes = ["QuantumTeleportManager", "ZZZ_NotARealClass", "BugFixBot9000",
                 "__hallucinated__", "OverrideAllTheThings"];
    let mut out = Vec::new();
    for i in 0..n {
        let name = fakes[i % fakes.len()];
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Precision-layer: invented class name `{name}` must not resolve"),
            query: json!({"op":"lookup","name":name,"bare_name":true}),
            expected: json!({"kind":"exact_set","stable_ids":[]}),
            gold_source: "adversarial".into(),
        });
    }
    let _ = bundle;
    out
}

fn gen_typo_detection(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    // Typo → expected empty. Real typo tolerance (edit distance) is
    // a v0.2 enhancement; today adapters that don't do fuzzy match
    // honestly say "no" — which is the correct behaviour when the
    // LLM typed a name that doesn't exist.
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let b = bare_name(bundle, idx);
        if b.len() < 6 { continue; }
        // Swap two adjacent chars to form a typo.
        let mut chars: Vec<char> = b.chars().collect();
        chars.swap(2, 3);
        let typo: String = chars.into_iter().collect();
        if typo == b { continue; }
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Typo `{typo}` — should not resolve (adapter may return empty or fuzzy match)"),
            query: json!({"op":"lookup","name":typo,"bare_name":true}),
            expected: json!({"kind":"exact_set","stable_ids":[]}),
            gold_source: "adversarial".into(),
        });
        picked += 1;
    }
    out
}

fn gen_bare_name_collision(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    // Names with many candidates — verify the adapter returns them
    // all. "new", "open", "get", "init" are typical collision names.
    let candidates = ["new", "open", "get", "init", "read", "write", "close", "len"];
    let mut out = Vec::new();
    for (i, name) in candidates.iter().take(n).enumerate() {
        let hits: Vec<String> = bundle.lookup_bare_name(name).iter()
            .filter_map(|j| stable_id(bundle, *j)).collect();
        if hits.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Ambiguity: `{name}` has {} candidates — list them all", hits.len()),
            query: json!({"op":"lookup","name":name,"bare_name":true}),
            expected: json!({"kind":"contains","required":hits}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_signature_arity_check(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    gen_signature_recall(bundle, n, cat, sub)
}

fn gen_naming_convention_query(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    // Pick N public functions and ask the adapter to reflect them;
    // scoring is exact-id. A system returning them proves it knows
    // the convention the repo uses.
    gen_exact_symbol_exists(bundle, n, cat, sub)
}

fn gen_disambiguate_by_type(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    // "Get me `new` on BundleWriter specifically" — we ask for the
    // full id; adapter must return exactly that one.
    let mut out = Vec::new();
    let mut picked = 0usize;
    for cls in classes(bundle) {
        if picked >= n { break; }
        for edge in bundle.outgoing_of_kind(cls, EdgeKind::Contains) {
            if picked >= n { break; }
            let Some(mid) = stable_id(bundle, edge.dst) else { continue };
            let mbare = bare_name(bundle, edge.dst);
            if mbare.is_empty() { continue; }
            out.push(Scenario {
                id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
                intent: format!("Disambiguate `{mbare}` on enclosing type `{}`", bare_name(bundle, cls)),
                query: json!({"op":"lookup","name":mid,"bare_name":false}),
                expected: json!({"kind":"exact_symbol","stable_id":mid}),
                gold_source: "scip_roundtrip".into(),
            });
            picked += 1;
        }
    }
    out
}

fn gen_disambiguate_by_kind(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let common = ["new", "default", "open", "from"];
    let mut out = Vec::new();
    for (i, name) in common.iter().cycle().take(n).enumerate() {
        let hits: Vec<String> = bundle.lookup_bare_name(name).iter()
            .filter(|j| bundle.symbol(**j).map(|s| matches!(s.kind, SymbolKindRepr::Function)).unwrap_or(false))
            .filter_map(|j| stable_id(bundle, *j)).collect();
        if hits.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("`{name}` filtered to function kind"),
            query: json!({"op":"lookup","name":name,"bare_name":true,"kind":"function"}),
            expected: json!({"kind":"contains","required":hits}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_disambiguate_by_module(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    // "give me `open` in io/" — approximation: return full id, which
    // already encodes module path.
    gen_exact_symbol_exists(bundle, n, cat, sub)
}

fn gen_scope_restricted_lookup(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    gen_cross_module_sibling(bundle, n, cat, sub)
}

fn gen_fuzzy_match_ranking(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    gen_typo_detection(bundle, n, cat, sub)
}

// ── Control-flow & Type-shape (3) ─────────────────────────────────

pub fn control_flow_subtypes() -> Vec<(&'static str, Gen)> {
    vec![
        ("returns_given_shape", gen_returns_given_shape),
        ("exception_raisers", gen_exception_raisers),
        ("async_sync_shape", gen_async_sync_shape),
    ]
}

fn gen_returns_given_shape(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let Some(sym) = bundle.symbol(idx) else { continue };
        let sig = bundle.string(sym.signature);
        if !(sig.contains("-> Result") || sig.contains("-> Option")) { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Type-shape: `{}` returns a wrapped type", bare_name(bundle, idx)),
            query: json!({"op":"lookup","name":id,"bare_name":false}),
            expected: json!({"kind":"exact_symbol","stable_id":id}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_exception_raisers(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    gen_exception_type_discovery(bundle, n, cat, sub)
}

fn gen_async_sync_shape(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let Some(sym) = bundle.symbol(idx) else { continue };
        let sig = bundle.string(sym.signature);
        if !sig.starts_with("async") { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Async shape: `{}` is an async fn", bare_name(bundle, idx)),
            query: json!({"op":"lookup","name":id,"bare_name":false}),
            expected: json!({"kind":"exact_symbol","stable_id":id}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

// ── Config-surface (3) ────────────────────────────────────────────

pub fn config_subtypes() -> Vec<(&'static str, Gen)> {
    vec![
        ("env_var_call_sites", gen_env_var_call_sites),
        ("config_module_imports", gen_config_module_imports),
        ("feature_flag_enum_usage", gen_feature_flag_enum_usage),
    ]
}

fn gen_env_var_call_sites(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    // Target known env-var helpers. Many bundles won't have these
    // (small test corpora) — that's fine, generator yields less.
    let mut out = Vec::new();
    let candidates = ["var", "environ", "getenv"];
    for (i, name) in candidates.iter().cycle().take(n).enumerate() {
        let hits: Vec<String> = bundle.lookup_bare_name(name).iter()
            .filter_map(|j| stable_id(bundle, *j)).collect();
        if hits.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{i:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Env-var access point: `{name}`"),
            query: json!({"op":"lookup","name":name,"bare_name":true}),
            expected: json!({"kind":"contains","required":hits}),
            gold_source: "scip_roundtrip".into(),
        });
    }
    out
}

fn gen_config_module_imports(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for file in &bundle.root().files {
        if picked >= n { break; }
        let low = file.to_ascii_lowercase();
        if !(low.contains("config") || low.contains("settings")) { continue; }
        let Some(fidx) = bundle.file_idx(file) else { continue };
        let syms: Vec<String> = bundle.symbols_in_file(fidx).iter()
            .filter_map(|j| stable_id(bundle, *j)).take(5).collect();
        if syms.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Config module: `{file}`"),
            query: json!({"op":"file_symbols","file_path":file}),
            expected: json!({"kind":"contains","required":syms}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}

fn gen_feature_flag_enum_usage(bundle: &Bundle, n: usize, cat: &str, sub: &str) -> Vec<Scenario> {
    let mut out = Vec::new();
    let mut picked = 0usize;
    for idx in 0..bundle.root().symbols.len() as u32 {
        if picked >= n { break; }
        let b = bare_name(bundle, idx).to_ascii_lowercase();
        if !(b.contains("flag") || b.contains("feature")) { continue; }
        let Some(id) = stable_id(bundle, idx) else { continue };
        let callers: Vec<String> = bundle.incoming(idx).iter().filter_map(|j| stable_id(bundle, *j)).take(5).collect();
        if callers.is_empty() { continue; }
        out.push(Scenario {
            id: format!("{sub}-{picked:03}"), category: cat.into(), sub_type: sub.into(),
            intent: format!("Feature-flag usage: `{}`", bare_name(bundle, idx)),
            query: json!({"op":"callers","sym_stable_id":id}),
            expected: json!({"kind":"contains","required":callers}),
            gold_source: "scip_roundtrip".into(),
        });
        picked += 1;
    }
    out
}
