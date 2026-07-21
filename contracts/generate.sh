#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"

mode="${1:-generate}"
if [[ "$mode" != "generate" && "$mode" != "--check" ]]; then
  echo "usage: contracts/generate.sh [generate|--check]" >&2
  exit 2
fi

npx --yes @bufbuild/buf@1.58.0 lint contracts/proto
npx --yes @bufbuild/buf@1.58.0 build contracts/proto
npx --yes @bufbuild/buf@1.58.0 breaking contracts/proto \
  --against contracts/baseline/wildfire-v1.binpb
npm --prefix packages/contracts-client ci
npx --yes @bufbuild/buf@1.58.0 generate contracts/proto --template packages/contracts-client/buf.gen.yaml
cargo run -p contract-check -- --update
cargo test -p wildfire-contracts-generated
npm --prefix packages/contracts-client test

if [[ "$mode" == "--check" ]]; then
  git diff --exit-code -- packages/contracts-client/src/gen contracts/examples contracts/fixtures
fi
