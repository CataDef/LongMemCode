#!/usr/bin/env bash
# Fetch + index apache/commons-lang for LongMemCode (Java).
#
# Produces under _work/commons-lang/:
#
#   source/                  shallow clone of apache/commons-lang at the pinned tag
#   commons-lang.scip        SCIP index (javac + semanticdb-javac + scip-java index-semanticdb)
#   commons-lang.argosbundle tier-2 bundle produced by argosbrain-bundlegen
#
# Why commons-lang: 246 source files, Apache 2.0, idiomatic Java
# (builder / utility / tuple patterns, Stream integration, generics).
# Mid-size corpus that matches clap / fastify / trpc in scope so
# scores stay comparable across languages.
#
# Why javac direct rather than `scip-java index`: scip-java's build-
# tool-wrapped path goes through GradleBuildTool / MavenBuildTool,
# both of which (a) crash on macOS in DeleteVisitor.postVisitDirectory
# and (b) for Maven specifically, fail to attach the semanticdb
# plugin when the classes are already compiled. Driving javac + the
# semanticdb-javac plugin by hand is the documented "manual
# configuration" path and is the minimum-surface-area solution.

set -euo pipefail

# ── Reproducibility pins ──────────────────────────────────────────────
COMMONS_LANG_COMMIT="rel/commons-lang-3.14.0"
SCIP_JAVA_VERSION="0.12.3"
SEMANTICDB_JAVAC_VERSION="0.12.3"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/commons-lang"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/commons-lang.scip"
BUNDLE_OUT="${WORK}/commons-lang.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: commons-lang (Java) ──"
echo "pinned tag: ${COMMONS_LANG_COMMIT}"

# ── Cache check ────────────────────────────────────────────────────────
if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${COMMONS_LANG_COMMIT}" ] \
    && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit — bundle already built for ${COMMONS_LANG_COMMIT}"
    ls -la "${BUNDLE_OUT}"
    exit 0
fi
rm -rf "${WORK}"
mkdir -p "${WORK}"

# ── 1. Clone ──────────────────────────────────────────────────────────
echo "cloning apache/commons-lang@${COMMONS_LANG_COMMIT} …"
git clone --depth=1 --branch="${COMMONS_LANG_COMMIT}" \
    https://github.com/apache/commons-lang.git "${SOURCE_DIR}" 2>&1 | tail -3

# ── 2. Toolchain: JDK + Coursier + scip-java + Maven ─────────────────
# JDK 17 matches neurogenesis/scripts/build-bundle-java.sh. Coursier
# lets us pull scip-java without relying on an OS package; Maven is
# only used for resolving commons-lang's dependencies, not for
# running the compile (javac handles that below).
if ! command -v javac >/dev/null 2>&1; then
    echo "ERROR: javac not on PATH — install openjdk-17 or newer."
    exit 1
fi
if ! command -v mvn >/dev/null 2>&1; then
    echo "ERROR: mvn not on PATH — commons-lang needs Maven to resolve deps."
    exit 1
fi
if ! command -v cs >/dev/null 2>&1 && ! command -v coursier >/dev/null 2>&1; then
    echo "ERROR: Coursier not on PATH — install via 'brew install coursier/formulas/coursier'"
    echo "       or fetch from https://github.com/coursier/coursier/releases"
    exit 1
fi
CS=$(command -v cs || command -v coursier)

# scip-java via Coursier's `--contrib` channel.
SCIP_JAVA_BIN="${WORK}/bin/scip-java"
mkdir -p "$(dirname "${SCIP_JAVA_BIN}")"
if [ ! -x "${SCIP_JAVA_BIN}" ]; then
    echo "installing scip-java ${SCIP_JAVA_VERSION} via Coursier (--contrib)…"
    "${CS}" install --contrib --install-dir "$(dirname "${SCIP_JAVA_BIN}")" scip-java 2>&1 | tail -3
fi
echo "scip-java: $("${SCIP_JAVA_BIN}" --version 2>&1 | head -1)"

