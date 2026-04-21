#!/usr/bin/env bash
# Fetch + index PHP stdlib (phpstorm-stubs) for LongMemCode.
#
# Produces under _work/php-stdlib/:
#
#   scaffold/                empty composer project pulling scip-php + stubs
#   scaffold/src/            phpstorm-stubs copied here (first-party for indexer)
#   php-stdlib.scip          SCIP index
#   php-stdlib.argosbundle   tier-2 bundle from argosbrain-bundlegen
#
# "PHP stdlib" = JetBrains/phpstorm-stubs: Apache-2.0 PHP-language
# re-declaration of every Zend built-in + 69 extensions + 6 SAPIs.
# Standard PhpStorm/PHPStan/Psalm input. Indexing php-src directly
# would surface C, not PHP semantics.
#
# Uses CataDef/scip-php fork pinned at catadef-v0.1.1 — the
# Packagist release is abandoned (v0.0.2, April 2023, stuck on
# phpstorm-stubs ^2022.3 + vulnerable protobuf). Our fork tracks
# upstream `main` + adds one patch that widens file discovery to
# include function-only stubs like Core.php (holds all built-in
# PHP functions).

set -euo pipefail

PHPSTORM_STUBS_TAG="v2025.3"
CATADEF_SCIP_PHP_SHA="af28c6d4442a5d36770f48639c279c24c183cfbe"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/php-stdlib"
SCAFFOLD="${WORK}/scaffold"
STUBS_CLONE="${WORK}/stubs-clone"
SCIP_OUT="${WORK}/php-stdlib.scip"
BUNDLE_OUT="${WORK}/php-stdlib.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: php-stdlib (phpstorm-stubs ${PHPSTORM_STUBS_TAG}) ──"

if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${PHPSTORM_STUBS_TAG}" ] \
    && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit — bundle already built for ${PHPSTORM_STUBS_TAG}"
    ls -la "${BUNDLE_OUT}"
    exit 0
fi
rm -rf "${WORK}"
mkdir -p "${WORK}"

# ── Prerequisites ─────────────────────────────────────────────────────
for cmd in php composer git jq curl; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "ERROR: $cmd not on PATH."
        exit 1
    fi
done
php --version | head -1
composer --version | head -1

# ── Scaffold composer project ─────────────────────────────────────────
mkdir -p "${SCAFFOLD}/src"
cat > "${SCAFFOLD}/composer.json" <<EOF
{
  "name": "catadef/php-stdlib-scaffold",
  "type": "project",
  "license": "MIT",
  "minimum-stability": "dev",
  "prefer-stable": true,
  "repositories": [
    { "type": "vcs", "url": "https://github.com/CataDef/scip-php.git" }
  ],
  "require": {
    "davidrjenni/scip-php": "dev-main#${CATADEF_SCIP_PHP_SHA}",
    "jetbrains/phpstorm-stubs": "^2025.3"
  },
  "autoload": { "classmap": ["src/"] },
  "config": {
    "optimize-autoloader": true,
    "audit": { "abandoned": "ignore" }
  }
}
EOF

(cd "${SCAFFOLD}" && composer install --no-interaction --no-progress --prefer-dist 2>&1 | tail -5)

# Copy stubs into scaffold/src/ so scip-php treats them as project code.
cp -R "${SCAFFOLD}/vendor/jetbrains/phpstorm-stubs/"* "${SCAFFOLD}/src/"
(cd "${SCAFFOLD}" && composer dump-autoload --optimize --no-interaction 2>&1 | tail -3)

# ── Run scip-php ─────────────────────────────────────────────────────
echo "── running scip-php ──"
(cd "${SCAFFOLD}" && vendor/bin/scip-php 2>&1 | tail -5)

mv "${SCAFFOLD}/index.scip" "${SCIP_OUT}"
ls -la "${SCIP_OUT}"

# ── Bundle ────────────────────────────────────────────────────────────
if ! command -v argosbrain-bundlegen >/dev/null 2>&1; then
    echo "ERROR: argosbrain-bundlegen not on PATH."
    exit 1
fi

argosbrain-bundlegen generate \
    --language php \
    --stdlib-version "${PHPSTORM_STUBS_TAG}" \
    --tier 2 \
    --scip "${SCIP_OUT}" \
    --out "${BUNDLE_OUT}"

echo "── corpus ready ──"
ls -la "${BUNDLE_OUT}"
echo "${PHPSTORM_STUBS_TAG}" > "${STAMP}"
