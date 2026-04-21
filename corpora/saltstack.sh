#!/usr/bin/env bash
set -euo pipefail
SALT_TAG="master"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/saltstack"
[ -s "${WORK}/saltstack.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${SALT_TAG}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${SALT_TAG}" https://github.com/saltstack/salt.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language jinja2 --stdlib-version "${SALT_TAG}" --tier 2 --source "${WORK}/source" --out "${WORK}/saltstack.argosbundle"
ls -la "${WORK}/saltstack.argosbundle"
echo "${SALT_TAG}" > "${WORK}/.commit"
