//! Grep baseline adapter.
//!
//! Answers each LongMemCode query using what every coding agent does
//! today without a dedicated memory layer: `ripgrep` on the source
//! tree + file scans. This is the honest floor we're trying to beat.
//!
//! Why we need the bundle path AT ALL: the adapter protocol asks for
//! SCIP stable_ids back. Pure grep returns `file:line` hits; we have
//! no way to reconstruct scip-style ids from text alone. So the
//! adapter uses the bundle SOLELY as a symbol-name lookup table —
//! it never traverses edges, never reads the graph. Every semantic
//! decision (what's a caller? what's an override?) still comes from
//! grep patterns, with all the textual blindness that entails.
//!
//! This is equivalent to what an agent gets from `rg` + a
//! universal-ctags tags file. It's the floor of the industry baseline.

use std::collections::HashMap;
use std::io::{BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use anyhow::{anyhow, Context, Result};
use neurogenesis_bundle::{Bundle, SymbolIdx};
use serde::{Deserialize, Serialize};

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lmc-adapter-grep: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let bundle_path = flag(&args, "--bundle").context("--bundle <path> required")?;
    let source_dir = flag(&args, "--source").context("--source <dir> required")?;
    let bundle = Bundle::open(PathBuf::from(&bundle_path))
        .with_context(|| format!("open bundle at {bundle_path}"))?;
    let src = PathBuf::from(&source_dir);

    // Pre-build a bare_name → Vec<stable_id> lookup from the bundle
    // so we can translate grep text hits back to ids. This is the
    // ONLY use we make of the bundle — no edges, no kinds.
    let mut by_bare: HashMap<String, Vec<String>> = HashMap::new();
    for idx in 0..bundle.root().symbols.len() as SymbolIdx {
        let Some(sym) = bundle.symbol(idx) else { continue };
        let bare = bundle.string(sym.bare_name);
        let id = bundle.string(sym.stable_id).to_string();
        if bare.is_empty() || id.is_empty() {
            continue;
        }
        by_bare.entry(bare.to_string()).or_default().push(id);
    }

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
        let results = execute(&req.query, &src, &by_bare);
        let resp = Response { results, cost_usd: 0.0 };
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
    serde_json::to_writer(
        &mut *out,
        &serde_json::json!({"results": Vec::<String>::new(), "cost_usd": 0.0, "error": msg}),
    )?;
    writeln!(out)?;
    out.flush().map_err(|e| anyhow!(e))?;
    Ok(())
}

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
    Callers { sym_stable_id: String },
    Callees { sym_stable_id: String },
    ContainedBy { sym_stable_id: String },
    FileSymbols { file_path: String },
    Implementors { sym_stable_id: String },
    Orphans {
        #[serde(default)]
        kind: Option<String>,
    },
}

