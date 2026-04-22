# mem0 adapter

Submits [`mem0ai`](https://github.com/mem0ai/mem0) as a LongMemCode
adapter. Mem0 is a general-purpose LLM-backed memory system; this adapter
evaluates it on a code-structural benchmark (LongMemCode).

## What it does

- Parses the SCIP JSON of the corpus; extracts all named SCIP
  definitions with source context.
- For each symbol, calls `memory.add(text, user_id, metadata={stable_id})`
  — Mem0's native ingest. Each add triggers an LLM extraction pass in
  Mem0's default flow.
- On `lookup` queries: builds a natural-language query text
  ("Find the canonical SCIP stable id of the `{kind}` named `{name}` ..."),
  passes it to `memory.search(...)`, maps hits back to SCIP stable ids via
  metadata, returns top-k.
- On structural ops (`callers`, `callees`, etc.): returns `results: []` —
  Mem0 has no graph primitive for these queries.

## Running it

```bash
pip install mem0ai chromadb

export OPENAI_API_KEY=...
export MEM0_LLM_MODEL=gpt-4o-mini            # or any supported model
export MEM0_INGEST_WORKERS=8                 # recommended for full-coverage ingest
export MEM0_VECTOR_PATH=/tmp/mem0-lmc-chroma

python3 adapters/mem0/adapter.py \
    --corpus    corpora/_work/kubernetes/kubernetes.argosbundle \
    --source    corpora/_work/kubernetes/source \
    --scip-json corpora/_work/kubernetes/kubernetes.scip.json \
    --user-id   k8s
```

After ingest completes, `--skip-ingest` re-uses the Chroma directory for
query-only runs.

## Configuration used in the 2026-04-22 result

- `mem0ai` 2.0.0 (PyPI)
- LLM: `openai/gpt-4o-mini`
- Embedder: `openai/text-embedding-3-small`
- Vector store: `chroma` 1.5.8 (local)
- Ingest workers: 32

## Known limitations at our test scale

- Under 32-worker parallelism, Chroma exhibits a concurrent-write race
  that produced 16 070 / 58 311 = 27.6 % ingest failures on Kubernetes.
  Result ran on 72.4 % coverage.
- A full-coverage alternative is to lower workers (e.g. 4-8) or serialise
  adds; both produce longer wall-clock ingest time. The result at 72.4 %
  coverage is the one we ran; we report it with that context, not hidden.
- P99 latency on read path (1.677 s) is the natural consequence of Mem0's
  LLM-backed search design and is reported in context in the report at
  [`docs/V0.2_KUBERNETES_REPORT.md`](../../docs/V0.2_KUBERNETES_REPORT.md).

## Intended audience for this adapter

Developers evaluating memory systems for **coding-agent workloads** who
want a clear data point on how a general-purpose memory system scores on
structural code questions.

LongMemCode does not measure Mem0's intended primary workload
(conversational memory). A low score here is not a critique of Mem0's
core job; it is a measurement of scope match. See the report for the full
framing.

## License

MIT.
