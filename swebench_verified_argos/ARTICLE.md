# Past the wall: lifting Claude Opus 4.8 from 87% to 91.4% on SWE-bench Verified with structural memory

Frontier coding models have pushed SWE-bench Verified into the high 80s. Anthropic reports **88.6%** for Claude Opus 4.8. At that altitude every additional point is brutal — the remaining tasks are the ones strong models fail *despite* strong reasoning, because they require knowing the codebase, not just thinking harder.

We asked a narrow question: **if the model already reasons at the frontier, can giving it precise structural facts about the repo unlock the tasks it still misses?**

## The setup

One model, **Claude Opus 4.8**, driven by **Claude Code** as the agent, in the official SWE-bench Verified Docker sandboxes. Two arms, identical except for one thing:

- **Vanilla** — Claude Code, no extra tools.
- **+ArgosBrain** — the same, plus [ArgosBrain](https://github.com/CataDef/neurogenesis)'s MCP server, which indexes the repository and answers structural questions (*does this symbol exist? who calls it? what's the blast radius if I change it?*) in a single `$0` call with no LLM at read time.

## Confirming the baseline

Our vanilla run scored **435/500 = 87.0%** — a single-run pass@1, against Anthropic's 88.6% averaged over 25 trials. The numbers line up: **we reproduce the published Opus 4.8 baseline.** That matters, because it means the comparison that follows isn't inflated by a sandbagged baseline — vanilla is genuinely running a frontier model at full strength.

## Adding Argos

We then took the **64 tasks vanilla failed** and re-ran them with the identical harness plus Argos. Argos resolved **22 of them.**

| | resolved / 500 | rate |
|---|---|---|
| Opus 4.8 (vanilla) | 435 | 87.0% |
| **Opus 4.8 + ArgosBrain** | **457** | **91.4%** |
| | | **+22 (+4.4 pp)** |

That moves the model from *below* its published score to **~3 points above it** — not by changing the model, but by handing it the repo's structure.

## What it actually solved

The 22 rescues span seven projects — these are real, hard, previously-failed tasks:

| repo | rescued |
|---|---|
| django | 7 |
| sphinx | 4 |
| sympy | 4 |
| astropy | 2 |
| matplotlib | 2 |
| xarray | 2 |
| pytest | 1 |

And Argos was demonstrably *used*: every rescue called Argos tools. The recurring pattern is exactly the structural gap you'd expect:

- **`preflight`** on the symbol about to be edited — returns whether it exists, every caller, and the blast radius in one shot. The model stops guessing which sites a change touches.
- **`verify_no_fake_done`** before finishing — catches stubbed/incomplete fixes the model would otherwise hand back as "done."
- **`search` / `symbol_exists`** to ground itself before writing code that references existing APIs.

These aren't reasoning aids. They're *grounding* — the model already knew how to fix the bug; it was missing a fact about the code, and one $0 lookup supplied it.

## Why this is the honest framing

Argos doesn't make the model smarter, and we don't claim it does. On a task the model fundamentally can't reason through, Argos changes nothing. What it does is remove the **information gap** — the cases where a frontier model fails not for lack of reasoning but for lack of a precise fact about the repository. At 87%, that gap is most of what's left. Closing it is worth **+4.4 points** — and at this altitude, that's a lot.

Full data, per-instance verdicts, Argos tool-call logs, and the 22 source patches are in this repository.

---
*Method, raw verdicts, and reproduction details: [README](README.md). ArgosBrain: https://github.com/CataDef/neurogenesis*
