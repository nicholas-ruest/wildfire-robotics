#!/usr/bin/env bash
set -euo pipefail

repository_root="$(git rev-parse --show-toplevel)"
destination="$repository_root/target/evidence/sbom"
mkdir -p "$destination"
find "$destination" -maxdepth 1 -type f -name '*.cdx.json' -delete
manifests_file="$destination/manifests.txt"
fixture_sbom="$repository_root/fixtures/persistence-service/wildfire-sbom.json"
fixture_backup="$(mktemp)"
cp "$fixture_sbom" "$fixture_backup"
cleanup() {
  cp "$fixture_backup" "$fixture_sbom"
  rm -f "$fixture_backup" "$manifests_file"
}
trap cleanup EXIT
find "$repository_root/crates" "$repository_root/tools" \
  -mindepth 2 -maxdepth 2 -name Cargo.toml -print \
  | sort > "$manifests_file"

while IFS= read -r manifest; do
  package_dir="$(dirname "$manifest")"
  rm -f "$package_dir/wildfire-sbom.json"
done < "$manifests_file"

cargo cyclonedx \
  --all \
  --format json \
  --spec-version 1.5 \
  --license-accept-named 'MIT/Apache-2.0' \
  --override-filename wildfire-sbom

while IFS= read -r manifest; do
  package_dir="$(dirname "$manifest")"
  relative_dir="${package_dir#"$repository_root"/}"
  artifact_name="${relative_dir//\//--}"
  mv "$package_dir/wildfire-sbom.json" "$destination/$artifact_name.cdx.json"
done < "$manifests_file"

for package in contracts-client api-client operator-web; do
  node "$repository_root/scripts/npm-sbom.mjs" \
    "$repository_root/packages/$package" \
    > "$destination/npm-$package.cdx.json"
done
