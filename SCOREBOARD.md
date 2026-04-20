# LongMemCode Scoreboard

Live leaderboard for memory systems evaluated on LongMemCode. Every row links to the full result JSON under [`results/`](results/).

**Submit yours via PR** — see [`results/README.md`](results/README.md) for the submission contract.

---

## v0.1 — FastAPI corpus (Python, fastapi/fastapi@0.117.0)

| Rank | Adapter | Version | Accuracy | P95 latency | Compression | $ / 1 k queries | Result file |
|-----:|---|---|---:|---:|---:|---:|---|
|   1 | **ArgosBrain** | 0.1.0 | — | — | — | — | _run pending_ |
|   — | grep-baseline | 0.1.0 | — | — | — | — | _run pending_ |
|   — | Mem0 | — | — | — | — | — | [submit](results/README.md) |
|   — | Zep | — | — | — | — | — | [submit](results/README.md) |
|   — | Letta | — | — | — | — | — | [submit](results/README.md) |
|   — | Pure-LLM (Claude / GPT, prompt-stuffed) | — | — | — | — | — | [submit](results/README.md) |
|   — | Vector-RAG (OpenAI emb + FAISS) | — | — | — | — | — | [submit](results/README.md) |
|   — | _your system_ | — | — | — | — | — | [submit](results/README.md) |

## v0.1 — clap corpus (Rust, clap-rs/clap@v4.5.20)

| Rank | Adapter | Version | Accuracy | P95 latency | Compression | $ / 1 k queries | Result file |
|-----:|---|---|---:|---:|---:|---:|---|
|   1 | **ArgosBrain** | 0.1.0 | — | — | — | — | _run pending_ |
|   — | grep-baseline | 0.1.0 | — | — | — | — | _run pending_ |
|   — | _your system_ | — | — | — | — | — | [submit](results/README.md) |

## Headline numbers (weighted across both corpora)

| Adapter | Accuracy | P95 | Compression | $ / 1 k queries |
|---|---:|---:|---:|---:|
| ArgosBrain 0.1.0 | — | — | — | — |
| grep-baseline 0.1.0 | — | — | — | — |

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
