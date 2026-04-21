#!/usr/bin/env bash
set -euo pipefail
LEVELDB_TAG="1.23"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/leveldb"
[ -s "${WORK}/leveldb.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${LEVELDB_TAG}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${LEVELDB_TAG}" https://github.com/google/leveldb.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language cpp --stdlib-version "${LEVELDB_TAG}" --tier 2 --source "${WORK}/source" --out "${WORK}/leveldb.argosbundle"
ls -la "${WORK}/leveldb.argosbundle"
echo "${LEVELDB_TAG}" > "${WORK}/.commit"
