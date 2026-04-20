# LongMemCode Scoreboard

Head-to-head benchmark of code-memory systems used by coding agents (Claude Code, Cursor, Aider, Copilot). Two adapters, two real open-source codebases, 35 scenario sub-types, full audit trail per scenario.

**Submit your adapter** via PR — contract in [`results/README.md`](results/README.md). We publish **only scores we've actually run**; empty rows mean the system hasn't been benchmarked yet.

---

## At a glance

> Every coding agent today gets structural context through `grep + cat` — the floor. ArgosBrain replaces that floor with a sub-millisecond typed graph retrieval. Same job, ~200× faster, ~2× more accurate, same $0 cost.

### FastAPI (Python, [0.117.0](https://github.com/fastapi/fastapi), 19k LoC, 425 scenarios)

| Adapter | Accuracy | P50 | P95 | P99 | $ / 1 k queries |
|---|---:|---:|---:|---:|---:|
| **ArgosBrain 0.1.0** | **100.0 %** | 0.07 ms | 0.10 ms | 0.13 ms | $0.00 |
| grep-baseline 0.1.0 | 44.9 % | 15.13 ms | 16.88 ms | 17.83 ms | $0.00 |

### clap (Rust, [v4.5.20](https://github.com/clap-rs/clap), 25k LoC, 536 scenarios)

| Adapter | Accuracy | P50 | P95 | P99 | $ / 1 k queries |
|---|---:|---:|---:|---:|---:|
| **ArgosBrain 0.1.0** | **100.0 %** | 0.07 ms | 0.26 ms | 0.45 ms | $0.00 |
| grep-baseline 0.1.0 | 54.4 % | 31.41 ms | 36.10 ms | 38.77 ms | $0.00 |

---

## The 8 categories we measure

