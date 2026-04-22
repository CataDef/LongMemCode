#!/usr/bin/env bash
# Fetch + index kubernetes/kubernetes for LongMemCode.
#
# This is the scale-ceiling stress test for LongMemCode v0.2. Kubernetes
# is ~2M LOC Go across ~5000 files and produces on the order of several
# hundred thousand symbols after SCIP indexing — right at the limit that
# Papers 1 and 3 declared as "not yet benchmarked".
#
# Expected timings (on laptop-class M-series Apple silicon, 32 GB RAM):
#   git clone --depth=1   :   60–120 s   (~1.2 GB on disk)
#   scip-go index         :  10–25 min   (heavy Go module download + type-check pass)
#   scip-go output size   :   400–800 MB
#   bundlegen → argosbundle:   1–3 min   (peak RSS ~2–4 GB)
#   argosbundle size      :   200–500 MB
#
# If scip-go OOMs, re-run with GOGC=50 or bump machine RAM.

set -euo pipefail
K8S_COMMIT="v1.32.0"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/kubernetes"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/kubernetes.scip"
BUNDLE_OUT="${WORK}/kubernetes.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: kubernetes (Go, SCALE TEST) ──"
if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${K8S_COMMIT}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit — ${K8S_COMMIT}"; ls -la "${BUNDLE_OUT}"; exit 0
fi
rm -rf "${WORK}"; mkdir -p "${WORK}"

echo "» git clone --depth=1 --branch=${K8S_COMMIT} kubernetes/kubernetes …"
time git clone --depth=1 --branch="${K8S_COMMIT}" https://github.com/kubernetes/kubernetes.git "${SOURCE_DIR}" 2>&1 | tail -3
du -sh "${SOURCE_DIR}" | tee "${WORK}/source-size.txt"

if ! command -v scip-go >/dev/null 2>&1; then
    echo "installing scip-go…"
    go install github.com/sourcegraph/scip-go/cmd/scip-go@latest
fi

echo "» scip-go index --module-root . --output ${SCIP_OUT} …"
echo "» (this takes 10–25 min on first run; Go module resolution dominates)"
time ( cd "${SOURCE_DIR}" && scip-go index --module-root . --output "${SCIP_OUT}" --skip-tests 2>&1 | tail -10 )
ls -la "${SCIP_OUT}"
du -sh "${SCIP_OUT}" | tee "${WORK}/scip-size.txt"

echo "» argosbrain-bundlegen generate …"
time argosbrain-bundlegen generate \
    --language go \
    --stdlib-version "${K8S_COMMIT}" \
    --tier 2 \
    --scip "${SCIP_OUT}" \
    --out "${BUNDLE_OUT}"

ls -la "${BUNDLE_OUT}"
du -sh "${BUNDLE_OUT}" | tee "${WORK}/bundle-size.txt"
echo "${K8S_COMMIT}" > "${STAMP}"

echo "── done. Summary ──"
echo "  commit  : ${K8S_COMMIT}"
echo "  source  : $(cat ${WORK}/source-size.txt 2>/dev/null || echo '?')"
echo "  scip    : $(cat ${WORK}/scip-size.txt 2>/dev/null || echo '?')"
echo "  bundle  : $(cat ${WORK}/bundle-size.txt 2>/dev/null || echo '?')"
