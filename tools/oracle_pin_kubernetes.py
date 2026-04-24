#!/usr/bin/env python3
"""
v0.3 oracle pinning — strengthen weak-oracle scenarios.

For every scenario in v2 with `expected.kind: contains, required: []`
(the v0.2 weak oracle), run the query through ArgosBrain offline and
pin the actual returned stable IDs as the canonical expected set.

After pinning:
  - kind: contains, required: [<top-k stable IDs from ArgosBrain>]
  - "non-empty pass" becomes "must contain at least one of these IDs"
  - Adapters that return *different* non-empty results now fail
    rather than getting a free pass

The pinning is deterministic: same bundle, same query, same output.

We use top-3 (not top-1) so the oracle isn't over-fitted to a single
implementation choice. A competing adapter that returns ANY of the
top-3 ArgosBrain results passes — that's still a strong signal.
"""

import json
import subprocess
import sys
import time
from pathlib import Path

V02_PATH = Path("/tmp/lmc2_gen/kubernetes-v2.json")
V03_OUT = Path("/tmp/lmc2_gen/kubernetes-v3.json")
ADAPTER = Path("/Users/catalinjibleanu/LongMemCode/target/release/lmc-adapter-argosbrain")
BUNDLE = Path("/Users/catalinjibleanu/LongMemCode/corpora/_work/kubernetes/kubernetes.argosbundle")

TOP_K_PIN = 3  # how many returned IDs we accept as canonical

assert V02_PATH.exists(), V02_PATH
assert ADAPTER.exists(), ADAPTER
assert BUNDLE.exists(), BUNDLE


def main() -> None:
    with V02_PATH.open() as f:
        scenarios = json.load(f)

    # Only weak-oracle scenarios need pinning
    weak = [
        i
        for i, s in enumerate(scenarios)
        if s.get("expected", {}).get("kind") == "contains"
        and not s.get("expected", {}).get("required")
    ]
    print(
        f"v0.2 has {len(scenarios)} scenarios; {len(weak)} have weak oracle"
        f" (kind=contains, required=[])",
        file=sys.stderr,
    )

    # Build a single batch JSONL of all weak queries for the adapter
    batch = "\n".join(json.dumps({"query": scenarios[i]["query"]}) for i in weak) + "\n"

    print(f"running adapter offline on {len(weak)} queries...", file=sys.stderr)
    t0 = time.time()
    proc = subprocess.run(
        [str(ADAPTER), "--corpus", str(BUNDLE)],
        input=batch,
        capture_output=True,
        text=True,
        timeout=300,
    )
    elapsed = time.time() - t0
    if proc.returncode != 0:
        print(f"adapter failed (exit {proc.returncode})", file=sys.stderr)
        print(proc.stderr[:500], file=sys.stderr)
        sys.exit(1)
    responses = [json.loads(line) for line in proc.stdout.splitlines() if line.strip()]
    print(
        f"adapter returned {len(responses)} responses in {elapsed:.2f}s",
        file=sys.stderr,
    )
    assert len(responses) == len(weak), (
        f"response count mismatch: {len(responses)} vs {len(weak)}"
    )

    # Pin top-K results back into the scenarios
    pinned = 0
    skipped_empty = 0
    for i, resp in zip(weak, responses):
        results = resp.get("results", [])
        if not results:
            # Adapter returned nothing — keep the weak oracle (the scenario
            # is essentially meaningless without expected results)
            scenarios[i]["expected"]["required"] = []
            scenarios[i]["expected"]["pin_skipped_reason"] = "adapter_returned_empty"
            scenarios[i]["gold_source"] += "+pin_skip"
            skipped_empty += 1
            continue
        scenarios[i]["expected"]["required"] = results[:TOP_K_PIN]
        scenarios[i]["expected"]["pin_top_k"] = TOP_K_PIN
        scenarios[i]["expected"]["pinned_against"] = "argosbrain-0.7.0"
        scenarios[i]["gold_source"] += "+pinned"
        pinned += 1

    V03_OUT.write_text(json.dumps(scenarios, indent=2))
    print(
        f"\npinning complete:\n"
        f"  pinned:                  {pinned}\n"
        f"  skipped (adapter empty): {skipped_empty}\n"
        f"  v0.3 scenarios written:  {len(scenarios)}\n"
        f"  output:                  {V03_OUT}",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
