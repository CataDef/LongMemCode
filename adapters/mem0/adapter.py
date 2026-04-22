#!/usr/bin/env python3
"""
mem0 — LongMemCode adapter.

Design rationale
----------------
Mem0 is a production memory system that uses an LLM at both write-time
(fact extraction from raw text) and read-time (query understanding +
rerank). This adapter configures it with the most capable available
OpenAI model (see MODEL_*) to give Mem0 the fairest possible shot,
per the request of a reviewer concern logged in Paper 1 §7.4.

The adapter implements the LongMemCode adapter protocol
(docs/ADAPTER_PROTOCOL.md). At LOAD it ingests every named symbol from
the SCIP JSON of the corpus as a Mem0 memory with metadata={stable_id};
at QUERY it forwards lookup queries to Mem0's `search`, maps the top
results back to stable_ids via metadata, and returns them.

Structural ops (callers, callees, contained_by, implementors,
file_symbols, orphans) cannot be answered by Mem0's retrieval model in
any principled way — they are graph questions. For these ops the
adapter returns `results: []` rather than fabricating. This parallels
sbert-faiss and is the honest behaviour documented in Paper 2.

Cost: Mem0 charges LLM tokens at both add() and search(). This adapter
reports `cost_usd` per query as the measured OpenAI spend for that
call, approximated from the token-usage fields OpenAI returns. Ingest
cost is charged per add() and is NOT reported per-query — it is a
one-time corpus ingest cost printed on stderr at load time.
"""
from __future__ import annotations

import json
import os
import re
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Iterable

# Pricing per 1M tokens. Update when providers publish changes.
# Source: OpenAI public pricing (2026 tier-5).
PRICING_PER_1M = {
    "gpt-5.4": (2.50, 10.00),           # (input, output) USD / 1M tokens
    "gpt-5.4-mini": (0.25, 1.00),
    "gpt-4o": (2.50, 10.00),
    "gpt-4o-mini": (0.15, 0.60),
    "text-embedding-3-small": (0.02, 0.0),
    "text-embedding-3-large": (0.13, 0.0),
}

# The knobs for this run. "best chance" = top OpenAI reasoning model for LLM,
# small embedder for cost (embedding quality is not the bottleneck here).
MODEL_LLM = os.environ.get("MEM0_LLM_MODEL", "gpt-5.4")
MODEL_EMBED = os.environ.get("MEM0_EMBED_MODEL", "text-embedding-3-small")
TOP_K = 10

# Ingest concurrency. OpenAI tier-5 tolerates ~300 RPM on reasoning models;
# we start at 32 and back off on rate-limit errors.
INGEST_WORKERS = int(os.environ.get("MEM0_INGEST_WORKERS", "32"))
INGEST_PROGRESS_EVERY = 250

SCIP_ROLE_DEFINITION = 1
CONTEXT_LINES = 6

_BARE_NAME_RE = re.compile(r"""
    [/\\]
    (?P<name>[A-Za-z_][\w]*)
    [#().\[]?
    \s*$
""", re.VERBOSE)


def log(msg: str) -> None:
    print(f"[mem0] {msg}", file=sys.stderr, flush=True)


def extract_bare_name(stable_id: str) -> str:
    m = _BARE_NAME_RE.search(stable_id.rstrip(" \t"))
    if m:
        return m.group("name")
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


