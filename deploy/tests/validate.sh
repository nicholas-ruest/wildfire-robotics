#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
rendered="$(mktemp)"
trap 'rm -f "$rendered"' EXIT
kubectl kustomize "$root/deploy/kubernetes/overlays/production-ca" > "$rendered"
station_rendered="$(kubectl kustomize "$root/deploy/kubernetes/station")"
grep -q '^apiVersion:' "$rendered"
grep -q '^kind:' "$rendered"
grep -q '^apiVersion:' <<<"$station_rendered"
grep -q 'WR_DATA_RESIDENCY: CA' "$rendered"
grep -q '@sha256:' "$rendered"
grep -q 'kind: NetworkPolicy' "$rendered"
grep -q 'automountServiceAccountToken: false' "$rendered"
if grep -REn 'kind: Secret|password:|token:' "$root/deploy/kubernetes"; then
  echo 'plaintext secret material or Secret object is prohibited' >&2
  exit 1
fi
node "$root/deploy/tests/validate-recovery-evidence.mjs"
echo 'deployment manifests validated'
