#!/usr/bin/env bash
# Fetch + index jbogard/MediatR for LongMemCode (C# test corpus).
#
# MediatR is the canonical mediator library for .NET — used in
# every Clean Architecture / CQRS template on the market.
# ~40 .cs files, netstandard2.0 + net6.0, zero custom MSBuild.
# Indexes cleanly through scip-dotnet v0.2.13 on macOS + Linux
# (AutoMapper / Serilog / Dapper all failed with different
# build-system quirks; MediatR is the known-working pick).
set -euo pipefail

MEDIATR_TAG="v12.4.1"
DOTNET_CHANNEL="10.0"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/mediatr"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/mediatr.scip"
BUNDLE_OUT="${WORK}/mediatr.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: mediatr (${MEDIATR_TAG}) ──"

if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${MEDIATR_TAG}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit"; exit 0
fi
rm -rf "${WORK}"; mkdir -p "${WORK}"

export DOTNET_ROOT="${HOME}/.dotnet"
export PATH="${DOTNET_ROOT}:${DOTNET_ROOT}/tools:${PATH}"
if [ ! -x "${DOTNET_ROOT}/dotnet" ]; then
    curl -fsSL https://dot.net/v1/dotnet-install.sh -o /tmp/dotnet-install.sh
    chmod +x /tmp/dotnet-install.sh
    /tmp/dotnet-install.sh --channel "${DOTNET_CHANNEL}" --install-dir "${DOTNET_ROOT}" 2>&1 | tail -2
fi
dotnet --version

if ! command -v scip-dotnet >/dev/null 2>&1; then
    dotnet tool install --global scip-dotnet 2>&1 | tail -2
fi

git clone --depth=1 --branch="${MEDIATR_TAG}" https://github.com/jbogard/MediatR.git "${SOURCE_DIR}" 2>&1 | tail -3

(cd "${SOURCE_DIR}" && scip-dotnet index \
    --output "${SCIP_OUT}" \
    src/MediatR/MediatR.csproj 2>&1 | tail -3)

command -v argosbrain-bundlegen >/dev/null || { echo "ERROR: argosbrain-bundlegen missing"; exit 1; }
argosbrain-bundlegen generate \
    --language csharp \
    --stdlib-version "${MEDIATR_TAG}" \
    --tier 2 \
    --scip "${SCIP_OUT}" \
    --out "${BUNDLE_OUT}"

ls -la "${BUNDLE_OUT}"
echo "${MEDIATR_TAG}" > "${STAMP}"
