#!/usr/bin/env bash
# Fetch + index FsToolkit.ErrorHandling for LongMemCode (F# test corpus).
# Uses tree-sitter-fsharp — scip-dotnet doesn't support F# (Roslyn).
set -euo pipefail
FSTOOLKIT_TAG="5.2.0"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/fstoolkit"
SOURCE_DIR="${WORK}/source"
BUNDLE_OUT="${WORK}/fstoolkit.argosbundle"
STAMP="${WORK}/.commit"

if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${FSTOOLKIT_TAG}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit"; exit 0
fi
rm -rf "${WORK}"; mkdir -p "${WORK}"

git clone --depth=1 --branch="${FSTOOLKIT_TAG}" https://github.com/demystifyfp/FsToolkit.ErrorHandling.git "${SOURCE_DIR}" 2>&1 | tail -3

command -v argosbrain-bundlegen >/dev/null || { echo "ERROR: argosbrain-bundlegen missing"; exit 1; }
argosbrain-bundlegen generate \
    --language fsharp \
    --stdlib-version "${FSTOOLKIT_TAG}" \
    --tier 2 \
    --source "${SOURCE_DIR}/src" \
    --out "${BUNDLE_OUT}"

ls -la "${BUNDLE_OUT}"
echo "${FSTOOLKIT_TAG}" > "${STAMP}"
