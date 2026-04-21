#!/usr/bin/env bash
set -euo pipefail
REF="master"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/distributions-jl"
[ -s "${WORK}/distributions-jl.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${REF}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${REF}" https://github.com/JuliaStats/Distributions.jl.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language julia --stdlib-version "${REF}" --tier 2 --source "${WORK}/source" --out "${WORK}/distributions-jl.argosbundle"
ls -la "${WORK}/distributions-jl.argosbundle"
echo "${REF}" > "${WORK}/.commit"
