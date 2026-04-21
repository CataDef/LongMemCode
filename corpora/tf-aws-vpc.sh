#!/usr/bin/env bash
set -euo pipefail
REF="master"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/tf-aws-vpc"
[ -s "${WORK}/tf-aws-vpc.argosbundle" ] && [ "$(cat "${WORK}/.commit" 2>/dev/null)" = "${REF}" ] && { echo "cache hit"; exit 0; }
rm -rf "${WORK}"; mkdir -p "${WORK}"
git clone --depth=1 --branch="${REF}" https://github.com/terraform-aws-modules/terraform-aws-vpc.git "${WORK}/source" 2>&1 | tail -3
argosbrain-bundlegen generate --language hcl --stdlib-version "${REF}" --tier 2 --source "${WORK}/source" --out "${WORK}/tf-aws-vpc.argosbundle"
ls -la "${WORK}/tf-aws-vpc.argosbundle"
echo "${REF}" > "${WORK}/.commit"
