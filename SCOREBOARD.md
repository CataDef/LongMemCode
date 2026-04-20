# LongMemCode Scoreboard

Live leaderboard for memory systems evaluated on LongMemCode.

**Submit your result** by opening a PR — contract in [`results/README.md`](results/README.md). We do **not** publish scores we didn't run; if your system's row is empty, it hasn't been benchmarked yet.

---

## At a glance

> ArgosBrain answers 100 % of the taxonomy's queries, in under a quarter of a millisecond, at $0 per 1 000 queries. Every number below is reproducible from the scripts and scenario files in this repo.

| Corpus | Language | Size | Adapter | Accuracy | P95 latency | $ / 1 k queries |
|---|---|---:|---|---:|---:|---:|
| [FastAPI](https://github.com/fastapi/fastapi) 0.117.0 | Python | ~19 k LoC | **ArgosBrain 0.1.0** | **100.0 %** | **0.09 ms** | **$0.00** |
| [clap](https://github.com/clap-rs/clap) v4.5.20 | Rust | ~25 k LoC | **ArgosBrain 0.1.0** | **100.0 %** | **0.23 ms** | **$0.00** |

Full per-category + per-sub-type breakdown below. Every headline is supported by an individual JSON result file in [`results/`](results/).

---

## The 8 scenario categories we measure

LongMemCode scores a memory system on **what a coding agent actually asks at work**. We derived the categories from [JetBrains Dev Ecosystem 2024](https://www.jetbrains.com/lp/devecosystem-2024/), [SWE-bench](https://arxiv.org/abs/2310.06770), and the [Copilot productivity study (CACM 2024)](https://cacm.acm.org/research/measuring-github-copilots-impact-on-productivity/). See [`docs/TAXONOMY.md`](docs/TAXONOMY.md) for the 35 concrete sub-types and their sources.

| # | Category | Weight | Example question a coding agent asks |
|---|---|---:|---|
| 1 | **Code completion** | 28 % | "Give me the full name of the `Task` class I just typed." |
| 2 | **Bug fixing** | 18 % | "Who else calls this broken function?" |
| 3 | **Safe refactoring** | 10 % | "If I rename `add_symbol`, where must I update the callers?" |
| 4 | **Test generation** | 8 % | "What test helpers already exist in this file?" |
| 5 | **Adding a feature** | 8 % | "Where else do we register a new middleware?" |
| 6 | **API discovery & anti-hallucination** | 15 % | "Does `teleport_to_mars()` exist?" (answer: **no**.) |
| 7 | **Type-shape & control-flow** | 5 % | "Which functions return a `Result<T, E>`?" |
| 8 | **Config surface** | 4 % | "Where is `DATABASE_URL` read in the codebase?" |

## Headline results

### v0.1 · FastAPI (Python, 425 scenarios)

| Category                 |   n | Passed | Avg score |
|---|---:|---:|---:|
| Code completion          | 159 | 159 | 100.0 % |
| Bug fixing               |  90 |  90 | 100.0 % |
| Safe refactoring         |  52 |  52 | 100.0 % |
| Test generation          |   1 |   1 | 100.0 % |
| Adding a feature         |  35 |  35 | 100.0 % |
| API discovery & anti-hallucination |  79 |  79 | 100.0 % |
| Type-shape & control-flow |  9 |   9 | 100.0 % |
| Config surface            | — | — | *not sampled (FastAPI core has no config module in scope)* |
| **Weighted total**       | **425** | **425** | **100.0 %** |

**Latency**: P50 0.065 ms · P95 0.089 ms · P99 0.106 ms
**Cost**: $0.00 per 1 000 queries
**Tokens returned**: 33 169 across all 425 scenarios
**Result file**: [`results/argosbrain-fastapi-2026-04-20.json`](results/argosbrain-fastapi-2026-04-20.json)

### v0.1 · clap (Rust, 536 scenarios)

| Category                 |   n | Passed | Avg score |
|---|---:|---:|---:|
| Code completion          | 191 | 191 | 100.0 % |
| Bug fixing               | 101 | 101 | 100.0 % |
| Safe refactoring         |  59 |  59 | 100.0 % |
| Test generation          |  34 |  34 | 100.0 % |
| Adding a feature         |  33 |  33 | 100.0 % |
| API discovery & anti-hallucination |  87 |  87 | 100.0 % |
| Type-shape & control-flow | 17 |  17 | 100.0 % |
| Config surface            | 14 |  14 | 100.0 % |
| **Weighted total**       | **536** | **536** | **100.0 %** |

**Latency**: P50 0.073 ms · P95 0.227 ms · P99 0.345 ms
**Cost**: $0.00 per 1 000 queries
**Tokens returned**: 595 461 across all 536 scenarios
**Result file**: [`results/argosbrain-clap-2026-04-20.json`](results/argosbrain-clap-2026-04-20.json)

---

## What 100 % really means (honest framing)

**What's being measured.** 93 % of v0.1 scenarios use `scip_roundtrip` gold — the ground truth is the bundle's own structural facts (call graphs, class hierarchies, file layouts). The other 7 % are `adversarial` — fabricated names that MUST return empty, testing the adapter's anti-hallucination layer. A 100 % score here says two concrete things: **(a) the decoder preserves every edge the SCIP indexer found, lossless end-to-end**, and **(b) the adapter correctly refuses to invent symbols that don't exist.**

**What's coming in v0.2.** An independent `grep_compared` partition — gold comes from `rg -n` on the source tree, outside our own decoder. That's the partition that will expose any gap between "what our graph says is there" and "what a pure text search finds". Expect the `grep_compared` score to be `<100 %` for some categories that legitimately need typed awareness (e.g. overloaded names) — and that's the score that cleanly breaks ties against text-only or LLM-based baselines. We publish v0.2 with a live grep baseline adapter run side-by-side.

**What this benchmark does NOT claim.** LongMemCode is *not* an end-to-end agent task benchmark (that's [SWE-bench](https://github.com/princeton-nlp/SWE-bench)). It isolates the memory layer — specifically, the structural retrieval it performs between user prompt and LLM call — and scores that layer on accuracy, speed, compression, and dollar cost.

## How to rank yourself

1. Build an adapter that speaks the [JSON-over-stdio protocol](docs/ADAPTER_PROTOCOL.md).
2. Fetch the corpora (`./corpora/fastapi.sh`, `./corpora/clap.sh`).
3. Run `lmc-runner` against your adapter.
4. Drop the result JSON under `results/<adapter>-<corpus>-<date>.json`, open a PR. The scoreboard is regenerated on merge.

The empty rows below are for systems that **haven't run the benchmark yet**. We do not estimate or project their scores; publishing unmeasured numbers would make this scoreboard indistinguishable from marketing. When a real result lands, the row fills.

| Adapter | FastAPI | clap |
|---|:---:|:---:|
| **ArgosBrain 0.1.0** | ✅ [100.0 % · 0.09 ms P95 · $0.00](results/argosbrain-fastapi-2026-04-20.json) | ✅ [100.0 % · 0.23 ms P95 · $0.00](results/argosbrain-clap-2026-04-20.json) |
| grep-baseline | _v0.2_ | _v0.2_ |
| Mem0 | _awaiting submission_ | _awaiting submission_ |
| Zep | _awaiting submission_ | _awaiting submission_ |
| Letta | _awaiting submission_ | _awaiting submission_ |
| Claude / GPT (prompt-stuffed) | _awaiting submission_ | _awaiting submission_ |
| Vector-RAG (OpenAI emb + FAISS) | _awaiting submission_ | _awaiting submission_ |

## Scoring rules

- **Primary sort**: weighted accuracy (higher is better), tie-break at 0.001.
- **Secondary sort**: P95 latency (lower is better).
- **Tertiary sort**: $ / 1 k queries (lower is better).
- **Reproducibility**: corpus commit + scenario SHA-256 + machine spec are all in the result JSON. A third party rerunning on the same corpus + scenario file must obtain identical accuracy numbers.

## Historical versions

LongMemCode scenarios are versioned. `v0.1` is the first tagged release; bumping the version is the only way to reshape the test. Results stay bound to their version forever. See [`results/HISTORY.md`](results/HISTORY.md).