# semanticdb-javac plugin jar (Maven Central, stable ABI).
SEMDB_JAR="${WORK}/semanticdb-javac.jar"
if [ ! -s "${SEMDB_JAR}" ]; then
    echo "downloading semanticdb-javac ${SEMANTICDB_JAVAC_VERSION} …"
    curl -fsSL \
        "https://repo1.maven.org/maven2/com/sourcegraph/semanticdb-javac/${SEMANTICDB_JAVAC_VERSION}/semanticdb-javac-${SEMANTICDB_JAVAC_VERSION}.jar" \
        -o "${SEMDB_JAR}"
fi

# ── 3. Resolve classpath via Maven (no compile, just deps) ───────────
echo "resolving commons-lang classpath via mvn dependency:build-classpath …"
(
    cd "${SOURCE_DIR}"
    mvn dependency:build-classpath -Dmdep.outputFile=target/cp.txt -q 2>&1 | tail -3
)
DEPS_CP=$(cat "${SOURCE_DIR}/target/cp.txt")

# ── 4. Drive javac + semanticdb-javac plugin directly ────────────────
# cd into src/main/java + use $(pwd -P) so sourceroot matches javac's
# internal file URIs even on macOS (/var → /private/var symlink).
SEMDB_OUT="${WORK}/semdb"
CLASSES_OUT="${WORK}/classes"
mkdir -p "${SEMDB_OUT}" "${CLASSES_OUT}"

SRC_JAVA_ROOT="${SOURCE_DIR}/src/main/java"
SRC_JAVA_REAL=$(cd "${SRC_JAVA_ROOT}" && pwd -P)
echo "compiling commons-lang with -Xplugin:semanticdb …"
(cd "${SRC_JAVA_REAL}" && javac \
    -J-Xmx2g \
    -source 8 -target 8 \
    -proc:none -implicit:none \
    -Xlint:none -nowarn \
    -sourcepath . \
    -d "${CLASSES_OUT}" \
    -classpath "${SEMDB_JAR}:${DEPS_CP}" \
    -Xplugin:"semanticdb -sourceroot:${SRC_JAVA_REAL} -targetroot:${SEMDB_OUT}" \
    $(find . -name '*.java') 2>&1 | tail -10) \
  || echo "::notice::javac exited non-zero — partial semanticdb still emitted"

SEMDB_COUNT=$(find "${SEMDB_OUT}" -name '*.semanticdb' | wc -l)
echo "semanticdb files: ${SEMDB_COUNT}"
if [ "${SEMDB_COUNT}" -eq 0 ]; then
    echo "::error::semanticdb-javac produced zero .semanticdb files"
    exit 1
fi

# ── 5. Convert SemanticDB → SCIP ─────────────────────────────────────
echo "converting SemanticDB → SCIP …"
(cd "${SEMDB_OUT}" && "${SCIP_JAVA_BIN}" index-semanticdb --output "${SCIP_OUT}" . 2>&1 | tail -3)
if [ ! -s "${SCIP_OUT}" ]; then
    echo "::error::scip-java index-semanticdb produced empty index.scip"
    exit 1
fi
ls -la "${SCIP_OUT}"

# ── 6. Bundle ────────────────────────────────────────────────────────
if ! command -v argosbrain-bundlegen >/dev/null 2>&1; then
    echo "ERROR: argosbrain-bundlegen not on PATH."
    echo "Build it from the neurogenesis repo:"
    echo "  cargo build --release -p neurogenesis-bundle-gen --bin argosbrain-bundlegen"
    exit 1
fi

argosbrain-bundlegen generate \
    --language java \
    --stdlib-version "${COMMONS_LANG_COMMIT}" \
    --tier 2 \
    --scip "${SCIP_OUT}" \
    --out "${BUNDLE_OUT}"

echo "── corpus ready ──"
ls -la "${BUNDLE_OUT}"
echo "${COMMONS_LANG_COMMIT}" > "${STAMP}"
