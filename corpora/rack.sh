#!/usr/bin/env bash
# Fetch + index rack/rack for LongMemCode (Ruby test corpus).
#
# Rack is Ruby's HTTP interface — every Ruby web framework
# (Rails, Sinatra, Roda, Hanami) depends on it. ~50 .rb files,
# mid-size, foundational. Pattern-match with fastify (JS) and
# gin (Go) — real-world project distinct from the stdlib.
#
# Uses Sourcegraph's scip-ruby v0.4.7 directly on rack/lib/ (no
# per-subdir dance — rack has no C-extension-requiring files
# that hang the Sorbet resolver, unlike the full Ruby stdlib).

set -euo pipefail

RACK_TAG="v3.1.8"
SCIP_RUBY_VERSION="0.4.7"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/rack"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/rack.scip"
BUNDLE_OUT="${WORK}/rack.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: rack (${RACK_TAG}) ──"

if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${RACK_TAG}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit"
    exit 0
fi
rm -rf "${WORK}"
mkdir -p "${WORK}"

for cmd in ruby git curl; do
    command -v "$cmd" >/dev/null 2>&1 || { echo "ERROR: $cmd not on PATH"; exit 1; }
done

SCIP_RUBY_BIN="${WORK}/scip-ruby"
case "$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)" in
    linux-x86_64)
        curl -fsSL "https://github.com/sourcegraph/scip-ruby/releases/download/scip-ruby-v${SCIP_RUBY_VERSION}/scip-ruby-x86_64-linux" -o "${SCIP_RUBY_BIN}"
        chmod +x "${SCIP_RUBY_BIN}"
        ;;
    *)
        if command -v scip-ruby >/dev/null 2>&1; then
            ln -sf "$(command -v scip-ruby)" "${SCIP_RUBY_BIN}"
        else
            NATIVE=$(find "$HOME/.local/share/gem" "$HOME/.gem" -name "scip-ruby" -path "*native*" 2>/dev/null | head -1)
            [ -x "${NATIVE}" ] && ln -sf "${NATIVE}" "${SCIP_RUBY_BIN}" || { echo "ERROR: no scip-ruby"; exit 1; }
        fi
        ;;
esac

git clone --depth=1 --branch="${RACK_TAG}" https://github.com/rack/rack.git "${SOURCE_DIR}" 2>&1 | tail -3
LIB_ROOT=$(cd "${SOURCE_DIR}/lib" && pwd -P)

(cd "${LIB_ROOT}" && "${SCIP_RUBY_BIN}" \
    --index-file "${SCIP_OUT}" \
    --gem-metadata "rack@${RACK_TAG#v}" \
    --quiet "rack" 2>&1 | tail -5)

if ! command -v argosbrain-bundlegen >/dev/null 2>&1; then
    echo "ERROR: argosbrain-bundlegen not on PATH"; exit 1
fi
argosbrain-bundlegen generate \
    --language ruby \
    --stdlib-version "${RACK_TAG}" \
    --tier 2 \
    --scip "${SCIP_OUT}" \
    --out "${BUNDLE_OUT}"

echo "── corpus ready ──"
ls -la "${BUNDLE_OUT}"
echo "${RACK_TAG}" > "${STAMP}"