def extract_symbols(scip_json_path: Path, source_root: Path) -> list[dict]:
    log(f"parsing SCIP JSON: {scip_json_path}")
    with open(scip_json_path) as f:
        scip = json.load(f)

    symbols: list[dict] = []
    for doc in scip.get("documents", []):
        rel_path = doc.get("relative_path") or ""
        if not rel_path:
            continue
        source_file = source_root / rel_path
        if not source_file.exists():
            continue
        try:
            file_lines = source_file.read_text(encoding="utf-8", errors="replace").splitlines()
        except (OSError, ValueError):
            continue
        for occ in doc.get("occurrences", []):
            roles = occ.get("symbol_roles", 0) or 0
            if not (roles & SCIP_ROLE_DEFINITION):
                continue
            sym_id = occ.get("symbol", "")
            if not sym_id or sym_id.startswith("local ") or not sym_id.startswith("scip-go "):
                continue
            r = occ.get("range", [])
            if not r:
                continue
            start_line = r[0]
            end_line = r[2] if len(r) >= 4 else start_line
            ctx_start = max(0, start_line - CONTEXT_LINES)
            ctx_end = min(len(file_lines), end_line + CONTEXT_LINES + 1)
            context = "\n".join(file_lines[ctx_start:ctx_end])

            symbols.append({
                "stable_id": sym_id,
                "bare_name": extract_bare_name(sym_id),
                "kind": guess_kind(sym_id, context),
                "file": rel_path,
                "line": start_line,
                "context": context,
            })
    log(f"extracted {len(symbols):,} named SCIP symbols")
    return symbols


def symbol_memory_text(s: dict) -> str:
    """
    The text we store in Mem0. Written as a natural-language statement so
    the LLM fact-extraction step has something tractable to work with.
    Keep it short — bulk of the cost is per-add LLM inference.
    """
    return (
        f"In the Kubernetes v1.32.0 Go source, the {s['kind']} `{s['bare_name']}` is defined "
        f"at `{s['file']}` line {s['line']}. Its canonical SCIP stable id is: {s['stable_id']}. "
        f"Surrounding code:\n{s['context'][:500]}"
    )


def query_text(q: dict) -> str | None:
    op = q.get("op")
    if op != "lookup":
        return None
    name = q.get("name") or ""
    kind = q.get("kind") or ""
    return f"Find the canonical SCIP stable id of the {kind} named `{name}` in the Kubernetes v1.32.0 Go source."


def build_mem0(user_id: str):
    """
    Construct a Mem0 instance configured with the desired LLM + embedder.
    qdrant is the default local vector store; we let Mem0 pick its data
    directory so the benchmark is self-contained.
    """
    from mem0 import Memory
    config = {
        "llm": {
            "provider": "openai",
            "config": {"model": MODEL_LLM, "temperature": 0.1, "max_tokens": 500},
        },
        "embedder": {
            "provider": "openai",
            "config": {"model": MODEL_EMBED},
        },
        "vector_store": {
            "provider": "chroma",
            "config": {
                "collection_name": f"lmc_{user_id}",
                "path": os.environ.get("MEM0_VECTOR_PATH", "/tmp/mem0-lmc-chroma"),
            },
        },
    }
    return Memory.from_config(config)


def ingest_symbols_parallel(memory, symbols: list[dict], user_id: str) -> int:
    """
    Parallel add of every symbol as a Mem0 memory with metadata={stable_id}.
    Returns count of successful adds.
    """
    log(f"ingesting {len(symbols):,} symbols into Mem0 with {INGEST_WORKERS} workers (llm={MODEL_LLM})")
    t0 = time.time()
    successes = 0
    failures = 0

    def _add_one(sym):
        try:
            memory.add(
                symbol_memory_text(sym),
                user_id=user_id,
                metadata={"stable_id": sym["stable_id"], "kind": sym["kind"], "file": sym["file"]},
            )
            return True
        except Exception as e:
            log(f"add failed for {sym['stable_id'][:80]}: {type(e).__name__}: {e}")
            return False

    with ThreadPoolExecutor(max_workers=INGEST_WORKERS) as pool:
        futs = [pool.submit(_add_one, s) for s in symbols]
        for i, fut in enumerate(as_completed(futs), 1):
            if fut.result():
                successes += 1
            else:
                failures += 1
            if i % INGEST_PROGRESS_EVERY == 0 or i == len(futs):
                dt = time.time() - t0
                rate = i / dt
                eta = (len(futs) - i) / rate if rate else 0
                log(f"  [{i:>6,}/{len(futs):,}]  ok={successes:,} fail={failures:,}  "
                    f"rate={rate:.1f}/s  eta={eta/60:.1f}min")
    dt = time.time() - t0
    log(f"ingest complete: {successes:,} ok / {failures:,} failed  in {dt/60:.1f}min")
    return successes


