#!/usr/bin/env bash
set -euo pipefail
REF="v1.16.1"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/plug"
[ -s "${WORK}/plug.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${REF}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${REF}" https://github.com/elixir-plug/plug.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language elixir --stdlib-version "${REF}" --tier 2 --source "${WORK}/source" --out "${WORK}/plug.argosbundle"
ls -la "${WORK}/plug.argosbundle"
echo "${REF}" > "${WORK}/.commit"
