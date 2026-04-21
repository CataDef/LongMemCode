#!/usr/bin/env bash
set -euo pipefail
REF="v1.12.0"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/bats-core"
[ -s "${WORK}/bats-core.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${REF}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${REF}" https://github.com/bats-core/bats-core.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language bash --stdlib-version "${REF}" --tier 2 --source "${WORK}/source" --out "${WORK}/bats-core.argosbundle"
ls -la "${WORK}/bats-core.argosbundle"
echo "${REF}" > "${WORK}/.commit"
