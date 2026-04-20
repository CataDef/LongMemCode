# Scenario Taxonomy

LongMemCode scores a memory system across six categories of coding-agent workload, broken into 24 concrete scenario types. The category weights below drive the *weighted accuracy* headline number; per-category and per-type scores are always reported separately so any skew in the weights doesn't hide a specific weakness.

## Why these categories?

Frequencies are triangulated from the best public signals available — **no single public dataset publishes a clean "what coding agents do all day" telemetry dump**, so we combined three sources and labelled each number with its provenance.

| Source | Used for |
|---|---|
| [JetBrains State of Developer Ecosystem 2024](https://www.jetbrains.com/lp/devecosystem-2024/), §AI Assistants | The only public dataset with clean % splits on *what* developers ask AI for. |
| [SWE-bench](https://arxiv.org/abs/2310.06770) / [SWE-Lancer](https://openai.com/research/swe-lancer) | Category definitions for bug-fix and IC-SWE vs SWE-Manager splits. |
| [Copilot productivity study (CACM 2024)](https://cacm.acm.org/research/measuring-github-copilots-impact-on-productivity/), [Grounded Copilot (Barke et al. 2023)](https://arxiv.org/abs/2206.15000) | Acceleration vs exploration modes; completion as dominant surface. |

When numbers had to be estimated (no public telemetry), that is flagged inline.

## Category weights

| Category | Weight | Rationale |
|---|---|---|
| Completion | 32 % | JetBrains 2024: 49 % "generating code" + Copilot completion surface dominance. |
| BugFix | 22 % | JetBrains 2024: 38 % "explaining code/errors" + SO 2024 56 % "debugging" (deflated, overlaps Completion). |
| Refactor | 12 % | JetBrains 2024: 20 % "refactoring" + GitClear 2024 rising churn signal. |
| TestGen | 10 % | JetBrains 2024: 28 % "writing tests" (deflated, overlaps with Completion). |
| FeatureAdd | 10 % | SWE-Lancer IC-SWE subset (~40 % of 1 488 tasks are feature-level). |
| ApiDiscovery | 14 % | No direct telemetry — estimated from Copilot API-hallucination failure-mode papers. Kept high because it is the category where Precision-Layer memory wins most visibly. |

Sum = 100 %. Categories 1-5 account for ~86 % of volume on their own; ApiDiscovery is uplift for the specific failure mode memory systems exist to prevent.

---

## The 24 scenario types

Each type maps to one or more retrieval primitives in the [adapter protocol](ADAPTER_PROTOCOL.md). A system that fails a type cleanly — returning an empty result or a wrong-rank answer — loses per-scenario credit, not per-type credit; the scoreboard shows the weakness.

### 1. Completion (32 %)

Agent is mid-token — memory must deliver the right symbol without forcing an extra file read.

| # | Type | Primitive | Example |
|---|---|---|---|
| 1.1 | Lookup class/struct by name | `lookup(bare_name)` | "import `Request` from somewhere in the repo" |
| 1.2 | Lookup method on a known type | `lookup("Type#method")` | "what was `Task.cancel`'s full name?" |
| 1.3 | Signature recall | `lookup` → read `signature` field | "what does `create_task` take?" |
| 1.4 | Import path / source file | `lookup` → `DefinedAt` edge → file | "where is `BaseEventLoop` defined?" |
| 1.5 | Cross-module sibling lookup | `file_symbols(path)` | "what else is in `asyncio.base_events`?" |
| 1.6 | Builder-pattern chain recall | multiple `lookup` | "how do I configure a `BundleWriter`?" |

### 2. BugFix (22 %)

Agent has an error or failing test; memory must surface the fault surface.

| # | Type | Primitive | Example |
|---|---|---|---|
| 2.1 | Callers of a symbol | `callers(sym)` | "who calls this buggy function?" |
| 2.2 | Callees of a symbol | `callees(sym)` | "where does this function push work to?" |
| 2.3 | Override / subclass detection | `implementors(sym)` | "who overrides `Future.cancel`?" |
| 2.4 | Multi-hop impact trace | `callers` ∘ `callers` | "who calls the caller of X?" (depth-2) |
| 2.5 | Exception type discovery | `lookup` on name ending in `Error`/`Exception` | "where is `TimeoutError`?" |
| 2.6 | Class field / data invariants | `contained_by(Type#)` filtered by kind | "what fields does `Future` have?" |
| 2.7 | Error-wrapping pattern | signature scan for `Result<_, _>` / `Optional` | "how do we wrap errors in this crate?" |

### 3. Refactor (12 %)

Agent is renaming / restructuring — memory must enumerate the hit list completely.

| # | Type | Primitive | Example |
|---|---|---|---|
| 3.1 | Methods of a class | `contained_by(Type#)` | "every method of `BundleWriter`" |
| 3.2 | Call-site enumeration | `callers(fn)` | "every place `add_symbol` is called" |
| 3.3 | Enum variant list | `contained_by(Enum#)` | "every `EdgeKind` variant" |
| 3.4 | Trait / Protocol implementors | `implementors(Trait#)` | "every `ScipLanguage` impl" |
| 3.5 | Cross-module caller chain | `callers` over a target in another file | "all external callers across modules" |
| 3.6 | Dead-export detection | `orphans(kind=Function)` | "what's declared but never called?" |

### 4. TestGen (10 %)

Agent is writing a test — memory must expose existing test infrastructure.

| # | Type | Primitive | Example |
|---|---|---|---|
| 4.1 | Existing tests in a file | `file_symbols(tests/…)` | "what tests already live in `test_events.py`?" |
| 4.2 | Fixture / conftest discovery | `file_symbols(conftest.py)` | "what fixtures are shared?" |
| 4.3 | Test-to-prod mapping | `lookup("test_X")` | "is there already a test for `foo`?" |
| 4.4 | Mock / stub pattern | `file_symbols` + name filter | "how have we mocked `X` elsewhere?" |
| 4.5 | Assertion-library idiom | `lookup("assert_*")` across repo | "what assertion style do we use?" |

### 5. FeatureAdd (10 %)

Agent is adding a new feature — memory must expose plug-points.

| # | Type | Primitive | Example |
|---|---|---|---|
| 5.1 | Closest existing feature | multiple `lookup` + `file_symbols` | "where's the nearest similar feature?" |
| 5.2 | Plugin / extension point | `implementors(Trait#)` | "where are `Middleware` impls?" |
| 5.3 | DI / wiring registration | `callers(register_fn)` | "where do new services get wired?" |
| 5.4 | Schema / migration partner file | pattern lookup by path | "models + migrations that go together" |
| 5.5 | Public API surface check | `lookup` filtered by public-visibility convention | "what's currently exported?" |

### 6. ApiDiscovery (14 %) — "Precision Layer"

Anti-hallucination: the memory must say *yes* or *no* deterministically so the LLM stops fabricating APIs that don't exist.

| # | Type | Primitive | Example |
|---|---|---|---|
| 6.1 | Exact symbol exists | `lookup(full_id)` | "does `Bundle::open` exist?" |
| 6.2 | Exact symbol absent | `lookup(fake_id)` must return `[]` | "does `Bundle::teleport_to_mars` exist?" |
| 6.3 | Fake type absent | `lookup("QuantumCoroutine")` | "type the LLM just invented" |
| 6.4 | Typo detection | `lookup` with edit distance ≤ 2 | "`lookup_stabel_id` — did you mean?" |
| 6.5 | Bare-name collision disambiguation | `lookup(short)` returns `N>1`, system must rank | "`new` — which type's constructor?" |
| 6.6 | Signature / arity check | `lookup` → inspect `signature` | "is it one arg or two?" |
| 6.7 | Naming-convention query | sample existing names, infer pattern | "snake_case vs camelCase in this repo" |

---

## Excluded for v0.1 (transparent gaps)

These are workload realities we don't score yet. They will be added as corpora & primitives mature.

- **Stack-trace → source line mapping**. Requires line-indexed retrieval; not in v0.1 bundle format.
- **Similar-code detection** ("is there already an abstraction for this?"). Requires semantic retrieval / embeddings — orthogonal axis.
- **Cross-version API diff** ("what's new in 2.0?"). Requires two bundles + diff tool.
- **Review-impact blast radius depth > 2**. Feasible with current primitives; parking until scoring stabilises.
- **Deprecation-marker scan**. Requires parsing decorators / attributes the structural bundle doesn't keep.

These gaps are disclosed in published results; a system claiming "100 %" on LongMemCode is claiming 100 % within the documented scope, not absolute coverage.

## References

- LongMemEval (Tran et al.) — [github.com/xiaowu0162/LongMemEval](https://github.com/xiaowu0162/LongMemEval). The naming parallel and a lot of methodology influence come from here.
- SWE-bench / SWE-bench Verified (Jimenez et al. 2023). [arxiv.org/abs/2310.06770](https://arxiv.org/abs/2310.06770)
- SWE-Lancer (OpenAI 2025). [openai.com/research/swe-lancer](https://openai.com/research/swe-lancer)
- "Grounded Copilot" (Barke et al. 2023). [arxiv.org/abs/2206.15000](https://arxiv.org/abs/2206.15000)
- JetBrains Dev Ecosystem 2024 — [jetbrains.com/lp/devecosystem-2024/](https://www.jetbrains.com/lp/devecosystem-2024/)
- Stack Overflow Dev Survey 2024 — [survey.stackoverflow.co/2024](https://survey.stackoverflow.co/2024)
