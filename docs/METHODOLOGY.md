# Methodology

How LongMemCode scores a memory system — all arithmetic, no judgement calls.

## Per-scenario scoring

Every scenario has an `expected` clause that dictates which scoring function runs. The possible shapes:

### `exact_symbol`
Top-1 result must equal `stable_id`.
- **Score** = 1.0 if `returned[0] == stable_id`, else 0.0.
- No precision / recall emitted.

### `in_top_k`
`stable_id` must appear within the first `k` results.
- **Score** = 1.0 if present, else 0.0.

### `exact_set`
Returned set must equal `stable_ids` set.
- **Precision** = |returned ∩ expected| / |returned|
- **Recall** = |returned ∩ expected| / |expected|
- **Score** = F1 = 2·P·R / (P + R)
- **Special case**: both sets empty → score = 1.0 (perfect agreement on absence — used for the "fake API must not exist" adversarial scenarios in the Precision Layer).

### `contains`
All ids in `required` must be present in returned. Extras don't penalize.
- **Score** = |required ∩ returned| / |required|
- Precision / recall not emitted (one-sided).

### Why four shapes and not one?
Different workloads have different tolerances. A rename refactor demands exact-set (a missed call-site breaks the build); a method lookup during completion just needs the target in the top-5 (the IDE shows the list). Forcing them onto a single metric punishes systems that make the right trade-off for the scenario.

## Aggregation

- **Per-category average**: arithmetic mean of per-scenario scores for scenarios tagged with that category. Reports `n`, `passed` (score ≥ 0.999), `avg`, and — where applicable — `avg_precision`, `avg_recall`.
- **Weighted accuracy**: `Σ (category_avg × category_weight)` over categories that have at least one scenario. This is the headline number.
- **Raw accuracy**: unweighted arithmetic mean of every per-scenario score. Sanity check against the weighted figure; a large gap means the scenario file has a skewed distribution.

## Ground-truth sources in this benchmark

| Source | ~% of scenarios | What it tests |
|---|---|---|
| **Bundle-derived** | ~60 % | Gold = bundle's own structural facts (callers / callees / contained_by as materialised by the indexer). Tests round-trip fidelity and retrieval speed. Systems scoring < 100 % here are losing edges during ingestion. |
| **Adversarial** | ~20 % | Gold = "not in bundle". Fake method names, typo'd class names, hallucinated signatures. Tests the anti-hallucination layer. A system scoring < 100 % here is inventing APIs. |
| **Grep-compared** *(v0.2)* | ~20 % | Gold = `rg -n` on source. Tests whether the memory catches what literal text search would; tighter than bundle-derived because text search has its own blind spots (dynamic dispatch, macros) the memory is *expected* to resolve. |

Bundle-derived and adversarial are in v0.1; grep-compared lands in v0.2.

## Speed measurement

- Each scenario is timed end-to-end inside the runner: start clock → send JSON request to adapter → parse JSON response → stop clock.
- Reports P50, P95, P99 across all scenarios in the run.
- P95 is the headline latency number because it matches IDE inner-loop UX (tab-completion needs to feel instant, not "usually instant, occasionally 400 ms").
- **Not measured**: adapter cold-start. First query in a session is excluded from the percentile population by convention (it's a one-time cost; systems should warm caches before serving).

## Compression measurement

Two numbers are reported.

- **Bundle / cache bytes vs repo source bytes** — the on-disk artefact the memory system ships or maintains, compared against the unpacked source tree it's derived from. Gzip both. Example: ArgosBrain ships a ~300 KB bundle for the asyncio corpus where the source tree is ~500 KB gzipped → ~1.7× compression.
- **Tokens returned per full run vs "cat the whole repo"** — the *practical* compression the agent sees. For each scenario the adapter returns a ranked list; we sum `approx_tokens(stable_id + signature)` across all scenarios, then divide that by `approx_tokens(repo_source)`. This number is what the cost savings actually come from — IDE "agent mode" dumps context, memory layers hand-pick it.

## Reproducibility contract

Every published result includes:
- Corpus commit hash (the upstream SHA the fetch script checked out)
- Scenario file SHA-256
- Adapter identifier + version string
- Machine spec (CPU model, RAM, OS/kernel, filesystem)
- Seed for any randomised scenario sub-sampling
- Full per-scenario JSONL alongside the summary report

A third party rerunning with the same corpus hash + scenario file SHA must get numerically identical accuracy. Latency will differ with hardware; the reported spec lets readers normalise.

## What this benchmark is *not*

- Not an end-to-end agent benchmark — that's [SWE-bench](https://github.com/princeton-nlp/SWE-bench)'s job. We isolate the memory layer.
- Not a code-generation benchmark — HumanEval / MBPP / BigCodeBench.
- Not an IR benchmark — we don't test "find the most semantically similar code"; that's a different problem.
- Not LLM-judged. Every score here is deterministic arithmetic.
