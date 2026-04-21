#!/usr/bin/env bash
# Fetch + index symfony/console for LongMemCode (PHP test corpus).
#
# Symfony Console is the canonical PHP CLI component — used by
# Laravel Artisan, PHPUnit, Composer itself, Magento, Drush.
# ~254 .php files, Apache-like ecosystem scale. Pattern-match
# with fastify (JS) / gin (Go) / tRPC (TS): real-world project
# distinct from the stdlib bundle.
#
# Uses CataDef/scip-php fork pinned at catadef-v0.1.3 (with
# psr-4-empty-path + vendor-skip patches); indexes symfony as
# its own root project (psr-4 path is the repo root).
set -euo pipefail

SYMFONY_TAG="v7.1.5"
CATADEF_SCIP_PHP_SHA="dbf4d193646b2cd9bcd560ebf818b00e2e87030c"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${HERE}/_work/symfony-console"
SOURCE_DIR="${WORK}/source"
SCIP_OUT="${WORK}/symfony-console.scip"
BUNDLE_OUT="${WORK}/symfony-console.argosbundle"
STAMP="${WORK}/.commit"

echo "── LongMemCode corpus: symfony/console (${SYMFONY_TAG}) ──"

if [ -f "${STAMP}" ] && [ "$(cat "${STAMP}")" = "${SYMFONY_TAG}" ] && [ -s "${BUNDLE_OUT}" ]; then
    echo "cache hit"; exit 0
fi
rm -rf "${WORK}"; mkdir -p "${WORK}"

for cmd in php composer git jq curl; do command -v "$cmd" >/dev/null || { echo "ERROR: $cmd missing"; exit 1; }; done

git clone --depth=1 --branch="${SYMFONY_TAG}" https://github.com/symfony/console.git "${SOURCE_DIR}" 2>&1 | tail -3
cd "${SOURCE_DIR}"

# Inject CataDef scip-php fork as a dev-dep of the symfony project.
jq '. + {
  "minimum-stability": "dev",
  "repositories": [{"type": "vcs", "url": "https://github.com/CataDef/scip-php.git"}],
  "require-dev": ((.["require-dev"] // {}) + {"davidrjenni/scip-php": "dev-main#'"${CATADEF_SCIP_PHP_SHA}"'"}),
  "config": ((.config // {}) + {"audit": {"abandoned": "ignore"}})
}' composer.json > composer.new.json
mv composer.new.json composer.json

composer install --no-interaction --no-progress --prefer-dist 2>&1 | tail -5
composer dump-autoload --optimize --no-interaction 2>&1 | tail -2

vendor/bin/scip-php 2>&1 | tail -3
if [ ! -s index.scip ]; then
    echo "::error::scip-php produced no index"; exit 1
fi
mv index.scip "${SCIP_OUT}"

command -v argosbrain-bundlegen >/dev/null || { echo "ERROR: argosbrain-bundlegen missing"; exit 1; }
argosbrain-bundlegen generate \
    --language php \
    --stdlib-version "${SYMFONY_TAG}" \
    --tier 2 \
    --scip "${SCIP_OUT}" \
    --out "${BUNDLE_OUT}"

ls -la "${BUNDLE_OUT}"
echo "${SYMFONY_TAG}" > "${STAMP}"
