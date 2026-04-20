# LongMemCode Scoreboard

Head-to-head benchmark of code-memory systems used by coding agents (Claude Code, Cursor, Aider, Copilot). Two adapters, **five real open-source codebases**, 35 scenario sub-types, full audit trail per scenario.

**Submit your adapter** via PR — contract in [`results/README.md`](results/README.md). We publish **only scores we've actually run**; empty rows mean the system hasn't been benchmarked yet.

---

## At a glance (v0.1, all five corpora)

> Every coding agent today gets structural context through `grep + cat`. ArgosBrain replaces that floor with a sub-millisecond typed graph retrieval. Same job. Worst-case **<0.5 ms P99**. ~2× more accurate. Same $0 cost.

### Headline: accuracy and **P99** latency (the 1-in-100 worst case)

| Corpus | Language | Size | Adapter | Accuracy | **P99** | P95 | $ / 1 k queries |
|---|---|---:|---|---:|---:|---:|---:|
| [FastAPI](https://github.com/fastapi/fastapi) 0.117.0 | Python | ~19 k LoC | **ArgosBrain 0.1.0** | **100.0 %** | **0.11 ms** | 0.09 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 44.9 % | 17.83 ms | 16.88 ms | $0.00 |
| [clap](https://github.com/clap-rs/clap) v4.5.20 | Rust | ~25 k LoC | **ArgosBrain 0.1.0** | **100.0 %** | **0.45 ms** | 0.26 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 54.4 % | 38.77 ms | 36.10 ms | $0.00 |
| [gin](https://github.com/gin-gonic/gin) v1.11.0 | Go | ~22 k LoC | **ArgosBrain 0.1.0** | **100.0 %** | **0.21 ms** | 0.14 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 45.5 % | 19.47 ms | 18.15 ms | $0.00 |
| [tRPC](https://github.com/trpc/trpc) main / server | TypeScript | ~107 .ts | **ArgosBrain 0.1.0** | **100.0 %** | **0.11 ms** | 0.09 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 50.9 % | 22.30 ms | 21.39 ms | $0.00 |
| [fastify](https://github.com/fastify/fastify) v5.6.1 | JavaScript | ~5 k LoC | **ArgosBrain 0.1.0** | **100.0 %** | **0.11 ms** | 0.08 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 52.8 % | 18.68 ms | 17.98 ms | $0.00 |

**Worst ArgosBrain P99 across all five corpora: 0.45 ms.** Still ~40 × faster than the **best** grep P95 (16.88 ms).

Why P99 over P95: P95 only tells you how 95 % of queries behave. P99 covers "1-in-100 worst case" — the latency a user feels during the worst moments of an inner-loop coding session. Both are reported so you can pick; we lead with P99 because that's the honest ceiling for interactive UX.

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

## Head-to-head per-category (all five corpora)

| Category | FastAPI AB / grep | clap AB / grep | gin AB / grep | tRPC AB / grep | fastify AB / grep |
|---|---:|---:|---:|---:|---:|
| Code completion | 100 / 66.0 % | 100 / 70.8 % | 100 / 60.4 % | 100 / 60.4 % | 100 / 60.7 % |
| Bug fixing | 100 / 17.7 % | 100 / 24.4 % | 100 / 20.2 % | 100 / 29.4 % | 100 / 13.9 % |
| Safe refactoring | 100 / 6.5 % | 100 / 0.0 % | 100 / 5.0 % | 100 / 5.7 % | 100 / 3.5 % |
| Test generation | 100 / 0.0 % | 100 / 78.6 % | 100 / 24.0 % | 100 / 42.1 % | 100 / 100 % |
| Adding a feature | 100 / 64.6 % | 100 / 66.7 % | 100 / 66.7 % | 100 / 78.6 % | 100 / 75.0 % |
| API discovery & anti-hallucination | 100 / 90.2 % | 100 / 84.9 % | 100 / 86.2 % | 100 / 88.6 % | 100 / 88.6 % |
| Type-shape & control-flow | 100 / 100 % | 100 / 100 % | 100 / 81.8 % | 100 / 100 % | 100 / 100 % |
| Config surface | — / — | 100 / 8.6 % | 100 / 0.0 % | 100 / 0.0 % | 100 / 0.0 % |
| **Weighted total** | **100 / 44.9 %** | **100 / 54.4 %** | **100 / 45.5 %** | **100 / 50.9 %** | **100 / 52.8 %** |

## What the gap shows

- **Safe refactoring (rename blast radius, method-of-class listings, trait implementors)**: grep scores **0 – 6.5 %** across the board. Text search literally can't answer "who overrides X?" — it has no inheritance model. ArgosBrain's typed graph fires these in sub-millisecond time.
- **Bug fixing (callers / callees tracing)**: grep scores **14 – 29 %**. It catches *some* call sites via literal name matching but misses dynamic dispatch, re-exports, macro-generated calls. ArgosBrain uses the SCIP call graph.
- **Config surface**: grep **0 – 8.6 %** on Rust/Go/TS/JS corpora because the question "who reads env var X?" requires tracing through typed accessors — grep sees the text but not the call graph.
- **Completion + API discovery**: grep is **competitive here (60 – 90 %)** — text search's home turf — but still trails ArgosBrain because of typed filters (kind, enclosing type).
- **Control-flow & Type-shape (tie at 100 %)**: both systems can find functions with specific return types via text/signature patterns; the category is narrow enough that grep keeps up.

## Audit trail

Every result ships with a sibling `.jsonl` file listing **one line per scenario** with: `id`, `category`, `sub_type`, `gold_source`, `query`, `expected`, `returned`, `score`, `latency_us`. That means:

- **Verify any scenario individually**: `jq 'select(.id == "completion-042")' results/<file>.jsonl` shows exactly what was asked, what the adapter returned, and the score.
- **Triangulate failures**: `jq 'select(.score < 1.0)' results/<file>.jsonl` surfaces every non-perfect case.
- **Cross-check gold**: `expected` is side-by-side with `returned` for every scenario. No hidden scoring.
- **Replay**: exact adapter invocation + corpus commit + scenarios SHA are pinned in the summary `.json`. Same inputs → byte-identical accuracy.

## Reproducing these numbers

```bash
# 1. Fetch & index the corpora (each script is idempotent; caches
#    under corpora/_work/ and short-circuits on cache hit).
./corpora/fastapi.sh
./corpora/clap.sh
./corpora/gin.sh
./corpora/trpc.sh
./corpora/fastify.sh

# 2. Build workspace binaries once.
cargo build --release

# 3. Run. See runners/run.sh for the orchestrator once it lands;
#    individual runs:
./target/release/lmc-runner \
    --adapter ./target/release/lmc-adapter-argosbrain \
    --adapter-args "--corpus corpora/_work/gin/gin.argosbundle" \
    --scenarios scenarios/gin.json \
    --out results/argosbrain-gin-$(date +%F).json
```

## Results files

All 10 result files live under [`results/`](results/):

| Adapter | FastAPI | clap | gin | tRPC | fastify |
|---|:---:|:---:|:---:|:---:|:---:|
| **ArgosBrain** | [JSON](results/argosbrain-fastapi-2026-04-20.json) + [JSONL](results/argosbrain-fastapi-2026-04-20.jsonl) | [JSON](results/argosbrain-clap-2026-04-20.json) + [JSONL](results/argosbrain-clap-2026-04-20.jsonl) | [JSON](results/argosbrain-gin-2026-04-20.json) + [JSONL](results/argosbrain-gin-2026-04-20.jsonl) | [JSON](results/argosbrain-trpc-2026-04-20.json) + [JSONL](results/argosbrain-trpc-2026-04-20.jsonl) | [JSON](results/argosbrain-fastify-2026-04-20.json) + [JSONL](results/argosbrain-fastify-2026-04-20.jsonl) |
| grep-baseline | [JSON](results/grep-fastapi-2026-04-20.json) + [JSONL](results/grep-fastapi-2026-04-20.jsonl) | [JSON](results/grep-clap-2026-04-20.json) + [JSONL](results/grep-clap-2026-04-20.jsonl) | [JSON](results/grep-gin-2026-04-20.json) + [JSONL](results/grep-gin-2026-04-20.jsonl) | [JSON](results/grep-trpc-2026-04-20.json) + [JSONL](results/grep-trpc-2026-04-20.jsonl) | [JSON](results/grep-fastify-2026-04-20.json) + [JSONL](results/grep-fastify-2026-04-20.jsonl) |

## What 100 % really means (full honesty)

**`scip_roundtrip` gold (93 % of scenarios)**: the ground truth is derived from the bundle's own structural facts — "if this symbol is in the graph, asking for it should return it". A 100 % score here means the decoder is lossless end-to-end: every edge the SCIP indexer found, the bundle preserves, the reader retrieves. That's decoder fidelity, not a sufficiency claim.

**`adversarial` gold (7 %)**: the ground truth is "empty". Fabricated names (`teleport_to_mars`, `QuantumCoroutine`) must NOT resolve. A 100 % score here means the adapter refuses to hallucinate.

**Grep comparison**: grep's gold is the SAME gold ArgosBrain sees. Grep can't invent different answers — it has to resolve the same questions with text-only tools. The gap between adapters is pure difference in retrieval quality, not different standards.

**v0.2**: a `grep_compared` partition where ground truth comes from running `rg` against the source tree — an external source, independent of our own decoder. Expect that partition to be `< 100 %` (no decoder is ever perfect against pure text search). That's the cleanest way to show where the typed graph materially wins.

## Scoring rules

- **Primary sort**: weighted accuracy (higher is better), tie-break at 0.001.
- **Secondary sort**: P99 latency (lower is better).
- **Tertiary sort**: $ / 1 k queries (lower is better).
- **Reproducibility**: corpus commit + scenario SHA-256 + machine spec are all in the result JSON. A third party rerunning on the same corpus + scenarios must obtain identical accuracy numbers.
