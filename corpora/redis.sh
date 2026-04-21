#!/usr/bin/env bash
set -euo pipefail
REDIS_TAG="7.4.1"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/redis"
[ -s "${WORK}/redis.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${REDIS_TAG}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${REDIS_TAG}" https://github.com/redis/redis.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language c --stdlib-version "${REDIS_TAG}" --tier 2 --source "${WORK}/source/src" --out "${WORK}/redis.argosbundle"
ls -la "${WORK}/redis.argosbundle"
echo "${REDIS_TAG}" > "${WORK}/.commit"
