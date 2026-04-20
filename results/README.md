# Submitting your result

So you built a memory adapter, ran it against LongMemCode, and want to put your system on the scoreboard. Here's how.

## The submission

1. Fork this repo.
2. Drop your result JSON under `results/<adapter>-<corpus>-<YYYY-MM-DD>.json`. Example filenames: `results/mem0-fastapi-2026-04-20.json`, `results/zep-clap-2026-04-21.json`.
3. Add a row to [`SCOREBOARD.md`](../SCOREBOARD.md) with a link to your file. Sort is re-run by the maintainers on merge, don't worry about rank.
4. Open a PR with the title `result: <adapter> <version> on <corpus>`.

## The JSON schema

Every result file **must** parse against `results/schema.json` and contain the fields below. A result that doesn't validate gets asked to fix, not rejected — the contract is strict so readers can compare systems, not to gatekeep.

```jsonc
{
  "longmemcode_version": "0.1",
  "adapter": {
    "name": "mem0",
    "version": "1.4.2",
    "repo": "https://github.com/mem0ai/mem0",
    "description": "LLM-backed long-term memory for coding agents."
  },
  "corpus": {
    "name": "fastapi",
    "commit": "0.117.0",
    "scenarios_sha256": "a5c4…e7f8",
    "scenarios_count": 500
  },
  "machine": {
    "cpu": "Apple M2 Pro, 10 cores",
    "ram_gb": 16,
    "os": "macOS 14.4",
    "rustc": null,
    "note": "no other processes pinned to bench cores"
  },
  "summary": {
    "weighted_accuracy": 0.82,
    "raw_accuracy":      0.85,
    "p50_latency_ms": 320,
    "p95_latency_ms": 720,
    "p99_latency_ms": 1180,
    "total_tokens_returned": 84210,
    "baseline_repo_tokens":  820000,
    "compression_ratio":     9.7,
    "total_cost_usd": 1.40,
    "cost_per_1k_queries_usd": 2.80
  },
  "per_category": {
    "completion":    { "n": 160, "passed": 132, "avg_score": 0.82, "avg_precision": null, "avg_recall": null },
    "bug_fix":       { "n": 110, "passed": 80,  "avg_score": 0.78, "avg_precision": 0.82, "avg_recall": 0.74 },
    "refactor":      { "n": 60,  "passed": 48,  "avg_score": 0.80, "avg_precision": 0.85, "avg_recall": 0.76 },
    "test_gen":      { "n": 50,  "passed": 42,  "avg_score": 0.84, "avg_precision": null, "avg_recall": null },
    "feature_add":   { "n": 50,  "passed": 40,  "avg_score": 0.80, "avg_precision": null, "avg_recall": null },
    "api_discovery": { "n": 70,  "passed": 66,  "avg_score": 0.94, "avg_precision": 0.99, "avg_recall": 0.92 }
  },
  "per_scenario_jsonl": "mem0-fastapi-2026-04-20.jsonl.gz",
  "notes": [
    "Used mem0 v1.4.2 with OpenAI text-embedding-3-small. Graph back-end: Neo4j 5.x, local.",
    "Cold start excluded from latency percentiles (first 10 queries). See methodology."
  ]
}
```

**Required fields** — every one of `longmemcode_version`, `adapter.{name,version}`, `corpus.{name,commit,scenarios_sha256}`, `machine.{cpu,ram_gb,os}`, `summary.{weighted_accuracy,p50_latency_ms,p95_latency_ms,cost_per_1k_queries_usd}`, and a full `per_category` map.

**Nice to have**: `per_scenario_jsonl` link to the raw JSONL (checked into `results/` alongside your summary, gzipped if > 100 kB).

## What we check at review time

- **Reproducibility**: `corpus.commit` matches a pinned corpus in `corpora/`, and `scenarios_sha256` matches the committed scenario file. If either has drifted, your adapter was tested against a different benchmark version and the result belongs on that version, not this one.
- **Honesty on `cost_usd`**: the summary number should add up to roughly `cost_per_1k_queries_usd * scenarios_count / 1000` — we verify it. Systems that route through an LLM API should publish real billing or a token-price × usage estimate; systems that don't pay at read time legitimately report `0.00`.
- **Machine fairness**: we don't reject slow machines — just disclose `machine.*` so readers can normalise. Latency numbers from a beefy M3 Max vs a Raspberry Pi aren't comparable at face value; they are comparable once you know both machines.
- **Per-category sanity**: a result claiming 100 % weighted with obvious failures in one category is suspicious. We spot-check three scenarios from the JSONL per review.

## Previously merged results

See [`HISTORY.md`](HISTORY.md) for every submission to date with its adapter version and date.

## Common questions

**My system can only answer some query ops. Can I still submit?**

Yes. Return `{ "results": [] }` for ops you don't handle. You'll score 0 on those scenarios and the per-category breakdown will show the gap; readers pick systems based on the shape of coverage, not just the headline number.

**My system paid for an LLM call but got it for free under my internal credits. What do I report for `cost_usd`?**

Report the public list-price of the calls. LongMemCode scores systems against what *running them would cost any adopter*, not what the adapter author's employer actually paid. Private credits are private.

**Can I submit multiple runs (e.g. retuned my retrieval)?**

Yes, but label them distinctly in the `adapter.version` field and keep the previous runs in `results/` too — the scoreboard shows the latest by default, history is kept in `HISTORY.md`.

**What if I find a bug in LongMemCode itself?**

Open an issue. Corpus errors, scoring edge cases, and scenario-file bugs happen. We publish a `v0.1.1`-style patch version when something material is fixed; old results stay valid against their pinned version.
