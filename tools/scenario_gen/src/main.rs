//! Generate LongMemCode scenarios from a `.argosbundle`, one
//! generator per sub-type (35 total), weighted to the TAXONOMY.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use neurogenesis_bundle::Bundle;
use serde::Serialize;

mod generators;
use generators::*;

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
        .unwrap_or(700);

    let bundle = Bundle::open(PathBuf::from(&bundle_path))
        .with_context(|| format!("open bundle at {bundle_path}"))?;

    // Category weights match docs/TAXONOMY.md. Each sub-type inside
    // a category gets an equal share of the category's budget.
    let categories: Vec<(&str, f64, Vec<(&str, Gen)>)> = vec![
        ("Completion", 0.28, completion_subtypes()),
        ("BugFix", 0.18, bug_fix_subtypes()),
        ("Refactor", 0.10, refactor_subtypes()),
        ("TestGen", 0.08, test_gen_subtypes()),
        ("FeatureAdd", 0.08, feature_add_subtypes()),
        ("ApiDiscovery", 0.15, api_discovery_subtypes()),
        ("ControlFlow", 0.05, control_flow_subtypes()),
        ("Config", 0.04, config_subtypes()),
    ];

    let mut all = Vec::new();
    for (cat, weight, subs) in &categories {
        let cat_budget = (*weight * target as f64).round() as usize;
        let per_sub = cat_budget / subs.len().max(1);
        for (sub_name, gen) in subs {
            let scenarios = gen(&bundle, per_sub, cat, sub_name);
            all.extend(scenarios);
        }
    }

    let json = serde_json::to_string_pretty(&all)?;
    std::fs::write(&out_path, json)?;
    eprintln!("generated {} scenarios → {}", all.len(), out_path);
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

#[derive(Debug, Serialize)]
pub struct Scenario {
    pub id: String,
    pub category: String,
    pub sub_type: String,
    pub intent: String,
    pub query: serde_json::Value,
    pub expected: serde_json::Value,
    pub gold_source: String,
}

pub type Gen = fn(&Bundle, usize, &str, &str) -> Vec<Scenario>;