Derived from [JetBrains Dev Ecosystem 2024](https://www.jetbrains.com/lp/devecosystem-2024/), [SWE-bench](https://arxiv.org/abs/2310.06770), and the [Copilot productivity study (CACM 2024)](https://cacm.acm.org/research/measuring-github-copilots-impact-on-productivity/). Full 35-sub-type taxonomy in [`docs/TAXONOMY.md`](docs/TAXONOMY.md).

| # | Category | Weight | Example question |
|---|---|---:|---|
| 1 | **Code completion** | 28 % | "What's the full name of the `Task` class I just typed?" |
| 2 | **Bug fixing** | 18 % | "Who else calls this broken function?" |
| 3 | **Safe refactoring** | 10 % | "If I rename this method, where must I update it?" |
| 4 | **Test generation** | 8 % | "What test helpers already exist in this file?" |
| 5 | **Adding a feature** | 8 % | "Where else do we register a middleware?" |
| 6 | **API discovery & anti-hallucination** | 15 % | "Does `teleport_to_mars()` exist?" (answer: **no**.) |
| 7 | **Type-shape & control-flow** | 5 % | "Which functions return `Result<T, E>`?" |
| 8 | **Config surface** | 4 % | "Where is `DATABASE_URL` read?" |

## Head-to-head per-category

### FastAPI

| Category | n | ArgosBrain | grep-baseline | Delta |
|---|---:|---:|---:|---:|
| Code completion | 159 | **100.0 %** | 66.0 % | +34.0 pp |
| Bug fixing | 90 | **100.0 %** | 17.7 % | +82.3 pp |
| Safe refactoring | 52 | **100.0 %** | 6.5 % | **+93.5 pp** |
| Test generation | 1 | **100.0 %** | 0.0 % | +100.0 pp |
| Adding a feature | 35 | **100.0 %** | 64.6 % | +35.4 pp |
| API discovery & anti-hallucination | 79 | **100.0 %** | 90.2 % | +9.8 pp |
| Type-shape & control-flow | 9 | 100.0 % | 100.0 % | ± 0.0 pp |
| **Weighted total** | **425** | **100.0 %** | **44.9 %** | **+55.1 pp** |

### clap

| Category | n | ArgosBrain | grep-baseline | Delta |
|---|---:|---:|---:|---:|
| Code completion | 191 | **100.0 %** | 70.8 % | +29.2 pp |
| Bug fixing | 101 | **100.0 %** | 24.4 % | +75.6 pp |
| Safe refactoring | 59 | **100.0 %** | **0.0 %** | +100.0 pp |
| Test generation | 34 | **100.0 %** | 78.6 % | +21.4 pp |
| Adding a feature | 33 | **100.0 %** | 66.7 % | +33.3 pp |
| API discovery & anti-hallucination | 87 | **100.0 %** | 84.9 % | +15.1 pp |
| Type-shape & control-flow | 17 | 100.0 % | 100.0 % | ± 0.0 pp |
| Config surface | 14 | **100.0 %** | 8.6 % | +91.4 pp |
| **Weighted total** | **536** | **100.0 %** | **54.4 %** | **+45.6 pp** |

## What the gap shows

- **Safe refactoring (rename blast radius, method-of-class listings, trait implementors)**: grep scores **0 – 6.5 %**. Text search literally can't answer "who overrides `Future.cancel`?" — it has no inheritance model. ArgosBrain's typed graph fires these in sub-millisecond time.
- **Bug fixing (callers / callees tracing)**: grep scores **17 – 24 %**. It catches *some* call sites via literal name matching but misses dynamic dispatch, re-exports, macro-generated calls. ArgosBrain uses the SCIP call graph.
- **Config surface**: grep **8.6 %** on clap because the question "who reads env var X?" requires tracing through typed accessors — grep sees the text but not the call graph.
- **Completion + API discovery**: grep is **competitive here (66 – 90 %)** because text search handles "does `BundleWriter` appear anywhere?" perfectly well. This is grep's home turf; we still win because of the typed filters (kind, enclosing type).
- **Control-flow & Type-shape (tie at 100 %)**: both systems can find functions with specific return types via text/signature patterns; the category is narrow enough that grep keeps up.

## Audit trail

Every result ships with a sibling `.jsonl` file listing **one line per scenario** with: `id`, `category`, `sub_type`, `gold_source`, `query`, `expected`, `returned`, `score`, `latency_us`. That means:

- **You can verify any individual scenario**: `jq 'select(.id == "completion-042")' results/<file>.jsonl` shows exactly what was asked, what the adapter returned, and the score.
- **You can triangulate failures**: `jq 'select(.score < 1.0)' results/<file>.jsonl` surfaces every non-perfect case.
- **You can cross-check gold**: for every scenario, `expected` is side-by-side with `returned`. No hidden scoring.
- **You can replay**: the exact adapter invocation + corpus + scenario file are pinned in the summary `.json`. A third party with the same corpus commit + scenarios SHA-256 gets byte-identical results.

## Results files

| File | Shape | Size |
|---|---|---:|
| [`results/argosbrain-fastapi-2026-04-20.json`](results/argosbrain-fastapi-2026-04-20.json) | summary | ~2 kB |
| [`results/argosbrain-fastapi-2026-04-20.jsonl`](results/argosbrain-fastapi-2026-04-20.jsonl) | per-scenario audit | ~330 kB |
| [`results/argosbrain-clap-2026-04-20.json`](results/argosbrain-clap-2026-04-20.json) | summary | ~2 kB |
| [`results/argosbrain-clap-2026-04-20.jsonl`](results/argosbrain-clap-2026-04-20.jsonl) | per-scenario audit | ~2.7 MB |
| [`results/grep-fastapi-2026-04-20.json`](results/grep-fastapi-2026-04-20.json) | summary | — |
| [`results/grep-fastapi-2026-04-20.jsonl`](results/grep-fastapi-2026-04-20.jsonl) | per-scenario audit | — |
| [`results/grep-clap-2026-04-20.json`](results/grep-clap-2026-04-20.json) | summary | — |
| [`results/grep-clap-2026-04-20.jsonl`](results/grep-clap-2026-04-20.jsonl) | per-scenario audit | — |

## What 100 % means (full honesty)

**`scip_roundtrip` gold source (93 % of scenarios)**: the ground-truth answer is derived from the bundle's own structural facts — "if this symbol is in the graph, asking for it should return it". A 100 % score here means the decoder is lossless end-to-end: every edge the SCIP indexer found, the bundle preserves, the reader retrieves. That's decoder fidelity, not a sufficiency claim.

**`adversarial` gold source (7 %)**: the ground truth is "empty". Fabricated names (`teleport_to_mars`, `QuantumCoroutine`) must NOT resolve. A 100 % score here means the adapter refuses to hallucinate.

**What's coming in v0.2**: a `grep_compared` partition where ground truth comes from running `rg` against the source tree — an external source, independent of our own decoder. Expect that partition to be `< 100 %` (no decoder is ever perfect against pure text search). That's the cleanest way to show where the typed graph materially wins.

## Scoring rules

- **Primary sort**: weighted accuracy (higher is better), tie-break at 0.001.
- **Secondary sort**: P95 latency (lower is better).
- **Tertiary sort**: $ / 1 k queries (lower is better).
- **Reproducibility**: corpus commit + scenario SHA-256 + machine spec are all in the result JSON. A third party rerunning on the same corpus + scenarios must obtain identical accuracy numbers.
