#!/usr/bin/env bash
# Fetch + index Ruby stdlib (ruby/ruby/lib at v3_3_0) for LongMemCode.
#
# Same per-subdir / crash-isolated / relative-paths pattern
# documented in neurogenesis/scripts/build-bundle-ruby.sh. yaml/
# subdir is skipped because its two files require C extensions
# that hang scip-ruby's Sorbet resolver.

set -euo pipefail

RUBY_TAG="v3_3_0"
SCIP_RUBY_VERSION="0.4.7"
PER_SUBDIR_TIMEOUT=60

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/ruby-stdlib"
SOURCE_DIR="${WORK}/ruby-src"
BUNDLE_OUT="${WORK}/ruby-stdlib.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: ruby-stdlib (${RUBY_TAG}) ──"

if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${RUBY_TAG}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit"
    exit 0
fi
rm -rf "${WORK}"
mkdir -p "${WORK}"

for cmd in ruby git curl timeout; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        # gtimeout is Coreutils' macOS-friendly alias — accept either.
        if [ "$cmd" = "timeout" ] && command -v gtimeout >/dev/null 2>&1; then
            continue
        fi
        echo "ERROR: $cmd not on PATH."
        exit 1
    fi
done
TIMEOUT_BIN=$(command -v timeout || command -v gtimeout)

# scip-ruby binary
SCIP_RUBY_BIN="${WORK}/scip-ruby"
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
case "${OS}-${ARCH}" in
    linux-x86_64)
        curl -fsSL "https://github.com/sourcegraph/scip-ruby/releases/download/scip-ruby-v${SCIP_RUBY_VERSION}/scip-ruby-x86_64-linux" -o "${SCIP_RUBY_BIN}"
        chmod +x "${SCIP_RUBY_BIN}"
        ;;
    *)
        # Fall back to whichever scip-ruby is on PATH — darwin dev
        # boxes typically gem-install the native gem.
        if command -v scip-ruby >/dev/null 2>&1; then
            ln -sf "$(command -v scip-ruby)" "${SCIP_RUBY_BIN}"
        else
            # Look for the gem's native binary (macOS gem install path).
            NATIVE=$(find "$HOME/.local/share/gem" "$HOME/.gem" /usr/local/lib/ruby -name "scip-ruby" -path "*native*" 2>/dev/null | head -1)
            if [ -n "${NATIVE}" ] && [ -x "${NATIVE}" ]; then
                ln -sf "${NATIVE}" "${SCIP_RUBY_BIN}"
            else
                echo "ERROR: no scip-ruby for ${OS}-${ARCH}. gem install scip-ruby from the release page."
                exit 1
            fi
        fi
        ;;
esac
"${SCIP_RUBY_BIN}" --version | head -1

# Clone ruby
git clone --depth=1 --branch="${RUBY_TAG}" https://github.com/ruby/ruby.git "${SOURCE_DIR}" 2>&1 | tail -3
LIB_ROOT=$(cd "${SOURCE_DIR}/lib" && pwd -P)

SCIPS="${WORK}/scips"
mkdir -p "${SCIPS}"
cd "${LIB_ROOT}"

for sub_dir in */; do
    sub="${sub_dir%/}"
    [ "$sub" = "yaml" ] && continue  # known-hang
    "${TIMEOUT_BIN}" "${PER_SUBDIR_TIMEOUT}" "${SCIP_RUBY_BIN}" \
        --index-file "${SCIPS}/${sub}.scip" \
        --gem-metadata "ruby-stdlib@3.3.0" \
        --quiet "${sub}" 2>/dev/null || true
    if [ -s "${SCIPS}/${sub}.scip" ]; then
        echo "  ✓ ${sub}"
    else
        rm -f "${SCIPS}/${sub}.scip"
    fi
done

# top-level
TOP_DIR="${WORK}/toplevel"
mkdir -p "${TOP_DIR}"
cp "${LIB_ROOT}"/*.rb "${TOP_DIR}/" 2>/dev/null || true
(cd "${WORK}" && "${TIMEOUT_BIN}" "${PER_SUBDIR_TIMEOUT}" "${SCIP_RUBY_BIN}" \
    --index-file "${SCIPS}/_toplevel.scip" \
    --gem-metadata "ruby-stdlib@3.3.0" \
    --quiet "toplevel" 2>&1 | tail -3 || true)

# Concat
MERGED="${WORK}/merged.scip"
cat "${SCIPS}"/*.scip > "${MERGED}"

if ! command -v argosbrain-bundlegen >/dev/null 2>&1; then
    echo "ERROR: argosbrain-bundlegen not on PATH."
    exit 1
fi

argosbrain-bundlegen generate \
    --language ruby \
    --stdlib-version "3.3.0" \
    --tier 2 \
    --scip "${MERGED}" \
    --out "${BUNDLE_OUT}"

echo "── corpus ready ──"
ls -la "${BUNDLE_OUT}"
echo "${RUBY_TAG}" > "${STAMP}"
