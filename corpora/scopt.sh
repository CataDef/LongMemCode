#!/usr/bin/env bash
# Fetch + index scopt/scopt for LongMemCode (Scala test corpus).
#
# scopt is the canonical Scala command-line parser — small (~14
# .scala files across shared/ + jvm/), self-contained (no scala3
# cross-build dep), widely depended on. Pattern-match with clap
# (Rust) and optparse stuff — a CLI parser, ~400 lines of typed FP.
set -euo pipefail

SCOPT_TAG="v4.1.0"
SCALA_VERSION="2.13.12"
SEMANTICDB_SCALAC_VERSION="4.12.3"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/scopt"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/scopt.scip"
BUNDLE_OUT="${WORK}/scopt.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: scopt (${SCOPT_TAG}) ──"

if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${SCOPT_TAG}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit"; exit 0
fi
rm -rf "${WORK}"; mkdir -p "${WORK}"

for cmd in javac git curl; do command -v "$cmd" >/dev/null || { echo "ERROR: $cmd missing"; exit 1; }; done
CS=$(command -v cs || command -v coursier) || { echo "ERROR: Coursier missing"; exit 1; }

SCALAC_DIR="${WORK}/scalac-bin"
mkdir -p "${SCALAC_DIR}"
"${CS}" install --install-dir "${SCALAC_DIR}" scalac:"${SCALA_VERSION}" 2>&1 | tail -2
export PATH="${SCALAC_DIR}:${PATH}"

SCIP_JAVA_BIN="${WORK}/bin/scip-java"
mkdir -p "$(dirname "${SCIP_JAVA_BIN}")"
"${CS}" install --contrib --install-dir "$(dirname "${SCIP_JAVA_BIN}")" scip-java 2>&1 | tail -2

SEMDB_JAR="${WORK}/semanticdb-scalac.jar"
curl -fsSL "https://repo1.maven.org/maven2/org/scalameta/semanticdb-scalac_${SCALA_VERSION}/${SEMANTICDB_SCALAC_VERSION}/semanticdb-scalac_${SCALA_VERSION}-${SEMANTICDB_SCALAC_VERSION}.jar" -o "${SEMDB_JAR}"

git clone --depth=1 --branch="${SCOPT_TAG}" https://github.com/scopt/scopt.git "${SOURCE_DIR}" 2>&1 | tail -3
cd "${SOURCE_DIR}"
SRC_REAL=$(pwd -P)

SEMDB_OUT="${WORK}/semdb"
CLASS_OUT="${WORK}/classes"
mkdir -p "${SEMDB_OUT}" "${CLASS_OUT}"

# Exclude scala-2.12 and scala-3 to avoid cross-build conflicts
# under scalac 2.13. Left: scala-2 (shared Scala 2), scala-2.13+
# (2.13 and later), shared/ (cross), jvm/ (platform-specific).
scalac \
    -Yrangepos -Xplugin:"${SEMDB_JAR}" \
    -P:semanticdb:sourceroot:"${SRC_REAL}" \
    -P:semanticdb:targetroot:"${SEMDB_OUT}" \
    -P:semanticdb:failures:warning -nowarn \
    -d "${CLASS_OUT}" \
    $(find shared/src/main jvm/src/main -name '*.scala' -not -path '*scala-2.12*' -not -path '*scala-3*' 2>/dev/null)

(cd "${SEMDB_OUT}" && "${SCIP_JAVA_BIN}" index-semanticdb --output "${SCIP_OUT}" . 2>&1 | tail -2)

command -v argosbrain-bundlegen >/dev/null || { echo "ERROR: argosbrain-bundlegen missing"; exit 1; }
argosbrain-bundlegen generate \
    --language scala \
    --stdlib-version "${SCOPT_TAG}" \
    --tier 2 \
    --scip "${SCIP_OUT}" \
    --out "${BUNDLE_OUT}"

ls -la "${BUNDLE_OUT}"
echo "${SCOPT_TAG}" > "${STAMP}"
