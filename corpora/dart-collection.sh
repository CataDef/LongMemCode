#!/usr/bin/env bash
# Fetch + index dart-lang/collection for LongMemCode (Dart test corpus).
set -euo pipefail

COLLECTION_TAG="v1.19.0"
SCIP_DART_VERSION="1.6.2"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/dart-collection"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/dart-collection.scip"
BUNDLE_OUT="${WORK}/dart-collection.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: dart-collection (${COLLECTION_TAG}) ──"

if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${COLLECTION_TAG}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit"; exit 0
fi
rm -rf "${WORK}"; mkdir -p "${WORK}"

for cmd in dart git; do command -v "$cmd" >/dev/null || { echo "ERROR: $cmd missing"; exit 1; }; done
export PATH="${HOME}/.pub-cache/bin:${PATH}"

dart pub global activate scip_dart "${SCIP_DART_VERSION}" 2>&1 | tail -3

git clone --depth=1 --branch="${COLLECTION_TAG}" https://github.com/dart-lang/collection.git "${SOURCE_DIR}" 2>&1 | tail -3

(cd "${SOURCE_DIR}" && dart pub get 2>&1 | tail -2 && dart pub global run scip_dart ./ 2>&1 | tail -3)
mv "${SOURCE_DIR}/index.scip" "${SCIP_OUT}"

command -v argosbrain-bundlegen >/dev/null || { echo "ERROR: argosbrain-bundlegen missing"; exit 1; }
argosbrain-bundlegen generate \
    --language dart \
    --stdlib-version "${COLLECTION_TAG}" \
    --tier 2 \
    --scip "${SCIP_OUT}" \
    --out "${BUNDLE_OUT}"

ls -la "${BUNDLE_OUT}"
echo "${COLLECTION_TAG}" > "${STAMP}"
