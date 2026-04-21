#!/usr/bin/env bash
# Fetch + index OpenZeppelin contracts for LongMemCode (Solidity test corpus).
set -euo pipefail
OZ_TAG="v5.1.0"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/openzeppelin"
SOURCE_DIR="${WORK}/source"
BUNDLE_OUT="${WORK}/openzeppelin.argosbundle"
STAMP="${WORK}/.commit"

if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${OZ_TAG}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit"; exit 0
fi
rm -rf "${WORK}"; mkdir -p "${WORK}"

git clone --depth=1 --branch="${OZ_TAG}" https://github.com/OpenZeppelin/openzeppelin-contracts.git "${SOURCE_DIR}" 2>&1 | tail -3

command -v argosbrain-bundlegen >/dev/null || { echo "ERROR: argosbrain-bundlegen missing"; exit 1; }
argosbrain-bundlegen generate \
    --language solidity \
    --stdlib-version "${OZ_TAG}" \
    --tier 2 \
    --source "${SOURCE_DIR}/contracts" \
    --out "${BUNDLE_OUT}"

ls -la "${BUNDLE_OUT}"
echo "${OZ_TAG}" > "${STAMP}"
