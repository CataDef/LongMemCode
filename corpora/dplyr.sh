#!/usr/bin/env bash
set -euo pipefail
REF="main"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/dplyr"
[ -s "${WORK}/dplyr.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${REF}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${REF}" https://github.com/tidyverse/dplyr.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language r --stdlib-version "${REF}" --tier 2 --source "${WORK}/source" --out "${WORK}/dplyr.argosbundle"
ls -la "${WORK}/dplyr.argosbundle"
echo "${REF}" > "${WORK}/.commit"
