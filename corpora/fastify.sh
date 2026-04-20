#!/usr/bin/env bash
# Fetch + index fastify/fastify for LongMemCode (JavaScript).

set -euo pipefail
FASTIFY_COMMIT="v5.6.1"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/fastify"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/fastify.scip"
BUNDLE_OUT="${WORK}/fastify.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: fastify (JavaScript) ──"
if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${FASTIFY_COMMIT}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit — ${FASTIFY_COMMIT}"; ls -la "${BUNDLE_OUT}"; exit 0
fi
rm -rf "${WORK}"; mkdir -p "${WORK}"

git clone --depth=1 --branch="${FASTIFY_COMMIT}" https://github.com/fastify/fastify.git "${SOURCE_DIR}" 2>&1 | tail -2

( cd "${SOURCE_DIR}" && npm install --ignore-scripts 2>&1 | tail -3 )

# fastify ships .js + .d.ts but no tsconfig.json; synthesise a
# minimal one so scip-typescript can index JS (allowJs: true).
cat > "${SOURCE_DIR}/tsconfig.json" <<'EOF'
{
  "compilerOptions": {
    "allowJs": true, "checkJs": false, "noEmit": true,
    "module": "commonjs", "target": "es2020",
    "moduleResolution": "node", "resolveJsonModule": true,
    "skipLibCheck": true, "strict": false
  },
  "include": ["lib/**/*.js", "fastify.js", "types/*.d.ts"]
}
EOF

if ! command -v scip-typescript >/dev/null 2>&1; then
    echo "installing @sourcegraph/scip-typescript…"
    npm install -g @sourcegraph/scip-typescript 2>&1 | tail -3
fi

( cd "${SOURCE_DIR}" && scip-typescript index 2>&1 | tail -3 )
mv "${SOURCE_DIR}/index.scip" "${SCIP_OUT}"
ls -la "${SCIP_OUT}"

argosbrain-bundlegen generate --language javascript --stdlib-version "${FASTIFY_COMMIT}" --tier 2 --scip "${SCIP_OUT}" --out "${BUNDLE_OUT}"
echo "${FASTIFY_COMMIT}" > "${STAMP}"; ls -la "${BUNDLE_OUT}"
