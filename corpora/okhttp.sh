#!/usr/bin/env bash
set -euo pipefail
OKHTTP_REF="master"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/okhttp"
[ -s "${WORK}/okhttp.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${OKHTTP_REF}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${OKHTTP_REF}" https://github.com/square/okhttp.git "${WORK}/source" 2>&1 | tail -3
# Kotlin needs live LSP — requires kotlin-language-server on PATH.
# Local dev: `brew install kotlin-language-server`.
# CI: see scripts/build-bundle-kotlin.sh in the product repo for
# the server.zip download flow. Local dev with brew-installed
# binary works out of the box when KOTLIN_LSP_BIN isn't set.
argosbrain-bundlegen generate --language kotlin --stdlib-version "${OKHTTP_REF}" --tier 2 --source "${WORK}/source" --out "${WORK}/okhttp.argosbundle"
ls -la "${WORK}/okhttp.argosbundle"
echo "${OKHTTP_REF}" > "${WORK}/.commit"
