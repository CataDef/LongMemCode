#!/usr/bin/env bash
set -euo pipefail
REF="v3.1.0"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/luasocket"
[ -s "${WORK}/luasocket.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${REF}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${REF}" https://github.com/lunarmodules/luasocket.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language lua --stdlib-version "${REF}" --tier 2 --source "${WORK}/source" --out "${WORK}/luasocket.argosbundle"
ls -la "${WORK}/luasocket.argosbundle"
echo "${REF}" > "${WORK}/.commit"
