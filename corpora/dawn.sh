#!/usr/bin/env bash
set -euo pipefail
DAWN_TAG="main"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/dawn"
[ -s "${WORK}/dawn.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${DAWN_TAG}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${DAWN_TAG}" https://github.com/Shopify/dawn.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language liquid --stdlib-version "${DAWN_TAG}" --tier 2 --source "${WORK}/source" --out "${WORK}/dawn.argosbundle"
ls -la "${WORK}/dawn.argosbundle"
echo "${DAWN_TAG}" > "${WORK}/.commit"