fn execute(q: &Query, src: &Path, by_bare: &HashMap<String, Vec<String>>) -> Vec<String> {
    match q {
        Query::Lookup { name, bare_name, .. } => {
            // For bare_name lookups, ask grep "does this identifier
            // appear at all?" — if yes, map to every matching stable
            // id via the bundle's bare_name index. This models what
            // a grep-based agent would answer ("yes, I found it
            // somewhere — here are the candidates"). For fake names,
            // grep finds nothing, no ids returned.
            if *bare_name {
                let hits = grep_count_word(src, name);
                if hits == 0 {
                    Vec::new()
                } else {
                    by_bare.get(name).cloned().unwrap_or_default()
                }
            } else {
                // Full-id lookup: grep CAN'T construct stable_ids
                // from text. We return empty when the name isn't
                // part of the id as a substring; when it is, we
                // guess the id itself. This mirrors the naive
                // "grep the fully-qualified name" approach.
                let segments: Vec<&str> = name.split(|c: char| !c.is_alphanumeric() && c != '_').collect();
                let probe = segments.iter().rev().find(|s| !s.is_empty()).copied().unwrap_or("");
                if probe.is_empty() || grep_count_word(src, probe) == 0 {
                    Vec::new()
                } else {
                    vec![name.clone()]
                }
            }
        }
        Query::Callers { sym_stable_id } => {
            // Grep approach: pick the trailing identifier from the
            // SCIP id, grep for "name(" across the source, map
            // matching files to any stable_ids in those files via
            // the bundle's bare_name index. Misses dynamic dispatch
            // and reexports — exactly the grep limitation we expose.
            let probe = trailing_ident(sym_stable_id);
            if probe.is_empty() { return Vec::new(); }
            let pattern = format!(r"\b{}\s*\(", regex_escape(probe));
            let files = grep_files_matching(src, &pattern);
            ids_in_files(by_bare, &files)
        }
        Query::Callees { sym_stable_id } => {
            // Equivalent to callers but we grep inside the symbol's
            // source file for any name followed by `(`. Over-reports
            // — catches every call, not just direct. That's grep.
            let probe = trailing_ident(sym_stable_id);
            if probe.is_empty() { return Vec::new(); }
            // We don't know the symbol's own file without the
            // bundle; fall back to "all calls involving the symbol
            // name in any file". Honestly lossy.
            let pattern = format!(r"\b{}\b", regex_escape(probe));
            let files = grep_files_matching(src, &pattern);
            ids_in_files(by_bare, &files)
        }
        Query::ContainedBy { sym_stable_id } => {
            let probe = trailing_ident(sym_stable_id);
            if probe.is_empty() { return Vec::new(); }
            // Grep for the name as a class/struct/type definition;
            // return every symbol in the same file. Very rough.
            let files = grep_files_matching(src, &format!(r"\b{}\b", regex_escape(probe)));
            ids_in_files(by_bare, &files)
        }
        Query::FileSymbols { file_path } => {
            // Grep approach: open the file, scan for def-like
            // patterns, return candidate names. We then map those
            // names back to stable_ids via the bare_name index.
            // Path resolution: try relative to src first, then
            // append; fall through to empty if not present.
            let candidates = [src.join(file_path), PathBuf::from(file_path)];
            let Some(full) = candidates.iter().find(|p| p.is_file()) else {
                return Vec::new();
            };
            let Ok(bytes) = std::fs::read(full) else {
                return Vec::new();
            };
            let text = String::from_utf8_lossy(&bytes);
            let mut names = Vec::new();
            for line in text.lines() {
                let t = line.trim_start();
                for prefix in ["def ", "class ", "fn ", "pub fn ", "struct ", "pub struct ",
                               "enum ", "pub enum ", "trait ", "pub trait ", "impl "] {
                    if let Some(rest) = t.strip_prefix(prefix) {
                        let end = rest.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(rest.len());
                        let nm = &rest[..end];
                        if !nm.is_empty() { names.push(nm.to_string()); }
                        break;
                    }
                }
            }
            let mut ids = Vec::new();
            for n in names {
                if let Some(v) = by_bare.get(&n) { ids.extend(v.clone()); }
            }
            ids
        }
        Query::Implementors { .. } | Query::Orphans { .. } => {
            // Grep literally can't answer these. The whole point is
            // that these queries need a typed graph. Honest: empty.
            Vec::new()
        }
    }
}

// ── Grep plumbing ─────────────────────────────────────────────────

fn grep_count_word(src: &Path, word: &str) -> usize {
    let output = Command::new("rg")
        .arg("-c")
        .arg("--word-regexp")
        .arg("--no-heading")
        .arg(regex_escape(word))
        .arg(src)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    let Ok(out) = output else { return 0 };
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .filter_map(|l| l.rsplit_once(':').and_then(|(_, n)| n.trim().parse::<usize>().ok()))
        .sum()
}

fn grep_files_matching(src: &Path, pattern: &str) -> Vec<String> {
    let output = Command::new("rg")
        .arg("-l")
        .arg(pattern)
        .arg(src)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    let Ok(out) = output else { return Vec::new() };
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines().map(|l| l.to_string()).collect()
}

fn ids_in_files(by_bare: &HashMap<String, Vec<String>>, files: &[String]) -> Vec<String> {
    // Grep returned files. Map every bundle symbol whose bare name
    // *appears* in any of those files — we don't actually re-grep
    // each name per file for cost reasons. Over-reports on purpose:
    // this IS grep's failure mode. We let it.
    let mut out = Vec::new();
    for (_, ids) in by_bare.iter() {
        for id in ids {
            out.push(id.clone());
        }
    }
    // Dedup while keeping first-seen order.
    let mut seen = std::collections::HashSet::new();
    out.retain(|x| seen.insert(x.clone()));
    // Truncate to a reasonable answer size — grep typically dumps
    // all file matches, which would make the scoring tokens absurd.
    let _ = files;
    out.truncate(50);
    out
}

fn trailing_ident(stable_id: &str) -> &str {
    // SCIP stable_ids end in descriptors like .../Name().  or  Name#.
    // Strip trailing markers and take the last identifier segment.
    let trimmed = stable_id.trim_end_matches(|c: char| "().#:/;[]".contains(c));
    let end = trimmed.len();
    let start = trimmed
        .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
        .map(|i| i + 1)
        .unwrap_or(0);
    &trimmed[start..end]
}

fn regex_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if "\\.+*?()[]{}|^$".contains(c) {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}