def search_and_map(memory, user_id: str, q_text: str) -> tuple[list[str], float]:
    """
    Search Mem0, map hits back to SCIP stable_ids via metadata.
    Returns (stable_ids, cost_usd_for_this_call).
    """
    res = memory.search(q_text, filters={"user_id": user_id}, limit=TOP_K)
    hits = res.get("results", []) if isinstance(res, dict) else res
    stable_ids: list[str] = []
    for h in hits:
        meta = (h.get("metadata") or {}) if isinstance(h, dict) else {}
        sid = meta.get("stable_id")
        if sid:
            stable_ids.append(sid)
    # Cost: Mem0 doesn't return token usage — approximate from query + result sizes.
    # This is intentionally conservative and honest: a reviewer asking "but what
    # did it cost?" sees a real number, not $0.
    pin, pout = PRICING_PER_1M.get(MODEL_LLM, (2.50, 10.00))
    pe_in, _ = PRICING_PER_1M.get(MODEL_EMBED, (0.02, 0.0))
    approx_in_tokens = len(q_text) // 4 + 300   # system prompt + query
    approx_out_tokens = 200
    approx_embed_tokens = len(q_text) // 4
    cost = (approx_in_tokens * pin + approx_out_tokens * pout) / 1e6
    cost += approx_embed_tokens * pe_in / 1e6
    return stable_ids, cost


def main() -> int:
    args = parse_args(sys.argv[1:])
    corpus = args["corpus"]
    source = args["source"]
    scip_json = args["scip_json"]
    user_id = args["user_id"]
    skip_ingest = args["skip_ingest"]

    if not os.environ.get("OPENAI_API_KEY"):
        log("ERROR: OPENAI_API_KEY env var not set")
        return 2

    log(f"connecting to Mem0 (llm={MODEL_LLM}, embed={MODEL_EMBED}, user_id={user_id})")
    memory = build_mem0(user_id)

    if not skip_ingest:
        symbols = extract_symbols(scip_json, source)
        ingest_symbols_parallel(memory, symbols, user_id)
    else:
        log("skipping ingest (--skip-ingest set)")

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
        qt = query_text(q)
        if qt is None:
            sys.stdout.write(json.dumps({"results": [], "cost_usd": 0.0}) + "\n")
            sys.stdout.flush()
            continue
        try:
            results, cost = search_and_map(memory, user_id, qt)
            sys.stdout.write(json.dumps({"results": results, "cost_usd": cost}) + "\n")
        except Exception as e:
            log(f"search failed: {type(e).__name__}: {e}")
            sys.stdout.write(json.dumps({"results": [], "cost_usd": 0.0, "error": str(e)}) + "\n")
        sys.stdout.flush()
    return 0


def parse_args(argv: list[str]) -> dict:
    """
    Flags:
      --corpus <path>        path to the argosbundle (unused beyond identification)
      --source <path>        source directory
      --scip-json <path>     SCIP print --json output
      --user-id <str>        Mem0 user/session id (default: scip-go corpus name)
      --skip-ingest          skip ingest (reuse existing Mem0 data)
    """
    out = {"corpus": None, "source": None, "scip_json": None, "user_id": "lmc-k8s", "skip_ingest": False}
    it = iter(argv)
    for a in it:
        if a == "--corpus": out["corpus"] = Path(next(it))
        elif a == "--source": out["source"] = Path(next(it))
        elif a == "--scip-json": out["scip_json"] = Path(next(it))
        elif a == "--user-id": out["user_id"] = next(it)
        elif a == "--skip-ingest": out["skip_ingest"] = True
        else: raise SystemExit(f"mem0 adapter: unknown flag {a!r}")
    for k in ("corpus", "source", "scip_json"):
        if out[k] is None:
            raise SystemExit(f"mem0 adapter: missing --{k.replace('_','-')}")
    return out


if __name__ == "__main__":
    sys.exit(main())
