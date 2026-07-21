#!/usr/bin/env bash
set -euo pipefail

repository_root="$(git rev-parse --show-toplevel)"
destination="$repository_root/target/evidence/sbom"
mkdir -p "$destination"
manifests_file="$destination/manifests.txt"
find "$repository_root/crates" "$repository_root/tools" -mindepth 2 -maxdepth 2 -name Cargo.toml -print | sort > "$manifests_file"

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
  package_name="$(basename "$package_dir")"
  mv "$package_dir/wildfire-sbom.json" "$destination/$package_name.cdx.json"
done < "$manifests_file"

rm -f "$manifests_file"
