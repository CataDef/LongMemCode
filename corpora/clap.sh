#!/usr/bin/env bash
# Fetch + index clap-rs/clap for LongMemCode.
#
# Produces under _work/clap/:
#
#   source/         shallow clone of clap-rs/clap at the pinned commit
#   clap.scip       raw SCIP from `rust-analyzer scip`
#   clap.argosbundle    tier-2 bundle produced by argosbrain-bundlegen

set -euo pipefail

CLAP_COMMIT="v4.5.20"   # pin to a release; bump to refresh

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/clap"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/clap.scip"
BUNDLE_OUT="${WORK}/clap.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: clap ──"
echo "pinned commit: ${CLAP_COMMIT}"

if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${CLAP_COMMIT}" ] \
    && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit — bundle already built for ${CLAP_COMMIT}"
    ls -la "${BUNDLE_OUT}"
    exit 0
fi
rm -rf "${WORK}"
mkdir -p "${WORK}"

echo "cloning clap-rs/clap@${CLAP_COMMIT} …"
git clone --depth=1 --branch="${CLAP_COMMIT}" \
    https://github.com/clap-rs/clap.git "${SOURCE_DIR}" 2>&1 | tail -3

# `rust-analyzer scip` must be on PATH. Install via `rustup component
# add rust-analyzer` on a stable toolchain. Pin nothing — the index is
# language-grammar stable.
if ! command -v rust-analyzer >/dev/null 2>&1; then
    echo "ERROR: rust-analyzer not on PATH."
    echo "Install: rustup component add rust-analyzer --toolchain stable"
    exit 1
fi
echo "rust-analyzer: $(rust-analyzer --version | head -1)"

echo "indexing clap with rust-analyzer scip…"
(
    cd "${SOURCE_DIR}"
    rust-analyzer scip . 2>&1 | tail -5
)
mv "${SOURCE_DIR}/index.scip" "${SCIP_OUT}"
ls -la "${SCIP_OUT}"

if ! command -v argosbrain-bundlegen >/dev/null 2>&1; then
    echo "ERROR: argosbrain-bundlegen not on PATH (see corpora/fastapi.sh)."
    exit 1
fi

argosbrain-bundlegen generate \
    --language rust \
    --stdlib-version "${CLAP_COMMIT}" \
    --tier 2 \
    --scip "${SCIP_OUT}" \
    --out "${BUNDLE_OUT}"

echo "── corpus ready ──"
ls -la "${BUNDLE_OUT}"
echo "${CLAP_COMMIT}" > "${STAMP}"
