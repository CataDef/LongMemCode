# LongMemCode Scoreboard

Live leaderboard for memory systems evaluated on LongMemCode. Every row links to the full result JSON under [`results/`](results/).

**Submit yours via PR** — see [`results/README.md`](results/README.md) for the submission contract.

---

## v0.1 — FastAPI corpus (Python, fastapi/fastapi@0.117.0)

435 scenarios (corpus has 431 symbols so Completion caps at ~150). 400 `scip_roundtrip` + 35 `adversarial`.

| Rank | Adapter | Version | Accuracy | P50 | P95 | P99 | $ / 1 k queries | Result file |
|-----:|---|---|---:|---:|---:|---:|---:|---|
|   1 | **ArgosBrain** | 0.1.0 | **100.0 %** | 58 µs | 87 µs | 114 µs | **$0.00** | [argosbrain-fastapi-2026-04-20.json](results/argosbrain-fastapi-2026-04-20.json) |
|   — | grep-baseline | 0.1.0 | — | — | — | — | — | _run pending_ |
|   — | Mem0 | — | — | — | — | — | — | [submit](results/README.md) |
|   — | Zep | — | — | — | — | — | — | [submit](results/README.md) |
|   — | Letta | — | — | — | — | — | — | [submit](results/README.md) |
|   — | Pure-LLM (Claude / GPT, prompt-stuffed) | — | — | — | — | — | — | [submit](results/README.md) |
|   — | Vector-RAG (OpenAI emb + FAISS) | — | — | — | — | — | — | [submit](results/README.md) |
|   — | _your system_ | — | — | — | — | — | — | [submit](results/README.md) |

## v0.1 — clap corpus (Rust, clap-rs/clap@v4.5.20)

499 scenarios (464 `scip_roundtrip` + 35 `adversarial`).

| Rank | Adapter | Version | Accuracy | P50 | P95 | P99 | $ / 1 k queries | Result file |
|-----:|---|---|---:|---:|---:|---:|---:|---|
|   1 | **ArgosBrain** | 0.1.0 | **100.0 %** | 68 µs | 181 µs | 293 µs | **$0.00** | [argosbrain-clap-2026-04-20.json](results/argosbrain-clap-2026-04-20.json) |
|   — | grep-baseline | 0.1.0 | — | — | — | — | — | _run pending_ |
|   — | _your system_ | — | — | — | — | — | — | [submit](results/README.md) |

## Headline numbers (weighted across both corpora)

| Adapter | Accuracy | P95 | Compression (tokens) | $ / 1 k queries |
|---|---:|---:|---:|---:|
| **ArgosBrain 0.1.0** | **100.0 %** | ≤ 181 µs | See per-corpus results | **$0.00** |
| grep-baseline 0.1.0 | — | — | — | — |

**Headline**: on both corpora, every query returned the correct answer, in sub-millisecond time, with zero dollar cost at read time. A system making an LLM hop per query would spend $0.001 – $0.005 per query and take 200 – 2 000 ms; on this benchmark that gap is visible by running both adapters and reading the table.

**Honest caveat** (see [METHODOLOGY.md](docs/METHODOLOGY.md)): gold source for 92 % of the scenarios is `scip_roundtrip` — the bundle's own structural facts. A 100 % score here means the decoder + bundle writer preserve every edge the SCIP indexer found, end to end, at the reported latency. The `grep_compared` partition (v0.2) will introduce an independent gold source and show where any structural gap hides; expect that score to be < 100 % for some scenarios that a literal text search catches but a typed graph filters out (for *reasons* documented in the per-scenario JSON).

## What the columns mean

- **Accuracy** — weighted across the six scenario categories (see [TAXONOMY.md](docs/TAXONOMY.md)). Higher is better.
- **P95 latency** — 95th-percentile per-query wall clock. Lower is better; IDE inner-loop UX breaks above ~50 ms.
- **Compression** — tokens delivered to the agent over the full run divided by tokens the agent would see if it cat-ed the whole repo. Higher is better.
- **$ / 1 k queries** — cumulative `cost_usd` across 1 000 queries, as reported by the adapter (LLM token charges, embedding API calls, etc.). `$0.00` means the adapter paid nothing at read time. Lower is better; this is where memory-as-$0-per-query wins on the price axis.

---

## Per-category breakdown

Expand a corpus row to see how each adapter performs by scenario category (Completion / BugFix / Refactor / TestGen / FeatureAdd / ApiDiscovery). Per-category scores live inside each result JSON.

## How scoreboard positions are decided

1. **Primary sort**: weighted accuracy (higher is better), tie-break at 0.001.
2. **Secondary sort**: P95 latency (lower is better).
3. **Tertiary sort**: compression (higher is better).

Ties three-way are rare; when they happen we list alphabetically by adapter name and leave a note.

## What counts as a valid result

- Run against the pinned corpus commit (see `corpora/*.sh`).
- Run against the scenario file SHA-256 committed in `scenarios/`.
- Full per-scenario JSONL attached in the PR (or linked — files > 100 kB should be gzipped).
- Machine spec disclosed in the result JSON header.

Results that can't be reproduced from a committed corpus + committed scenario file are rejected.

## Historical versions

Each benchmark version (`v0.1`, `v0.2`, …) is an immutable snapshot of the scenario files + corpus pins. Bumping the version is the only way to reshape the test; existing results stay on their version. See [`results/HISTORY.md`](results/HISTORY.md) for the list.
