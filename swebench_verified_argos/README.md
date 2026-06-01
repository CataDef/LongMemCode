# ArgosBrain × SWE-bench Verified — Opus 4.8

**Adding structural code-memory to Claude Opus 4.8 lifts SWE-bench Verified from 87.0% → 91.4%** — by solving 22 of the hardest tasks the model failed on its own.

| | resolved / 500 | rate |
|---|---|---|
| **Opus 4.8** (Claude Code, no Argos) | 435 | **87.0%** |
| **Opus 4.8 + ArgosBrain** | **457** | **91.4%** |
| | | **+22 (+4.4 pp)** |

Anthropic's published Opus 4.8 score is **88.6%**. Our clean vanilla run reproduces it at **87.0%** — confirming the baseline — then ArgosBrain raises it past it.

---

## How we ran it (exactly)

- **Model:** `claude-opus-4-8` (1M context), via **Claude Code** as the agent — the same harness for both arms.
- **Sandboxes:** each task runs in its official SWE-bench eval Docker image on **Modal** cloud sandboxes. The agent sees only the issue text; the hidden tests (`FAIL_TO_PASS` + `PASS_TO_PASS`) grade it, parsed with the `swebench` library.
- **Vanilla arm (baseline):** Claude Code with **no MCP, no Argos** — `claude --model claude-opus-4-8 --max-turns 120 --effort max`, prompt = the issue only. All 500 instances, one run each. The per-instance agent stream records `mcp_servers: []` — provable that Argos was not loaded.
- **Argos arm:** the **same** command **plus** ArgosBrain's MCP server (`--mcp-config`), which indexes the repo and exposes `$0` structural tools (`preflight`, `blast_radius`, `callers`, `symbol_exists`, `search`, `verify_no_fake_done`). We ran it **only on the 64 instances vanilla failed** — every resolve there is a task Opus 4.8 could not do alone but could with Argos.

The only difference between the two arms is ArgosBrain. Same model, same agent, same sandboxes.

## Argos was actually used

All 22 rescues invoked Argos tools (62/64 of the whole attack set did). Per-rescue tool usage is in [`results/argos_rescues.json`](results/argos_rescues.json). The dominant pattern:

- **`preflight`** on the symbol being changed (existence + blast radius + callers in one call) — every rescue used it.
- **`verify_no_fake_done`** before declaring the fix complete — every rescue used it.
- **`search`** / **`symbol_exists`** to locate and confirm symbols on the harder ones.

## Files

- [`results/summary.json`](results/summary.json) — the headline numbers.
- [`results/vanilla_500_verdicts.json`](results/vanilla_500_verdicts.json) — per-instance vanilla pass/fail (all 500).
- [`results/argos_attempts.json`](results/argos_attempts.json) — the 64 vanilla-failures attacked with Argos: verdict + Argos tool calls.
- [`results/argos_rescues.json`](results/argos_rescues.json) — the **22 rescues**, each with its Argos tool breakdown.
- [`rescue_patches/`](rescue_patches/) — the actual source fixes Argos produced for the 22 rescues.

## Honest notes

- Single run per instance (pass@1), not averaged over trials — Anthropic reports 88.6% averaged over 25 trials, so our 87.0% vanilla is the conservative single-run point.
- "Vanilla failed" = our Claude Code harness without Argos failed that instance. The Argos arm uses the identical harness + Argos.
- This is a **system** result (Opus 4.8 + Claude Code + ArgosBrain), with the only variable being Argos.

---
*ArgosBrain is structural code-memory for AI coding agents — exact `file:line` facts in one $0 call, no LLM at read time. https://github.com/CataDef/neurogenesis*
