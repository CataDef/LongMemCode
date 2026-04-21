#!/usr/bin/env bash
set -euo pipefail
REF="master"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/afnetworking"
[ -s "${WORK}/afnetworking.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${REF}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${REF}" https://github.com/AFNetworking/AFNetworking.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language objc --stdlib-version "${REF}" --tier 2 --source "${WORK}/source" --out "${WORK}/afnetworking.argosbundle"
ls -la "${WORK}/afnetworking.argosbundle"
echo "${REF}" > "${WORK}/.commit"
