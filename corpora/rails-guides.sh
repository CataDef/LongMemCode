#!/usr/bin/env bash
set -euo pipefail
RAILS_REF="main"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/rails-guides"
[ -s "${WORK}/rails-guides.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${RAILS_REF}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
# Sparse checkout — rails/rails is multi-GB, guides/ is ~20MB.
git clone --depth=1 --filter=blob:none --sparse --branch="${RAILS_REF}" https://github.com/rails/rails.git "${WORK}/source" 2>&1 | tail -3
(cd "${WORK}/source" && git sparse-checkout set guides 2>&1 | tail -1)
argosbrain-bundlegen generate --language erb --stdlib-version "${RAILS_REF}" --tier 2 --source "${WORK}/source/guides" --out "${WORK}/rails-guides.argosbundle"
ls -la "${WORK}/rails-guides.argosbundle"
echo "${RAILS_REF}" > "${WORK}/.commit"
