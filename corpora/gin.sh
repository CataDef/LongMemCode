#!/usr/bin/env bash
# Fetch + index gin-gonic/gin for LongMemCode.

set -euo pipefail
GIN_COMMIT="v1.11.0"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/gin"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/gin.scip"
BUNDLE_OUT="${WORK}/gin.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: gin (Go) ──"
if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${GIN_COMMIT}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit — ${GIN_COMMIT}"; ls -la "${BUNDLE_OUT}"; exit 0
fi
rm -rf "${WORK}"; mkdir -p "${WORK}"

git clone --depth=1 --branch="${GIN_COMMIT}" https://github.com/gin-gonic/gin.git "${SOURCE_DIR}" 2>&1 | tail -2

if ! command -v scip-go >/dev/null 2>&1; then
    echo "installing scip-go…"
    go install github.com/sourcegraph/scip-go/cmd/scip-go@latest
fi

( cd "${SOURCE_DIR}" && scip-go --project-root . --output "${SCIP_OUT}" 2>&1 | tail -3 )
ls -la "${SCIP_OUT}"

argosbrain-bundlegen generate --language go --stdlib-version "${GIN_COMMIT}" --tier 2 --scip "${SCIP_OUT}" --out "${BUNDLE_OUT}"
echo "${GIN_COMMIT}" > "${STAMP}"; ls -la "${BUNDLE_OUT}"
