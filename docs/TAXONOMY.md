# Scenario Taxonomy

LongMemCode scores a memory system across **9 categories of coding-agent workload**, broken into **35 concrete sub-types**. Category weights drive the *weighted accuracy* headline; per-category and per-subtype scores are always reported separately so any skew in the weights cannot hide a specific weakness.

## Why these categories?

Frequencies are triangulated from the best public signals available — **no single public dataset publishes a clean "what coding agents do all day" telemetry dump**, so we combined three sources and labelled each number with its provenance.

| Source | Used for |
|---|---|
| [JetBrains State of Developer Ecosystem 2024](https://www.jetbrains.com/lp/devecosystem-2024/), §AI Assistants | The only public dataset with clean % splits on *what* developers ask AI for. |
| [SWE-bench](https://arxiv.org/abs/2310.06770) / [SWE-Lancer](https://openai.com/research/swe-lancer) | Category definitions for bug-fix and IC-SWE vs SWE-Manager splits. |
| [Copilot productivity study (CACM 2024)](https://cacm.acm.org/research/measuring-github-copilots-impact-on-productivity/), [Grounded Copilot (Barke et al. 2023)](https://arxiv.org/abs/2206.15000) | Acceleration vs exploration modes; completion as dominant surface. |

When a percentage had to be estimated (no public telemetry existed), that is flagged inline.

## Category weights

| # | Category | Weight | Rationale |
|---|---|---:|---|
| 1 | Completion | 28 % | JetBrains 2024: 49 % "generating code" + Copilot completion surface dominance. |
| 2 | BugFix | 18 % | JetBrains 2024: 38 % "explaining code/errors" + SO 2024: 56 % "debugging" (deflated, overlaps Completion). |
| 3 | Refactor | 10 % | JetBrains 2024: 20 % "refactoring" + GitClear 2024 rising-churn signal. |
| 4 | TestGen | 8 % | JetBrains 2024: 28 % "writing tests" (deflated, overlaps Completion). |
| 5 | FeatureAdd | 8 % | SWE-Lancer IC-SWE subset (~40 % of their 1 488 tasks are feature-level). |
| 6 | ApiDiscovery + Ambiguity | 15 % | No direct telemetry — estimated from Copilot API-hallucination failure-mode papers. Kept high because this is where the Precision Layer visibly prevents the failure modes memory systems exist to fix. |
| 7 | Control-flow & Type-shape | 5 % | Estimated — no telemetry split. Type-aware retrieval (return-shape, throws-analysis, async/sync) shows up in refactor-impact audits. |
| 8 | Config-surface | 4 % | Estimated — enterprise surveys consistently flag config discovery as a painpoint; we cover the code-side only. |
| 9 | Safety-net | 4 % | Reserved for hand-curated edge cases that don't fit the taxonomy cleanly. |

Sum = 100 %. Categories 1 – 5 account for ~72 % of volume; 6 – 7 (the Precision Layer + Control-flow) are uplift for the specific failure modes memory systems exist to prevent.

---

## The 35 sub-types

Each sub-type maps to one or more retrieval primitives in the [adapter protocol](ADAPTER_PROTOCOL.md). A system that fails a sub-type cleanly — returning an empty result or a wrong-rank answer — loses per-scenario credit, not per-sub-type credit; the scoreboard shows the weakness.

### 1. Completion (28 %)

Agent is mid-token — memory must deliver the right symbol without forcing an extra file read.

| # | Sub-type | Primitive | Example |
|---|---|---|---|
| 1.1 | Lookup class / struct by short name | `lookup(bare_name)` | "import `Request` from somewhere in the repo" |
| 1.2 | Lookup method on a known type | `lookup("Type#method")` | "what was `Task.cancel`'s full name?" |
| 1.3 | Signature recall | `lookup` → read `signature` field | "what does `create_task` take?" |
| 1.4 | Import path / source file | `lookup` → `DefinedAt` edge → file | "where is `BaseEventLoop` defined?" |
| 1.5 | Cross-module sibling lookup | `file_symbols(path)` | "what else is in `asyncio.base_events`?" |
| 1.6 | Builder-pattern chain recall | multiple `lookup` | "how do I configure a `BundleWriter`?" |

### 2. BugFix (18 %)

Agent has an error or failing test; memory must surface the fault surface.

| # | Sub-type | Primitive | Example |
|---|---|---|---|
| 2.1 | Callers of a symbol | `callers(sym)` | "who calls this buggy function?" |
| 2.2 | Callees of a symbol | `callees(sym)` | "where does this function push work to?" |
| 2.3 | Override / subclass detection | `implementors(sym)` | "who overrides `Future.cancel`?" |
| 2.4 | Multi-hop impact trace | `callers` ∘ `callers` | "who calls the caller of X?" (depth-2) |
| 2.5 | Exception type discovery | `lookup` on name ending in `Error` / `Exception` | "where is `TimeoutError`?" |
| 2.6 | Class field / data invariants | `contained_by(Type#)` filtered by kind | "what fields does `Future` have?" |
| 2.7 | Error-wrapping pattern | signature scan for `Result<_, _>` / `Optional` | "how do we wrap errors in this crate?" |

### 3. Refactor (10 %)

Agent is renaming / restructuring — memory must enumerate the hit list completely.

| # | Sub-type | Primitive | Example |
|---|---|---|---|
| 3.1 | Methods of a class | `contained_by(Type#)` | "every method of `BundleWriter`" |
| 3.2 | Call-site enumeration | `callers(fn)` | "every place `add_symbol` is called" |
| 3.3 | Enum variant list | `contained_by(Enum#)` | "every `EdgeKind` variant" |
| 3.4 | Trait / Protocol implementors | `implementors(Trait#)` | "every `ScipLanguage` impl" |
| 3.5 | Cross-module caller chain | `callers` over a target in another file | "all external callers across modules" |
| 3.6 | Dead-export detection | `orphans(kind=Function)` | "what's declared but never called?" |

### 4. TestGen (8 %)

Agent is writing a test — memory must expose existing test infrastructure.

| # | Sub-type | Primitive | Example |
|---|---|---|---|
| 4.1 | Existing tests in a file | `file_symbols(tests/…)` | "what tests already live in `test_events.py`?" |
| 4.2 | Fixture / conftest discovery | `file_symbols(conftest.py)` | "what fixtures are shared?" |
| 4.3 | Test-to-prod mapping | `lookup("test_X")` | "is there already a test for `foo`?" |
| 4.4 | Mock / stub pattern | `file_symbols` + name filter | "how have we mocked `X` elsewhere?" |
| 4.5 | Assertion-library idiom | `lookup("assert_*")` across repo | "what assertion style do we use?" |

### 5. FeatureAdd (8 %)

Agent is adding a new feature — memory must expose plug-points.

| # | Sub-type | Primitive | Example |
|---|---|---|---|
| 5.1 | Closest existing feature | multiple `lookup` + `file_symbols` | "where's the nearest similar feature?" |
| 5.2 | Plugin / extension point | `implementors(Trait#)` | "where are `Middleware` impls?" |
| 5.3 | DI / wiring registration | `callers(register_fn)` | "where do new services get wired?" |
| 5.4 | Schema / migration partner file | pattern lookup by path | "models + migrations that go together" |
| 5.5 | Public API surface check | `lookup` filtered by public-visibility convention | "what's currently exported?" |

### 6. ApiDiscovery + Ambiguity (15 %)

Anti-hallucination and collision resolution: the memory must say *yes* or *no* deterministically so the LLM stops fabricating APIs, and when a short name collides across many real symbols, the memory must rank or narrow honestly.

| # | Sub-type | Primitive | Example |
|---|---|---|---|
| 6.1 | Exact symbol exists | `lookup(full_id)` | "does `Bundle::open` exist?" |
| 6.2 | Exact symbol absent | `lookup(fake_id)` must return `[]` | "does `Bundle::teleport_to_mars` exist?" |
| 6.3 | Fake type absent | `lookup("QuantumCoroutine")` | "type the LLM just invented" |
| 6.4 | Typo detection | `lookup` with edit distance ≤ 2 | "`lookup_stabel_id` — did you mean?" |
| 6.5 | Bare-name collision (list all) | `lookup(short)` returns `N > 1`, runner ranks | "`new` — give me every constructor" |
| 6.6 | Signature / arity check | `lookup` → inspect `signature` | "is it one arg or two?" |
| 6.7 | Naming-convention query | sample existing names, infer pattern | "snake_case vs camelCase in this repo" |
| 6.8 | Disambiguate by enclosing type | `lookup` with `enclosing` hint | "`cancel` — but only on `Task#`" |
| 6.9 | Disambiguate by kind | `lookup` with `kind` filter | "`new` — constructors only, not free functions" |
| 6.10 | Disambiguate by module path | `lookup` with path prefix filter | "`open` — but only in `io::`" |
| 6.11 | Scope-restricted lookup | `file_symbols` + name filter | "match inside one module only" |
| 6.12 | Fuzzy match ranking | `lookup` with edit distance > 0 | "`get_confg` — closest real symbol?" |

### 7. Control-flow & Type-shape (5 %)

Type-level patterns the structural graph can answer — but NOT a full dataflow analysis.

| # | Sub-type | Primitive | Example |
|---|---|---|---|
| 7.1 | Functions returning a given shape | signature scan for `Result<_, _>` / `Option<_>` | "who returns `Result<T, E>`?" |
| 7.2 | Exception / error raisers | `callers` of error types | "who throws `TimeoutError`?" |
| 7.3 | Async / sync shape | signature prefix `async` or kind tag | "list all async functions" |

> **Scope limitation, documented**: full control-flow analysis (conditional branches, reachability, dataflow) is **out of scope for v0.1 structural bundles**. v2.0 will add a dataflow extension to the bundle format; these three sub-types are the subset the current bundle can answer honestly.

### 8. Config-surface (4 %)

Code-side config / env / feature-flag discovery.

| # | Sub-type | Primitive | Example |
|---|---|---|---|
| 8.1 | Env-var call-sites | `callers` of `env::var` / `os.environ.get` | "who reads `DATABASE_URL`?" |
| 8.2 | Config-module imports | `file_symbols` + import filter | "who imports `settings`?" |
| 8.3 | Feature-flag enum usage | `callers` of enum variants | "every site that checks `FeatureFlag::X`" |

> **Scope limitation, documented**: this category covers **code-level** config usage only. `.yaml`, `.toml`, `.env` file wiring and string-literal feature flags (`get_flag("ENABLE_X")`) are **not in scope for v0.1** — they'd require multi-format bundles that go beyond SCIP's code-only coverage. v2.0 is the target for that.

### 9. Safety-net (4 %)

Hand-curated scenarios for edge cases the eight generative categories do not cover cleanly. Lives as a small hand-labelled subset per corpus, updated when a new failure mode is discovered. Transparency: the exact scenarios are in `scenarios/*-safety-net.json` alongside the generated ones.

---

## Ground-truth sources (per scenario)

Every scenario declares its gold source explicitly. See [METHODOLOGY.md](METHODOLOGY.md) for per-source scoring. The label shows up in the result JSON's `per_gold_source` breakdown so readers can see where the score comes from.

| Label | Meaning | ~% in v0.1 |
|---|---|---:|
| `scip_roundtrip` | Gold = bundle's own structural facts. Tests decoder fidelity and retrieval speed. | ~60 % |
| `grep_compared` | Gold = `rg -n` output on the source tree. Independent of our decoder. | ~20 % |
| `adversarial` | Gold = empty. Fabricated ids / typos / hallucinated shapes. | ~10 % |
| `manual` | Hand-curated by a human reviewer. Rare, for edge cases. | ~10 % |

A published result MUST report each source's contribution independently. A 100 % on `scip_roundtrip` alone says "decoder is lossless"; a high `grep_compared` score is the independent signal.

## References

- LongMemEval (Tran et al.) — [github.com/xiaowu0162/LongMemEval](https://github.com/xiaowu0162/LongMemEval). The naming parallel and a lot of methodology influence come from here.
- SWE-bench / SWE-bench Verified (Jimenez et al. 2023). [arxiv.org/abs/2310.06770](https://arxiv.org/abs/2310.06770)
- SWE-Lancer (OpenAI 2025). [openai.com/research/swe-lancer](https://openai.com/research/swe-lancer)
- "Grounded Copilot" (Barke et al. 2023). [arxiv.org/abs/2206.15000](https://arxiv.org/abs/2206.15000)
- JetBrains Dev Ecosystem 2024 — [jetbrains.com/lp/devecosystem-2024/](https://www.jetbrains.com/lp/devecosystem-2024/)
- Stack Overflow Dev Survey 2024 — [survey.stackoverflow.co/2024](https://survey.stackoverflow.co/2024)
