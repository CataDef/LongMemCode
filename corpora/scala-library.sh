#!/usr/bin/env bash
# Fetch + index the Scala 2.13 standard library for LongMemCode.
#
# Produces under _work/scala-library/:
#
#   source/                 extracted scala-library-X-sources.jar contents
#   scala-library.scip      SCIP index (scalac + semanticdb-scalac + scip-java index-semanticdb)
#   scala-library.argosbundle
#
# Why scala-library 2.13 rather than scala3-library: 2.13 is where
# the majority of real-world Scala code lives (Akka, Spark, Play,
# Finagle), it's binary-compatible with itself, and the stdlib
# source is a single jar on Maven Central. Scala 3 migration is a
# separate corpus when we need to score it.
#
# Why this drives scalac directly rather than sbt / Mill: sbt boot
# on a fresh runner takes 2-3 minutes before compiling anything.
# scalac + semanticdb-scalac plugin is the same pattern scip-java
# uses under the hood for sbt builds — we just skip the build tool.

set -euo pipefail

SCALA_VERSION="2.13.12"
SEMANTICDB_SCALAC_VERSION="4.12.3"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/scala-library"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/scala-library.scip"
BUNDLE_OUT="${WORK}/scala-library.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: scala-library (Scala ${SCALA_VERSION}) ──"

# ── Cache check ────────────────────────────────────────────────────────
if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${SCALA_VERSION}" ] \
    && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit — bundle already built for scala ${SCALA_VERSION}"
    ls -la "${BUNDLE_OUT}"
    exit 0
fi
rm -rf "${WORK}"
mkdir -p "${WORK}"

# ── 1. Toolchain ──────────────────────────────────────────────────────
if ! command -v javac >/dev/null 2>&1; then
    echo "ERROR: javac not on PATH — install openjdk-17 or newer."
    exit 1
fi
if ! command -v cs >/dev/null 2>&1 && ! command -v coursier >/dev/null 2>&1; then
    echo "ERROR: Coursier not on PATH — install via 'brew install coursier/formulas/coursier' or fetch from https://github.com/coursier/coursier/releases"
    exit 1
fi
CS=$(command -v cs || command -v coursier)

SCALAC_DIR="${WORK}/scalac-bin"
mkdir -p "${SCALAC_DIR}"
if [ ! -x "${SCALAC_DIR}/scalac" ]; then
    "${CS}" install --install-dir "${SCALAC_DIR}" scalac:"${SCALA_VERSION}" 2>&1 | tail -3
fi
export PATH="${SCALAC_DIR}:${PATH}"
scalac -version 2>&1 | head -1

SCIP_JAVA_BIN="${WORK}/bin/scip-java"
mkdir -p "$(dirname "${SCIP_JAVA_BIN}")"
if [ ! -x "${SCIP_JAVA_BIN}" ]; then
    "${CS}" install --contrib --install-dir "$(dirname "${SCIP_JAVA_BIN}")" scip-java 2>&1 | tail -3
fi

# semanticdb-scalac plugin jar
SEMDB_JAR="${WORK}/semanticdb-scalac.jar"
curl -fsSL \
    "https://repo1.maven.org/maven2/org/scalameta/semanticdb-scalac_${SCALA_VERSION}/${SEMANTICDB_SCALAC_VERSION}/semanticdb-scalac_${SCALA_VERSION}-${SEMANTICDB_SCALAC_VERSION}.jar" \
    -o "${SEMDB_JAR}"

# ── 2. Fetch scala-library sources ───────────────────────────────────
SOURCES_JAR="${WORK}/scala-library-sources.jar"
curl -fsSL \
    "https://repo1.maven.org/maven2/org/scala-lang/scala-library/${SCALA_VERSION}/scala-library-${SCALA_VERSION}-sources.jar" \
    -o "${SOURCES_JAR}"

mkdir -p "${SOURCE_DIR}"
unzip -q "${SOURCES_JAR}" -d "${SOURCE_DIR}"
SCALA_COUNT=$(find "${SOURCE_DIR}" -name '*.scala' | wc -l)
echo "extracted ${SCALA_COUNT} .scala files"

# ── 3. Compile with -Xplugin:semanticdb-scalac ───────────────────────
SEMDB_OUT="${WORK}/semdb"
CLASS_OUT="${WORK}/classes"
mkdir -p "${SEMDB_OUT}" "${CLASS_OUT}"

SRC_DIR_REAL=$(cd "${SOURCE_DIR}" && pwd -P)
echo "── running scalac with -Xplugin:semanticdb-scalac ──"
(cd "${SRC_DIR_REAL}" && scalac \
    -J-Xmx4g \
    -Yrangepos \
    -Xplugin:"${SEMDB_JAR}" \
    -P:semanticdb:sourceroot:"${SRC_DIR_REAL}" \
    -P:semanticdb:targetroot:"${SEMDB_OUT}" \
    -P:semanticdb:failures:warning \
    -nowarn \
    -d "${CLASS_OUT}" \
    $(find . -name '*.scala') 2>&1 | tail -10) \
  || echo "::notice::scalac exited non-zero — semanticdb still emitted"

SEMDB_COUNT=$(find "${SEMDB_OUT}" -name '*.semanticdb' | wc -l)
echo "semanticdb files: ${SEMDB_COUNT}"
if [ "${SEMDB_COUNT}" -eq 0 ]; then
    echo "::error::semanticdb-scalac produced zero .semanticdb files"
    exit 1
fi

# ── 4. SemanticDB → SCIP ─────────────────────────────────────────────
(cd "${SEMDB_OUT}" && "${SCIP_JAVA_BIN}" index-semanticdb --output "${SCIP_OUT}" . 2>&1 | tail -3)
ls -la "${SCIP_OUT}"

# ── 5. Bundle ────────────────────────────────────────────────────────
if ! command -v argosbrain-bundlegen >/dev/null 2>&1; then
    echo "ERROR: argosbrain-bundlegen not on PATH."
    echo "Build from the neurogenesis repo:"
    echo "  cargo build --release -p neurogenesis-bundle-gen --bin argosbrain-bundlegen"
    exit 1
fi

argosbrain-bundlegen generate \
    --language scala \
    --stdlib-version "${SCALA_VERSION}" \
    --tier 2 \
    --scip "${SCIP_OUT}" \
    --out "${BUNDLE_OUT}"

echo "── corpus ready ──"
ls -la "${BUNDLE_OUT}"
echo "${SCALA_VERSION}" > "${STAMP}"
