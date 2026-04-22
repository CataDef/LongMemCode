# sbert-faiss adapter

A zero-LLM, CPU-only semantic-retrieval baseline for LongMemCode.

## What it does

- Parses the SCIP JSON of the corpus (produced by `scip print --json` on
  the `.scip` file).
- Extracts every **named definition** (skipping anonymous locals and
  non-gomod symbols), with ±6 lines of source context.
- Embeds each definition with
  `sentence-transformers/all-MiniLM-L6-v2` (384-dim).
- Builds a FAISS HNSW index keyed by **SCIP stable id** as metadata.
- On a `lookup` query, embeds `"{kind} {name}"`, retrieves top-k by
  cosine similarity, returns stable ids.
- On structural ops (`callers`, `callees`, `contained_by`, etc.): returns
  `results: []` — embedding retrieval cannot answer graph queries, and we
  prefer an honest 0 over a fabricated answer.

## Running it

```bash
pip install sentence-transformers faiss-cpu numpy

python3 adapters/sbert-faiss/adapter.py \
    --corpus    corpora/_work/kubernetes/kubernetes.argosbundle \
    --source    corpora/_work/kubernetes/source \
    --scip-json corpora/_work/kubernetes/kubernetes.scip.json
```

Index build on 58 311 Kubernetes symbols: ~75 seconds single-pass, ~9 GB
peak RSS. Cached to `.sbert-faiss-cache/` next to the bundle for subsequent
runs.

## Design rationale

This adapter is intentionally the **most generous naive-semantic
configuration** we can ship without a parameter sweep, per the fairness
argument in [Paper 1 §7.4](https://argosbrain.com/papers/longmemcode-benchmark).
A *better* semantic baseline would use a code-specific embedder
(Jina-Code-v2, voyage-code-3, bge-code-large), tuned `top-k`, tuned
similarity threshold, and a cross-encoder reranker. Any such configuration
is welcome as a PR against this adapter or a new adapter directory.

## License

MIT, same as the rest of the repository.
