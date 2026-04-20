# Adapter Protocol

LongMemCode runs any memory system that speaks a tiny JSON-over-stdio contract. This doc is all you need to plug your own system in.

## What an adapter does

An adapter is a process that:

1. Accepts configuration once (corpus path + any adapter-specific knobs) via CLI flags or environment variables.
2. Reads newline-delimited JSON queries on stdin, one per line.
3. Writes newline-delimited JSON responses on stdout, one per line, **in input order**.
4. Exits when stdin closes.

That's it. No gRPC, no sockets, no HTTP server. Stdio keeps the harness and the adapter in separate address spaces (a segfault in the adapter is visible; state leakage between queries is harder to hide) and keeps the protocol debuggable — you can feed scenarios to your adapter with `cat scenarios.jsonl | ./adapter`.

## Request schema

Each line of stdin is one JSON object with a `query` field. The query shape is one of:

```json
{ "query": { "op": "lookup", "name": "BundleWriter", "bare_name": true, "kind": "struct" } }

{ "query": { "op": "callers",       "sym_stable_id": "..." } }
{ "query": { "op": "callees",       "sym_stable_id": "..." } }
{ "query": { "op": "contained_by",  "sym_stable_id": "..." } }
{ "query": { "op": "implementors",  "sym_stable_id": "..." } }
{ "query": { "op": "orphans",       "kind": "function" } }
{ "query": { "op": "file_symbols",  "file_path": "fastapi/applications.py" } }
```

Fields:

- `op` — always present. One of `lookup`, `callers`, `callees`, `contained_by`, `implementors`, `orphans`, `file_symbols`.
- `name`, `bare_name`, `kind` — lookup-only. `bare_name: true` matches by short identifier; `false` matches by full stable id.
- `sym_stable_id` — the SCIP-style symbol id your adapter saw when it ingested the corpus.
- `file_path` — repo-relative.

## Response schema

Each line of stdout is one JSON object:

```json
{ "results": ["sym-stable-id-1", "sym-stable-id-2", "..."], "cost_usd": 0.0 }
```

- `results` — **ordered** stable-id list. The runner treats position as rank for `in_top_k` scenarios. Set-valued scenarios ignore order.
- `cost_usd` — **optional, defaults to 0.0**. The marginal dollar cost of *this one query*, including any LLM or embedding API hops the adapter paid for. Used to compute the `$/1k queries` column on the scoreboard. Systems that are $0 per query at read time (structural, local) report 0 — and that's an important differentiator on the scoreboard, not a missing field.
- **Exact stable ids**. If your system normalises stable ids differently, carry a translation table internally — the harness compares against the ids the corpus's canonical SCIP indexer produced. The corpus docs disclose the indexer + version.
- **One response per request**. The runner enforces order; returning them out of order or dropping one causes a hard error.
- **Error path**: on any failure, emit `{ "results": [], "cost_usd": 0.0, "error": "free-form message" }`. The runner logs the error and scores 0 for that scenario; it does not abort the run.

## Why `cost_usd` matters

LongMemCode measures three axes — accuracy, latency, cost — because **coding-agent memory is a three-way trade-off**. An LLM-backed memory can beat a structural one on semantic queries but pays in latency and dollars; a grep-baseline is fast and free but fails on multi-hop retrieval; ArgosBrain is 0-cost and sub-millisecond but scoped to structural queries.

The scoreboard shows all three so readers can pick the operating point that fits their budget, not just accuracy:

- **ArgosBrain** — graph + HNSW + keyword, $0 at read time → `cost_usd: 0.0` every query.
- **Mem0 / Zep / Letta** — embed-and-retrieve via an LLM hop → `cost_usd` is the sum of their embedding + retrieve API charges.
- **Pure-LLM baseline** (prompt-stuffing the whole repo) → `cost_usd` is the input-token cost for that call.
- **Vector-RAG (OpenAI embeddings + FAISS locally)** → `cost_usd` is just the embedding call for the query text.

The adapter computes `cost_usd` however it wants — cached token prices × observed usage, or a measured live billing — but it must be honest. Published results get audited; a result that systematically under-reports cost is rejected at PR review.

## CLI contract

Adapters are invoked as:

```
<adapter-binary> --corpus <path-to-corpus-work-dir> [adapter-specific-flags...]
```

The `--corpus` directory contains whatever `corpora/<name>.sh` produced — for most corpora that is `<name>.argosbundle` + a `source/` subdir with the original repo checkout. Your adapter decides what to load.

## Reference adapters in this repo

- [`adapters/argosbrain/`](../adapters/argosbrain/) — wraps `scenario_eval` from the `neurogenesis` crate; `--corpus` expects a `.argosbundle`.
- [`adapters/grep-baseline/`](../adapters/grep-baseline/) — thin `rg` wrapper, sets the floor. Run this on every corpus to bound how hard the benchmark actually is; if a new adapter beats it only by 10 points, the corpus is too easy.
- [`adapters/mem0/`](../adapters/mem0/), [`adapters/zep/`](../adapters/zep/) — stubs. Contribute yours.

## Adding your adapter

1. `mkdir adapters/<yourname>`
2. Ship a `run.sh` (or `run.py`, or a binary) that meets the contract above.
3. `README.md` in your adapter dir listing the one-time setup, environment variables, and any memory-system-specific flags.
4. Open a PR. We'll run it against the published corpora and link results from this repo's scoreboard.

## FAQ

**Why stdio and not gRPC?**
Zero infra. Zero ports. You can smoke-test your adapter with `echo '{"query":...}' | ./adapter`.

**My memory system needs to stay warm across queries — is that allowed?**
Yes. The adapter is a long-running process; cache what you like. The runner invokes it once per corpus and feeds all queries through the same stdin.

**Do I have to use SCIP stable ids?**
Not internally, but the wire format does. You ingest the corpus, you decide how to key your own index; at query time you translate your internal result back into SCIP-style stable ids for comparison. The corpus fetch script documents the indexer (typically `rust-analyzer scip` or `@catadef/scip-python`), so the id format is predictable per corpus.

**Can an adapter call an LLM at query time?**
Yes — but it'll be measured for it. P95 latency with an LLM hop is ~500-2000 ms; systems that pay that cost will show it on the scoreboard next to the accuracy number. Part of the point.
