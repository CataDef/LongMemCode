# Corpora

Each `*.sh` script in this directory produces a reproducible benchmark corpus under `_work/<name>/`.

## Shipped corpora (v0.1)

| Corpus | Language | Source | Pinned release | LoC | Why |
|---|---|---|---|---|---|
| `fastapi` | Python | [fastapi/fastapi](https://github.com/fastapi/fastapi) | `0.117.0` | ~19 k (core only) | Mid-size, modern async, heavy class + method structure. |
| `clap` | Rust | [clap-rs/clap](https://github.com/clap-rs/clap) | `v4.5.20` | ~25 k | Ubiquitous CLI library; every Rust dev verifies it mentally. |

## Running

```bash
./corpora/fastapi.sh   # first run: ~3 min (clone + npm install + index + bundle)
./corpora/clap.sh      # first run: ~2 min (clone + rust-analyzer scip + bundle)
```

Both scripts are **idempotent**. Re-running with the same pinned commit is a no-op; bumping the pin force-rebuilds.

## Adding a corpus

1. `cp fastapi.sh mycorpus.sh` and rewrite the commit pin, clone URL, and indexer invocation.
2. Document the choice here: language, source, pinned release, LoC, one line on why you picked it.
3. Generate scenarios for it (`scenario-gen` from `neurogenesis` produces 500 from a bundle).
4. Drop the scenario JSON in `../scenarios/<name>.json`.
5. Open a PR.

## Why pin commits?

Benchmarks need to produce the same numbers on every re-run. A drifting `HEAD` breaks that. Pinning to a released tag gives us a deterministic corpus at the cost of one commit bump per year per corpus — cheap.

## What we ignore on disk

`_work/` is in `.gitignore`. The corpora clones, SCIP indexes, and bundles are **recreatable from the scripts** — they don't belong in git history. The scripts + the pin are enough.

## Toolchain requirements

- **Python (`fastapi.sh`)**: Node 18+ (for npm) and `@catadef/scip-python` — installed automatically. NPM scope `@catadef` hosts our fork of `@sourcegraph/scip-python` with pyright bumped to 1.1.408 (fixes the wildcard-import crash upstream has open).
- **Rust (`clap.sh`)**: `rust-analyzer` on PATH. Install with `rustup component add rust-analyzer --toolchain stable`.
- **Both**: `argosbrain-bundlegen` on PATH. Build it from the [`neurogenesis`](https://github.com/CataDef/neurogenesis) repo with `cargo build --release -p neurogenesis-bundle-gen --bin argosbrain-bundlegen`.

## Storage footprint

| Corpus | Clone | SCIP | Bundle |
|---|---|---|---|
| fastapi | ~10 MB | ~1 MB | ~200 KB |
| clap | ~5 MB | ~10 MB | ~600 KB |

Numbers are ballpark; rerun the scripts to get exact on your machine.
