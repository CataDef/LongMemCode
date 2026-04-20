# LongMemCode

**A public benchmark for evaluating memory systems used by AI coding agents.**

LongMemCode is what [LongMemEval](https://github.com/xiaowu0162/LongMemEval) is for general-purpose conversational memory, but for code. It scores a memory system by how well it serves a coding agent mid-task — completion, bug fix, refactor, test generation, feature add, API discovery, code review — across real open-source codebases, at retrieval latencies that are usable in an IDE inner loop.

If you ship a memory layer that plugs into Cursor / Claude Code / Aider / Cline / Copilot, you should be able to rank yourself on LongMemCode.

## Live scoreboard

See [**SCOREBOARD.md**](SCOREBOARD.md) for the current rankings across every adapter that has submitted a result. The scoreboard breaks systems down on four columns — accuracy, P95 latency, compression, and **dollars per 1 000 queries** — so readers can pick the operating point that fits their budget. `$0.00` for systems that don't pay for a read-time LLM call is an important column, not a missing one.

Submit yours via PR — see [`results/README.md`](results/README.md).

## What this benchmark actually measures

Four orthogonal axes. Every system that implements the [adapter protocol](docs/ADAPTER_PROTOCOL.md) gets a score per axis.

1. **Accuracy** — Given a realistic agent query ("who overrides `Future.cancel`?", "what methods does class `BundleWriter` expose?", "does `teleport_to_mars()` exist on `Task`?"), does the memory return the right answer. Weighted by the estimated frequency with which real coding agents hit that scenario type. See [TAXONOMY.md](docs/TAXONOMY.md) for the 24+ scenario types and their weights.
2. **Speed** — P50 / P95 / P99 retrieval latency per query, and cumulative time for the full 500-scenario suite. Memory systems that need an LLM call at read time (Mem0, Zep, Letta) pay 200-2000 ms per lookup; we want to know.
3. **Compression** — Bytes delivered to the agent per query, compared against a naive "cat every file in the repo" baseline. Token savings matter because they turn into direct LLM-cost savings.
4. **Cost** — Dollars per 1 000 queries, as reported by the adapter itself (LLM token charges, embedding API costs, etc.). Systems that pay nothing at read time — $0 — become Pareto-dominant when they also match on accuracy, and the scoreboard makes that visible instead of letting it hide.

## Scope in v0.1

- **2 corpora**: [FastAPI](https://github.com/fastapi/fastapi) (Python, mid-size, popular) and [clap](https://github.com/clap-rs/clap) (Rust, mid-size, ubiquitous).
- **~500 scenarios per corpus**: spread across the [24 scenario types](docs/TAXONOMY.md) in proportion to estimated workload frequency (Completion 32%, BugFix 22%, Refactor 12%, TestGen 10%, FeatureAdd 10%, ApiDiscovery 14%).
- **2 ground-truth sources, mixed**: bundle-derived (tests round-trip fidelity + speed) and adversarial (fabricated ids; system must correctly say "no"). Next release adds a grep-compared partition for harder signal.
- **3 reference adapters** (published in this repo):
  - [`argosbrain`](adapters/argosbrain/) — the memory system this benchmark was born out of.
  - [`grep-baseline`](adapters/grep-baseline/) — trivial `rg` wrapper. Sets the floor.
  - [`mem0`](adapters/mem0/), [`zep`](adapters/zep/) — stubs. Contribute yours.

## Running it

```bash
# 1. Fetch corpora (git clones + any indexing needed, into ./corpora/_work/).
./corpora/fastapi.sh
./corpora/clap.sh

# 2. Point the runner at your adapter.
./runners/run.sh --adapter argosbrain --corpus fastapi
./runners/run.sh --adapter argosbrain --corpus clap

# Results land in results/<adapter>-<corpus>-<date>.json
```

Adding your own adapter is ~50 LoC — speak the JSON-over-stdio [adapter protocol](docs/ADAPTER_PROTOCOL.md) and point the runner at it.

## Methodology

- Ground truth is **deterministic** — no LLM judge. LongMemCode is a reproducibility-first benchmark; a regression in your system produces the same delta on every re-run.
- Scenarios are **shared across systems** — one JSON file per corpus, committed to this repo. No "but their harness was different" excuses.
- Scoring is **open** — see [METHODOLOGY.md](docs/METHODOLOGY.md) for F1 / top-K / contains semantics per scenario type.
- Numbers are **publishable** — every published result includes the scenario file hash, the corpus commit, the adapter version, and the machine spec. We want you to cite this.

## Relation to other benchmarks

| Benchmark | Scope |
|---|---|
| [LongMemEval](https://github.com/xiaowu0162/LongMemEval) | General long-term conversational memory (Zep / Mem0 / Letta). |
| [SWE-bench](https://github.com/princeton-nlp/SWE-bench) | End-to-end agent task success on GitHub issues. Tests the *full* agent. |
| [HumanEval](https://github.com/openai/human-eval), [MBPP](https://github.com/google-research/google-research/tree/master/mbpp) | Function-level code generation. Tests the LLM, not its memory. |
| [RepoBench](https://github.com/Leolty/repobench), [CrossCodeEval](https://github.com/amazon-science/cceval) | Retrieval-augmented code completion. Closest neighbour to us — we fill the gap of *structural* memory (callers / callees / overrides / Precision Layer) they don't cover. |
| **LongMemCode** | **Memory-system retrieval quality, speed, and compression — in isolation from the LLM, at coding-agent workloads.** |

## Contributing

- Adding a corpus? See `corpora/README.md`.
- Adding an adapter? See `docs/ADAPTER_PROTOCOL.md`.
- Proposing a new scenario type? Open an issue; we want to reach 30+ types before v1.0.

## License

MIT. We want you to read, fork, fight, and improve this.

## Maintainers

- [CataDef](https://github.com/CataDef) — ArgosBrain team.
