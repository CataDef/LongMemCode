#!/usr/bin/env bash
# Fetch + index the FastAPI `fastapi/` core module for LongMemCode.
#
# What this script produces under _work/fastapi/:
#
#   source/         shallow clone of fastapi/fastapi at the pinned commit
#   fastapi.scip    raw SCIP index from @catadef/scip-python
#   fastapi.argosbundle    tier-2 bundle produced by argosbrain-bundlegen
#
# Re-running is idempotent — the script deletes _work/fastapi/ if the
# pinned commit changes, otherwise fast-paths out.

set -euo pipefail

# ── Reproducibility pins ──────────────────────────────────────────────
# Bump these when you refresh the corpus. Commit is a FastAPI tag — pick
# a release, not HEAD, so the scenarios don't drift under us.
FASTAPI_COMMIT="0.117.0"
SCIP_PYTHON_VERSION="0.6.6-catadef.0"   # matches package.json on npm

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/fastapi"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/fastapi.scip"
BUNDLE_OUT="${WORK}/fastapi.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: FastAPI ──"
echo "pinned commit: ${FASTAPI_COMMIT}"

# ── Cache check ────────────────────────────────────────────────────────
if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${FASTAPI_COMMIT}" ] \
    && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit — bundle already built for ${FASTAPI_COMMIT}"
    ls -la "${BUNDLE_OUT}"
    exit 0
fi
rm -rf "${WORK}"
mkdir -p "${WORK}"

# ── 1. Clone ──────────────────────────────────────────────────────────
echo "cloning fastapi/fastapi@${FASTAPI_COMMIT} …"
git clone --depth=1 --branch="${FASTAPI_COMMIT}" \
    https://github.com/fastapi/fastapi.git "${SOURCE_DIR}" 2>&1 | tail -3

# ── 2. Install scip-python (our fork, pinned) ─────────────────────────
if ! command -v scip-python >/dev/null 2>&1; then
    echo "installing @catadef/scip-python@${SCIP_PYTHON_VERSION} globally…"
    npm install -g "@catadef/scip-python@${SCIP_PYTHON_VERSION}" 2>&1 | tail -3
fi
INSTALLED_VERSION="$(scip-python --version 2>&1 | head -1 || true)"
echo "scip-python installed version: ${INSTALLED_VERSION}"

# ── 3. Index the `fastapi/` core module only ──────────────────────────
# Skip docs_src/ (tutorials) + tests/ — they inflate symbol count
# without contributing to the LongMemCode scenario surface. If you want
# the full tree, add --target tests here too.
echo "indexing fastapi/ core with scip-python (NODE_OPTIONS=max-old-space-size=8192)…"
(
    cd "${SOURCE_DIR}"
    NODE_OPTIONS="--max-old-space-size=8192" scip-python index \
        --project-name fastapi \
        --project-version "${FASTAPI_COMMIT}" \
        --target-only fastapi \
        --output "${SCIP_OUT}" 2>&1 | tail -5
)
ls -la "${SCIP_OUT}"

# ── 4. Bundle ─────────────────────────────────────────────────────────
if ! command -v argosbrain-bundlegen >/dev/null 2>&1; then
    echo "ERROR: argosbrain-bundlegen not on PATH."
    echo "Build it from the neurogenesis repo:"
    echo "  cargo build --release -p neurogenesis-bundle-gen --bin argosbrain-bundlegen"
    echo "Then add target/release/ to PATH or symlink the binary."
    exit 1
fi

argosbrain-bundlegen generate \
    --language python \
    --stdlib-version "${FASTAPI_COMMIT}" \
    --tier 2 \
    --scip "${SCIP_OUT}" \
    --out "${BUNDLE_OUT}"

echo "── corpus ready ──"
ls -la "${BUNDLE_OUT}"
echo "${FASTAPI_COMMIT}" > "${STAMP}"
