#!/usr/bin/env python3
"""
sbert-faiss — a naive semantic baseline for LongMemCode.

Design rationale
----------------
Paper 1 §7.4 of LongMemCode acknowledges the paradigm-fairness concern with a
semantic-first baseline: a poor one straw-mans the paradigm. This adapter is the
*most generous* semantic-first configuration we can ship without parameter
search, and we explicitly label it as such:

  - model: sentence-transformers/all-MiniLM-L6-v2 (384-dim, CPU-friendly default
    in 2025-2026 semantic-retrieval-on-code writeups)
  - index: FAISS HNSW (HNSW32 + efSearch=64)
  - chunking: *symbol-level*, NOT text-level — we extract every definition the
    SCIP indexer emitted, together with its surrounding ±6 lines of source
    context, and embed that as one document keyed by SCIP stable id. This gives
    the semantic retrieval the same addressable unit (stable id) as the
    structural retrieval it is being compared against. This is deliberately
    more generous than naive line-window chunking, which could not produce
    stable ids as its output.

The adapter is honest about its limits: structural query ops
(`callers`, `callees`, `contained_by`, `implementors`, `file_symbols`,
`orphans`) cannot be answered by embedding retrieval in any principled way —
they require a graph, which is exactly Paper 2's argument. For these ops the
adapter returns `results: []` rather than fabricating an answer. The scoreboard
shows the resulting 0% score for the proof.

On `lookup` (the dominant class) the adapter embeds the query text derived from
the scenario (`{kind} {name}` e.g. "struct FitError") and returns the top-10
SCIP stable ids ranked by cosine similarity.

Cost: $0.0 / query — fully local.
"""
from __future__ import annotations

import json
import os
import pickle
import re
import sys
from pathlib import Path
from typing import Iterable

import numpy as np
import faiss
from sentence_transformers import SentenceTransformer

MODEL_NAME = "sentence-transformers/all-MiniLM-L6-v2"
EMBED_DIM = 384
TOP_K = 10
CONTEXT_LINES = 6  # lines of context around each definition
BATCH_SIZE = 128

# SCIP role bitmask: 1 = Definition. See scip.proto.
SCIP_ROLE_DEFINITION = 1


def log(msg: str) -> None:
    """Stderr only — never touch stdout (reserved for JSON protocol)."""
    print(f"[sbert-faiss] {msg}", file=sys.stderr, flush=True)


def extract_symbols(scip_json_path: Path, source_root: Path) -> list[dict]:
    """
    Walk the SCIP index, extract every *definition* occurrence, and build a
    symbol record with surrounding source context.
    """
    log(f"parsing SCIP JSON: {scip_json_path}")
    with open(scip_json_path) as f:
        scip = json.load(f)

    symbols: list[dict] = []
    skipped_no_def = 0
    skipped_no_file = 0

    for doc in scip.get("documents", []):
        rel_path = doc.get("relative_path") or ""
        if not rel_path:
            continue
        source_file = source_root / rel_path
        if not source_file.exists():
            skipped_no_file += 1
            continue
        try:
            file_lines = source_file.read_text(encoding="utf-8", errors="replace").splitlines()
        except (OSError, ValueError):
            skipped_no_file += 1
            continue

        for occ in doc.get("occurrences", []):
            roles = occ.get("symbol_roles", 0) or 0
            if not (roles & SCIP_ROLE_DEFINITION):
                continue
            sym_id = occ.get("symbol", "")
            if not sym_id:
                continue
            # Skip anonymous locals (scip-go emits per-scope locals keyed
            # "local N"). They cannot match any named scenario expectation
            # and would pollute the top-k with noise.
            if sym_id.startswith("local "):
                continue
            # Only keep SCIP gomod-prefixed named symbols — the format the
            # scenarios are scored against.
            if not sym_id.startswith("scip-go "):
                continue
            r = occ.get("range", [])
            if not r:
                skipped_no_def += 1
                continue
            # SCIP range: [start_line, start_col, end_col]  OR [start_line, start_col, end_line, end_col]
            start_line = r[0]
            if len(r) == 3:
                end_line = start_line
            else:
                end_line = r[2]
            ctx_start = max(0, start_line - CONTEXT_LINES)
            ctx_end = min(len(file_lines), end_line + CONTEXT_LINES + 1)
            context = "\n".join(file_lines[ctx_start:ctx_end])

            bare_name = extract_bare_name(sym_id)
            kind_hint = guess_kind(sym_id, context)

            symbols.append({
                "stable_id": sym_id,
                "bare_name": bare_name,
                "kind_hint": kind_hint,
                "file": rel_path,
                "line": start_line,
                "context": context,
            })

    log(f"extracted {len(symbols)} definitions "
        f"(skipped {skipped_no_def} zero-range, {skipped_no_file} missing-file)")
    return symbols


_BARE_NAME_RE = re.compile(r"""
    [/\\]                     # descriptor boundary
    (?P<name>[A-Za-z_][\w]*) # identifier
    [#().\[]?                # kind marker (#=type, ()=method, .=term, [=type-param)
    \s*$                     # at the tail of the stable id
""", re.VERBOSE)


def extract_bare_name(stable_id: str) -> str:
    """Best-effort short-name extraction from a SCIP stable id."""
    m = _BARE_NAME_RE.search(stable_id.rstrip(" \t"))
    if m:
        return m.group("name")
    # Fallback: last identifier-ish run
    cleaned = re.sub(r"[^\w]", " ", stable_id).split()
    return cleaned[-1] if cleaned else stable_id


