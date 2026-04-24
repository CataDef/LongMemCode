# LongMemCode Scoreboard

Head-to-head benchmark of code-memory systems used by coding agents (Claude Code, Cursor, Aider, Copilot). Two adapters, **thirty-one real open-source codebases**, 35 scenario sub-types, full audit trail per scenario.

**Submit your adapter** via PR — contract in [`results/README.md`](results/README.md). We publish **only scores we've actually run**; empty rows mean the system hasn't been benchmarked yet.

---

## v0.3 — Kubernetes (pinned oracle, adversarially scoreable)

*Released 2026-04-24. Methodology: [`docs/V0.3_METHODOLOGY.md`](docs/V0.3_METHODOLOGY.md).*

v0.3 strengthens the three v0.2.1 weak-oracle categories
(ConversationalContinuation, RereadCascade, SubAgentContextHandoff)
by pinning concrete top-3 expected stable IDs from a deterministic
reference run. Same 953 scenarios, same protocol; oracle is now
exact-set rather than "non-empty pass." Other adapters are now
adversarially scoreable against this suite — returning *different*
non-empty results no longer passes.

272 of 361 weak scenarios pinned; 89 skipped (reference returned empty,
flagged via `expected.pin_skipped_reason`).

### Headline

| Adapter                 | Scope of system   | Scenarios | Accuracy | P99 latency | $ / 1 k     | Result |
|-------------------------|-------------------|----------:|---------:|------------:|------------:|-------|
| **ArgosBrain 0.7.0**    | Code-structural   |       953 | **99.16 %** | **0.404 ms** | **$0.0000** | [JSON](results/argosbrain-kubernetes-v3-2026-04-24.json) |

> ArgosBrain is the reference adapter the v0.3 oracle is pinned
> against; it scores 100 % on the 272 pinned scenarios by construction.
> The 99.16 % overall reflects 8 misses in the *unpinned* v0.1
> rollover scenarios (4 ambiguous bare-name lookups in Kubernetes
> where the v0.1 oracle picked an obscure variant). Other adapters
> running this suite will face the full pinned set as a strict
> exact-contains test.

---

## v0.2.1 — Kubernetes (workflow-shape categories added)

*Released 2026-04-24. Methodology: [`docs/V0.2_METHODOLOGY.md`](docs/V0.2_METHODOLOGY.md).*

This release adds four new scenario categories to the Kubernetes
corpus that capture **sustained-session workflow shapes** rather than
one-shot retrieval:

- **ConversationalContinuation** — follow-up queries on prior context
- **RereadCascade** — same lookup repeated 3-5× per session
- **BashFeedbackLoop** — Go compiler errors → source symbol (grounded
  in 100 real Kubernetes fix commits mined from git history)
- **SubAgentContextHandoff** — main → sub-agent process handoff

953 total scenarios, deterministic ground truth, no LLM judge. Three
of the four new categories use a "non-empty result is pass" oracle in
v0.2.1 — see methodology §4 for the explicit caveat. v0.3 will pin
exact expected results for these.

### Headline

| Adapter                 | Scope of system   | Scenarios | Accuracy | P99 latency | $ / 1 k     | Result |
|-------------------------|-------------------|----------:|---------:|------------:|------------:|-------|
| **ArgosBrain 0.7.0**    | Code-structural   |       953 | **99.16 %** | **0.402 ms** | **$0.0000** | [JSON](results/argosbrain-kubernetes-v2-2026-04-24.json) |

8/10 categories at 100 %. The two with misses (Completion, RereadCascade)
hit ambiguous bare-name lookups in the v0.1 rollover where the v0.1
oracle had picked an obscure variant (e.g. `csi-test/mock/Service`)
while the structural ranker prefers production code (`pkg/apis/core/Service`).
v0.3 oracle pinning will resolve the ambiguity by canonical-result
freezing.

---

## v0.2 — Kubernetes (scale corpus) · four adapters

*Released 2026-04-22. Full report: [`docs/V0.2_KUBERNETES_REPORT.md`](docs/V0.2_KUBERNETES_REPORT.md).*

