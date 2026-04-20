#!/usr/bin/env bash
# Fetch + index trpc/trpc server package for LongMemCode.

set -euo pipefail
TRPC_COMMIT="main"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/trpc"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/trpc.scip"
BUNDLE_OUT="${WORK}/trpc.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: tRPC (TypeScript) ──"
if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${TRPC_COMMIT}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit — ${TRPC_COMMIT}"; ls -la "${BUNDLE_OUT}"; exit 0
fi
rm -rf "${WORK}"; mkdir -p "${WORK}"

git clone --depth=1 --branch="${TRPC_COMMIT}" https://github.com/trpc/trpc.git "${SOURCE_DIR}" 2>&1 | tail -2

# Install workspace deps (tRPC uses pnpm). Fallback to npm if pnpm
# isn't on PATH.
if command -v pnpm >/dev/null 2>&1; then
    ( cd "${SOURCE_DIR}" && pnpm install --filter @trpc/server --ignore-scripts 2>&1 | tail -3 )
else
    ( cd "${SOURCE_DIR}" && npm install --ignore-scripts 2>&1 | tail -3 )
fi

if ! command -v scip-typescript >/dev/null 2>&1; then
    echo "installing @sourcegraph/scip-typescript…"
    npm install -g @sourcegraph/scip-typescript 2>&1 | tail -3
fi

( cd "${SOURCE_DIR}/packages/server" && scip-typescript index 2>&1 | tail -3 )
mv "${SOURCE_DIR}/packages/server/index.scip" "${SCIP_OUT}"
ls -la "${SCIP_OUT}"

argosbrain-bundlegen generate --language typescript --stdlib-version "${TRPC_COMMIT}" --tier 2 --scip "${SCIP_OUT}" --out "${BUNDLE_OUT}"
echo "${TRPC_COMMIT}" > "${STAMP}"; ls -la "${BUNDLE_OUT}"
