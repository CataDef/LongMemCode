#!/usr/bin/env bash
set -euo pipefail
AF_REF="master"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/alamofire"
[ -s "${WORK}/alamofire.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${AF_REF}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${AF_REF}" https://github.com/Alamofire/Alamofire.git "${WORK}/source" 2>&1 | tail -3
# Swift needs SourceKit-LSP via the Swift toolchain. macOS has it
# pre-installed via Xcode; Linux CI installs via swift.org tar.
argosbrain-bundlegen generate --language swift --stdlib-version "${AF_REF}" --tier 2 --source "${WORK}/source" --out "${WORK}/alamofire.argosbundle"
ls -la "${WORK}/alamofire.argosbundle"
echo "${AF_REF}" > "${WORK}/.commit"