This release widens LongMemCode along two axes: a **scale corpus**
(Kubernetes v1.32.0, 38 771 project symbols, 1 456 scenarios) and **two
new reference adapters** that represent different classes of memory
system, not just different performance points within the structural class.

- `sbert-faiss` — dense-semantic baseline, zero LLM.
- `mem0 @ gpt-4o-mini` — the open-source general-purpose LLM-backed
  memory system by Mem0. Evaluated here on a code-structural benchmark;
  Mem0's primary workload is conversational memory, not code retrieval.
  We publish the result with full framing and exact configuration in the
  report linked above, so readers can interpret it in context.

### Headline

| Adapter                 | Scope of system   | Scenarios | Accuracy | P99 latency | $ / 1 k     | Result |
|-------------------------|-------------------|----------:|---------:|------------:|------------:|-------|
| **ArgosBrain 0.1.0**    | Code-structural   |     1 456 | **99.25 %** | **0.366 ms** | **$0.0000** | [JSON](results/argosbrain-kubernetes-2026-04-22.json) |
| grep-baseline 0.1.0     | Text-search floor |     1 456 |   5.48 % |  11.838 ms  |  $0.0000    | [JSON](results/grep-kubernetes-2026-04-22.json) |
| sbert-faiss 0.1.0       | Dense semantic    |       500 |  11.21 % | 327.658 ms  |  $0.0000    | [JSON](results/sbert-faiss-kubernetes-2026-04-22.json) |
| mem0 2.0.0 @ gpt-4o-mini| General memory    |       500 |   4.93 % | 1 677.407 ms|  ~$0.20\*   | [JSON](results/mem0-kubernetes-2026-04-22.json) |

\* Mem0's SDK does not emit per-call token usage; cost is estimated from
observed prompt sizes. Full Mem0 ingest over the corpus cost approximately
$5-8 on `gpt-4o-mini` + `text-embedding-3-small`. See the report for
caveats around Mem0's ingest-time concurrency behaviour at our test scale.

### What this does and does not measure

LongMemCode measures **code-structural retrieval quality** on a code
corpus. A general-purpose memory system (Mem0, Zep, Letta) scoring low
here is not a verdict on that system's core job — it is a measurement of
scope match. The report walks through the per-category breakdown, the
adversarial-scenarios subtlety in `ApiDiscovery`, and the configuration
of every adapter in enough detail that any reader can challenge, rerun,
or submit a tuned configuration of their own.

---

## At a glance (v0.1, all five corpora)

> Every coding agent today gets structural context through `grep + cat`. ArgosBrain replaces that floor with a sub-millisecond typed graph retrieval. Same job. Worst-case **<0.5 ms P99**. ~2× more accurate. Same $0 cost.

### Headline: accuracy and **P99** latency (the 1-in-100 worst case)

