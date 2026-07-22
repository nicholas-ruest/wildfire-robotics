#!/usr/bin/env bash
set -euo pipefail

repository_root="$(git rev-parse --show-toplevel)"

for package in contracts-client api-client operator-web; do
  npm --prefix "$repository_root/packages/$package" ci --ignore-scripts
  npm --prefix "$repository_root/packages/$package" test
done

# Tests type-check the UI; this separately proves its optimized browser bundle.
npm --prefix "$repository_root/packages/operator-web" run build
