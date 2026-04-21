#!/usr/bin/env bash
set -euo pipefail
REF="main"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/0"
[ -s "${WORK}/0.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${REF}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${REF}" https://github.com/PowerShell/PSScriptAnalyzer.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language powershell --stdlib-version "${REF}" --tier 2 --source "${WORK}/source" --out "${WORK}/0.argosbundle"
ls -la "${WORK}/0.argosbundle"
echo "${REF}" > "${WORK}/.commit"