| Corpus | Language | Size | Adapter | Accuracy | **P99** | P95 | $ / 1 k queries |
|---|---|---:|---|---:|---:|---:|---:|
| [FastAPI](https://github.com/fastapi/fastapi) 0.117.0 | Python | ~19 k LoC | **ArgosBrain 0.1.0** | **100.0 %** | **0.11 ms** | 0.09 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 44.9 % | 17.83 ms | 16.88 ms | $0.00 |
| [clap](https://github.com/clap-rs/clap) v4.5.20 | Rust | ~25 k LoC | **ArgosBrain 0.1.0** | **100.0 %** | **0.45 ms** | 0.26 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 54.4 % | 38.77 ms | 36.10 ms | $0.00 |
| [gin](https://github.com/gin-gonic/gin) v1.11.0 | Go | ~22 k LoC | **ArgosBrain 0.1.0** | **100.0 %** | **0.21 ms** | 0.14 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 45.5 % | 19.47 ms | 18.15 ms | $0.00 |
| [tRPC](https://github.com/trpc/trpc) main / server | TypeScript | ~107 .ts | **ArgosBrain 0.1.0** | **100.0 %** | **0.11 ms** | 0.09 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 50.9 % | 22.30 ms | 21.39 ms | $0.00 |
| [fastify](https://github.com/fastify/fastify) v5.6.1 | JavaScript | ~5 k LoC | **ArgosBrain 0.1.0** | **100.0 %** | **0.11 ms** | 0.08 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 52.8 % | 18.68 ms | 17.98 ms | $0.00 |
| [commons-lang](https://github.com/apache/commons-lang) 3.14.0 | Java | ~60 k LoC | **ArgosBrain 0.1.0** | **99.2 %** | **0.09 ms** | 0.06 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 7.4 % | 2.05 ms | 1.74 ms | $0.00 |
| [scala-library](https://github.com/scala/scala) 2.13.12 | Scala | ~31 k symbols | **ArgosBrain 0.1.0** | **99.8 %** | **0.42 ms** | 0.33 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 7.5 % | 7.73 ms | 7.28 ms | $0.00 |
| [phpstorm-stubs](https://github.com/JetBrains/phpstorm-stubs) v2025.3 | PHP | ~611 .php | **ArgosBrain 0.1.0** | **100.0 %** | **0.82 ms** | 0.20 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 6.3 % | 4.56 ms | 4.24 ms | $0.00 |
| [ruby-stdlib](https://github.com/ruby/ruby) v3_3_0 | Ruby | ~843 .rb | **ArgosBrain 0.1.0** | **100.0 %** | **0.11 ms** | 0.05 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 9.5 % | 4.32 ms | 4.06 ms | $0.00 |
| [rack](https://github.com/rack/rack) v3.1.8 | Ruby | ~50 .rb | **ArgosBrain 0.1.0** | **100.0 %** | **0.03 ms** | 0.02 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 7.2 % | 1.61 ms | 1.17 ms | $0.00 |
| [scopt](https://github.com/scopt/scopt) v4.1.0 | Scala | ~10 .scala | **ArgosBrain 0.1.0** | **99.2 %** | **0.15 ms** | 0.03 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 9.5 % | 1.14 ms | 1.00 ms | $0.00 |
| [symfony/console](https://github.com/symfony/console) v7.1.5 | PHP | ~254 .php | **ArgosBrain 0.1.0** | **100.0 %** | **0.04 ms** | 0.02 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 9.0 % | 1.31 ms | 1.11 ms | $0.00 |
| [MediatR](https://github.com/jbogard/MediatR) v12.4.1 | C# | ~40 .cs | **ArgosBrain 0.1.0** | **100.0 %** | **0.01 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 16.1 % | 1.43 ms | 1.16 ms | $0.00 |
| [dart-lang/collection](https://github.com/dart-lang/collection) v1.19.0 | Dart | ~29 .dart | **ArgosBrain 0.1.0** | **100.0 %** | **0.03 ms** | 0.02 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 6.7 % | 1.26 ms | 1.07 ms | $0.00 |
| [OpenZeppelin](https://github.com/OpenZeppelin/openzeppelin-contracts) v5.1.0 | Solidity | ~262 .sol | **ArgosBrain 0.1.0** | **100.0 %** | **0.02 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 6.7 % | 1.51 ms | 1.28 ms | $0.00 |
| [FsToolkit.ErrorHandling](https://github.com/demystifyfp/FsToolkit.ErrorHandling) 5.2.0 | F# | ~66 .fs | **ArgosBrain 0.1.0** | **100.0 %** | **0.05 ms** | 0.02 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 7.3 % | 1.26 ms | 1.12 ms | $0.00 |
| [Shopify/dawn](https://github.com/Shopify/dawn) main | Liquid | ~92 .liquid | **ArgosBrain 0.1.0** | **100.0 %** | **0.08 ms** | 0.03 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 24.3 % | 2.27 ms | 1.01 ms | $0.00 |
| [saltstack/salt](https://github.com/saltstack/salt) master | Jinja2 | ~41 .jinja | **ArgosBrain 0.1.0** | **100.0 %** | **0.02 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 40.6 % | 2.46 ms | 1.05 ms | $0.00 |
| [rails/rails](https://github.com/rails/rails) guides/main | ERB | ~10 .erb | **ArgosBrain 0.1.0** | **100.0 %** | **2.04 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 15.5 % | 2.91 ms | 0.80 ms | $0.00 |
| [okhttp](https://github.com/square/okhttp) master | Kotlin | ~525 .kt | **ArgosBrain 0.1.0** | **99.7 %** | **0.33 ms** | 0.06 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 6.0 % | 3.29 ms | 3.09 ms | $0.00 |
| [alamofire](https://github.com/Alamofire/Alamofire) master | Swift | ~97 .swift | **ArgosBrain 0.1.0** | **99.7 %** | **0.05 ms** | 0.03 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 5.7 % | 1.43 ms | 1.23 ms | $0.00 |
| [pagila](https://github.com/devrimgunduz/pagila) master | SQL | ~2 .sql schemas | **ArgosBrain 0.1.0** | **100.0 %** | **0.04 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 7.5 % | 1.61 ms | 1.16 ms | $0.00 |
| [AFNetworking](https://github.com/AFNetworking/AFNetworking) master | Obj-C | ~80 .m/.h | **ArgosBrain 0.1.0** | **99.7 %** | **0.02 ms** | 0.02 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 6.8 % | 1.23 ms | 0.95 ms | $0.00 |
| [luasocket](https://github.com/lunarmodules/luasocket) v3.1.0 | Lua | ~58 .lua | **ArgosBrain 0.1.0** | **100.0 %** | **0.02 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 12.7 % | 1.21 ms | 0.87 ms | $0.00 |
| [ratpack](https://github.com/ratpack/ratpack) master | Groovy | ~398 .groovy | **ArgosBrain 0.1.0** | **100.0 %** | **0.03 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 13.3 % | 2.28 ms | 1.31 ms | $0.00 |
| [bats-core](https://github.com/bats-core/bats-core) v1.12.0 | Bash | ~54 .sh/.bash | **ArgosBrain 0.1.0** | **100.0 %** | **0.02 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 13.0 % | 0.96 ms | 0.79 ms | $0.00 |
| [dplyr](https://github.com/tidyverse/dplyr) main | R | ~195 .R | **ArgosBrain 0.1.0** | **100.0 %** | **0.02 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 11.3 % | 1.46 ms | 0.97 ms | $0.00 |
| [plug](https://github.com/elixir-plug/plug) v1.16.1 | Elixir | ~78 .ex/.exs | **ArgosBrain 0.1.0** | **99.5 %** | **0.01 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 12.7 % | 0.98 ms | 0.87 ms | $0.00 |
| [Distributions.jl](https://github.com/JuliaStats/Distributions.jl) master | Julia | ~255 .jl | **ArgosBrain 0.1.0** | **100.0 %** | **0.02 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 5.8 % | 1.06 ms | 0.93 ms | $0.00 |
| [terraform-aws-vpc](https://github.com/terraform-aws-modules/terraform-aws-vpc) master | HCL | ~77 .tf | **ArgosBrain 0.1.0** | **100.0 %** | **0.08 ms** | 0.03 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 7.0 % | 1.24 ms | 1.08 ms | $0.00 |
| [PSScriptAnalyzer](https://github.com/PowerShell/PSScriptAnalyzer) main | PowerShell | ~222 .ps1 | **ArgosBrain 0.1.0** | **100.0 %** | **0.01 ms** | 0.01 ms | $0.00 |
|   |   |   | grep-baseline 0.1.0 | 6.3 % | 1.05 ms | 0.84 ms | $0.00 |

**Worst ArgosBrain P99 across all sixteen corpora: 0.82 ms.** Still ~40 × faster than the **best** grep P95 (1.74 ms on commons-lang, ~17 ms on the others).

Why P99 over P95: P95 only tells you how 95 % of queries behave. P99 covers "1-in-100 worst case" — the latency a user feels during the worst moments of an inner-loop coding session. Both are reported so you can pick; we lead with P99 because that's the honest ceiling for interactive UX.

---

## The 8 categories we measure

Derived from [JetBrains Dev Ecosystem 2024](https://www.jetbrains.com/lp/devecosystem-2024/), [SWE-bench](https://arxiv.org/abs/2310.06770), and the [Copilot productivity study (CACM 2024)](https://cacm.acm.org/research/measuring-github-copilots-impact-on-productivity/). Full 35-sub-type taxonomy in [`docs/TAXONOMY.md`](docs/TAXONOMY.md).

| # | Category | Weight | Example question |
|---|---|---:|---|
| 1 | **Code completion** | 28 % | "What's the full name of the `Task` class I just typed?" |
| 2 | **Bug fixing** | 18 % | "Who else calls this broken function?" |
| 3 | **Safe refactoring** | 10 % | "If I rename this method, where must I update it?" |
| 4 | **Test generation** | 8 % | "What test helpers already exist in this file?" |
| 5 | **Adding a feature** | 8 % | "Where else do we register a middleware?" |
| 6 | **API discovery & anti-hallucination** | 15 % | "Does `teleport_to_mars()` exist?" (answer: **no**.) |
| 7 | **Type-shape & control-flow** | 5 % | "Which functions return `Result<T, E>`?" |
| 8 | **Config surface** | 4 % | "Where is `DATABASE_URL` read?" |

## Head-to-head per-category (all five corpora)

| Category | FastAPI AB / grep | clap AB / grep | gin AB / grep | tRPC AB / grep | fastify AB / grep |
|---|---:|---:|---:|---:|---:|
| Code completion | 100 / 66.0 % | 100 / 70.8 % | 100 / 60.4 % | 100 / 60.4 % | 100 / 60.7 % |
| Bug fixing | 100 / 17.7 % | 100 / 24.4 % | 100 / 20.2 % | 100 / 29.4 % | 100 / 13.9 % |
| Safe refactoring | 100 / 6.5 % | 100 / 0.0 % | 100 / 5.0 % | 100 / 5.7 % | 100 / 3.5 % |
| Test generation | 100 / 0.0 % | 100 / 78.6 % | 100 / 24.0 % | 100 / 42.1 % | 100 / 100 % |
| Adding a feature | 100 / 64.6 % | 100 / 66.7 % | 100 / 66.7 % | 100 / 78.6 % | 100 / 75.0 % |
| API discovery & anti-hallucination | 100 / 90.2 % | 100 / 84.9 % | 100 / 86.2 % | 100 / 88.6 % | 100 / 88.6 % |
| Type-shape & control-flow | 100 / 100 % | 100 / 100 % | 100 / 81.8 % | 100 / 100 % | 100 / 100 % |
| Config surface | — / — | 100 / 8.6 % | 100 / 0.0 % | 100 / 0.0 % | 100 / 0.0 % |
| **Weighted total** | **100 / 44.9 %** | **100 / 54.4 %** | **100 / 45.5 %** | **100 / 50.9 %** | **100 / 52.8 %** |

## What the gap shows

- **Safe refactoring (rename blast radius, method-of-class listings, trait implementors)**: grep scores **0 – 6.5 %** across the board. Text search literally can't answer "who overrides X?" — it has no inheritance model. ArgosBrain's typed graph fires these in sub-millisecond time.
- **Bug fixing (callers / callees tracing)**: grep scores **14 – 29 %**. It catches *some* call sites via literal name matching but misses dynamic dispatch, re-exports, macro-generated calls. ArgosBrain uses the SCIP call graph.
- **Config surface**: grep **0 – 8.6 %** on Rust/Go/TS/JS corpora because the question "who reads env var X?" requires tracing through typed accessors — grep sees the text but not the call graph.
- **Completion + API discovery**: grep is **competitive here (60 – 90 %)** — text search's home turf — but still trails ArgosBrain because of typed filters (kind, enclosing type).
- **Control-flow & Type-shape (tie at 100 %)**: both systems can find functions with specific return types via text/signature patterns; the category is narrow enough that grep keeps up.

## Audit trail

Every result ships with a sibling `.jsonl` file listing **one line per scenario** with: `id`, `category`, `sub_type`, `gold_source`, `query`, `expected`, `returned`, `score`, `latency_us`. That means:

- **Verify any scenario individually**: `jq 'select(.id == "completion-042")' results/<file>.jsonl` shows exactly what was asked, what the adapter returned, and the score.
- **Triangulate failures**: `jq 'select(.score < 1.0)' results/<file>.jsonl` surfaces every non-perfect case.
- **Cross-check gold**: `expected` is side-by-side with `returned` for every scenario. No hidden scoring.
- **Replay**: exact adapter invocation + corpus commit + scenarios SHA are pinned in the summary `.json`. Same inputs → byte-identical accuracy.

## Reproducing these numbers

```bash
# 1. Fetch & index the corpora (each script is idempotent; caches
#    under corpora/_work/ and short-circuits on cache hit).
./corpora/fastapi.sh
./corpora/clap.sh
./corpora/gin.sh
./corpora/trpc.sh
./corpora/fastify.sh

# 2. Build workspace binaries once.
cargo build --release

# 3. Run. See runners/run.sh for the orchestrator once it lands;
#    individual runs:
./target/release/lmc-runner \
    --adapter ./target/release/lmc-adapter-argosbrain \
    --adapter-args "--corpus corpora/_work/gin/gin.argosbundle" \
    --scenarios scenarios/gin.json \
    --out results/argosbrain-gin-$(date +%F).json
```

## Results files

All 10 result files live under [`results/`](results/):

| Adapter | FastAPI | clap | gin | tRPC | fastify |
|---|:---:|:---:|:---:|:---:|:---:|
| **ArgosBrain** | [JSON](results/argosbrain-fastapi-2026-04-20.json) + [JSONL](results/argosbrain-fastapi-2026-04-20.jsonl) | [JSON](results/argosbrain-clap-2026-04-20.json) + [JSONL](results/argosbrain-clap-2026-04-20.jsonl) | [JSON](results/argosbrain-gin-2026-04-20.json) + [JSONL](results/argosbrain-gin-2026-04-20.jsonl) | [JSON](results/argosbrain-trpc-2026-04-20.json) + [JSONL](results/argosbrain-trpc-2026-04-20.jsonl) | [JSON](results/argosbrain-fastify-2026-04-20.json) + [JSONL](results/argosbrain-fastify-2026-04-20.jsonl) |
| grep-baseline | [JSON](results/grep-fastapi-2026-04-20.json) + [JSONL](results/grep-fastapi-2026-04-20.jsonl) | [JSON](results/grep-clap-2026-04-20.json) + [JSONL](results/grep-clap-2026-04-20.jsonl) | [JSON](results/grep-gin-2026-04-20.json) + [JSONL](results/grep-gin-2026-04-20.jsonl) | [JSON](results/grep-trpc-2026-04-20.json) + [JSONL](results/grep-trpc-2026-04-20.jsonl) | [JSON](results/grep-fastify-2026-04-20.json) + [JSONL](results/grep-fastify-2026-04-20.jsonl) |

## What 100 % really means (full honesty)

**`scip_roundtrip` gold (93 % of scenarios)**: the ground truth is derived from the bundle's own structural facts — "if this symbol is in the graph, asking for it should return it". A 100 % score here means the decoder is lossless end-to-end: every edge the SCIP indexer found, the bundle preserves, the reader retrieves. That's decoder fidelity, not a sufficiency claim.

**`adversarial` gold (7 %)**: the ground truth is "empty". Fabricated names (`teleport_to_mars`, `QuantumCoroutine`) must NOT resolve. A 100 % score here means the adapter refuses to hallucinate.

**Grep comparison**: grep's gold is the SAME gold ArgosBrain sees. Grep can't invent different answers — it has to resolve the same questions with text-only tools. The gap between adapters is pure difference in retrieval quality, not different standards.

**v0.2**: a `grep_compared` partition where ground truth comes from running `rg` against the source tree — an external source, independent of our own decoder. Expect that partition to be `< 100 %` (no decoder is ever perfect against pure text search). That's the cleanest way to show where the typed graph materially wins.

## Scoring rules

- **Primary sort**: weighted accuracy (higher is better), tie-break at 0.001.
- **Secondary sort**: P99 latency (lower is better).
- **Tertiary sort**: $ / 1 k queries (lower is better).
- **Reproducibility**: corpus commit + scenario SHA-256 + machine spec are all in the result JSON. A third party rerunning on the same corpus + scenarios must obtain identical accuracy numbers.