def guess_kind(stable_id: str, context: str) -> str:
    tail = stable_id.rstrip()
    if tail.endswith("#"):
        return "type"
    if tail.endswith("()."):
        return "method"
    if tail.endswith("."):
        return "term"
    low = context.lower()
    if "func " in low:
        return "function"
    if "struct {" in low or "struct{" in low:
        return "struct"
    if "interface {" in low or "interface{" in low:
        return "interface"
    return "symbol"


def build_index(symbols: list[dict], model: SentenceTransformer, cache_path: Path) -> tuple[faiss.Index, list[str]]:
    """
    Build or load a FAISS HNSW index over symbol-level documents.

    The document text is `{kind_hint} {bare_name}\n{context}` — a deliberately
    simple formulation. More elaborate prompt engineering would require a
    hyperparameter sweep, which Paper 1 §7.4 argues against for the baseline.
    """
    if cache_path.exists():
        log(f"loading cached index from {cache_path}")
        with open(cache_path, "rb") as f:
            data = pickle.load(f)
        return data["index"], data["stable_ids"]

    texts = [f"{s['kind_hint']} {s['bare_name']}\n{s['context']}" for s in symbols]
    stable_ids = [s["stable_id"] for s in symbols]

    log(f"embedding {len(texts):,} symbol documents with {MODEL_NAME} …")
    embs = model.encode(
        texts,
        batch_size=BATCH_SIZE,
        show_progress_bar=False,
        convert_to_numpy=True,
        normalize_embeddings=True,
    ).astype(np.float32)

    log(f"building FAISS HNSW32 index (cosine via inner product on L2-normalised vectors)")
    index = faiss.IndexHNSWFlat(EMBED_DIM, 32, faiss.METRIC_INNER_PRODUCT)
    index.hnsw.efConstruction = 80
    index.hnsw.efSearch = 64
    index.add(embs)

    log(f"caching index to {cache_path}")
    cache_path.parent.mkdir(parents=True, exist_ok=True)
    with open(cache_path, "wb") as f:
        pickle.dump({"index": index, "stable_ids": stable_ids}, f, protocol=pickle.HIGHEST_PROTOCOL)
    return index, stable_ids


def query_text_for_scenario(q: dict) -> str | None:
    """
    Build an embedding query string from a scenario query object.

    We only embed `lookup` queries. For structural ops we return None and the
    caller responds with an empty result set — the taxonomy paper's prediction.
    """
    op = q.get("op")
    if op != "lookup":
        return None
    name = q.get("name") or ""
    kind = q.get("kind") or ""
    return f"{kind} {name}".strip()


def main() -> int:
    args = parse_args(sys.argv[1:])
    corpus = args["corpus"]
    source = args["source"]
    scip_json = args["scip_json"]

    log(f"loading model {MODEL_NAME} …")
    model = SentenceTransformer(MODEL_NAME)

    symbols = extract_symbols(scip_json, source)

    # Cache per-corpus so a second run on the same bundle skips embedding.
    cache_dir = corpus.parent / ".sbert-faiss-cache"
    cache_path = cache_dir / f"{corpus.stem}.pkl"
    index, stable_ids = build_index(symbols, model, cache_path)
    log(f"index ready: {index.ntotal:,} vectors, {EMBED_DIM}-dim")

    # Protocol loop — one JSON line in, one JSON line out, in order.
    log("ready — awaiting queries on stdin")
    for raw in sys.stdin:
        raw = raw.strip()
        if not raw:
            continue
        try:
            req = json.loads(raw)
        except json.JSONDecodeError as e:
            sys.stdout.write(json.dumps({"results": [], "cost_usd": 0.0, "error": f"bad json: {e}"}) + "\n")
            sys.stdout.flush()
            continue

        q = req.get("query") or {}
        text = query_text_for_scenario(q)
        if text is None:
            # Structural op — honest 0.
            sys.stdout.write(json.dumps({"results": [], "cost_usd": 0.0}) + "\n")
            sys.stdout.flush()
            continue

        emb = model.encode([text], convert_to_numpy=True, normalize_embeddings=True).astype(np.float32)
        _, idxs = index.search(emb, TOP_K)
        results = [stable_ids[i] for i in idxs[0] if 0 <= i < len(stable_ids)]
        sys.stdout.write(json.dumps({"results": results, "cost_usd": 0.0}) + "\n")
        sys.stdout.flush()

    return 0


def parse_args(argv: list[str]) -> dict:
    """
    Flags:
      --corpus    <path-to-.argosbundle>   (used only to derive cache location)
      --source    <path-to-source-dir>     (directory scip-go indexed)
      --scip-json <path-to-scip-print-json>(scip print --json output)
    """
    out = {"corpus": None, "source": None, "scip_json": None}
    it = iter(argv)
    for a in it:
        if a == "--corpus":
            out["corpus"] = Path(next(it))
        elif a == "--source":
            out["source"] = Path(next(it))
        elif a == "--scip-json":
            out["scip_json"] = Path(next(it))
        else:
            raise SystemExit(f"sbert-faiss: unknown flag {a!r}")
    missing = [k for k, v in out.items() if v is None]
    if missing:
        raise SystemExit(f"sbert-faiss: missing required flag(s): {missing}")
    return out


if __name__ == "__main__":
    sys.exit(main())
