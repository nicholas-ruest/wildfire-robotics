import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const here = new URL("./", import.meta.url);
const load = (name) => JSON.parse(readFileSync(fileURLToPath(new URL(name, here)), "utf8"));
const exercise = load("recovery-exercise.local-simulated.json");
const schema = load("recovery-exercise.schema.json");
const catalog = JSON.parse(readFileSync(fileURLToPath(new URL("../../docs/runbooks/backup-catalog.json", here)), "utf8"));

for (const key of schema.required) assert.ok(Object.hasOwn(exercise, key), `missing exercise field: ${key}`);
assert.equal(exercise.evidence_class, "local_simulated_qualification");
assert.match(exercise.canadian_region, /^ca-/);
assert.ok(exercise.authorized_by.length >= 2 && new Set(exercise.authorized_by).size === exercise.authorized_by.length);
const steps = exercise.restore_steps;
assert.deepEqual(steps.map((step) => step.order), Array.from({ length: steps.length }, (_, i) => i + 1));
const requiredOrder = ["network-dns-time-kms", "identity-pki-revocation", "audit-evidence-object-manifests", "context-databases-migrations", "broker-paused-quarantine", "read-projections", "owning-context-apis", "gateway-read-traffic", "command-traffic-after-current-authority"];
assert.deepEqual(steps.map((step) => step.component), requiredOrder);
for (const step of steps) assert.match(step.evidence_digest, /^[a-f0-9]{64}$/);
assert.equal(exercise.authority_controls.recovery_epoch_incremented, true);
assert.equal(exercise.authority_controls.expired_grants_rejected, true);
assert.equal(exercise.authority_controls.leases_invalidated, true);
assert.equal(exercise.authority_controls.fresh_authority_required, true);
assert.equal(exercise.authority_controls.command_traffic_enabled_after_reconciliation, true);
assert.equal(exercise.message_controls.consumers_restored_paused, true);
assert.equal(exercise.message_controls.facts_replayed_idempotently, true);
assert.equal(exercise.message_controls.commands_quarantined, true);
assert.equal(exercise.message_controls.blind_command_replay, false);
assert.equal(exercise.message_controls.duplicate_physical_effects, 0);
assert.ok(exercise.measured_rpo_seconds <= exercise.rpo_target_seconds, "RPO target missed");
assert.ok(exercise.measured_rto_seconds <= exercise.rto_target_seconds, "RTO target missed");
assert.equal(exercise.result, "pass");
assert.ok(exercise.limitations.some((item) => /simulation only/i.test(item)));

assert.equal(catalog.schema_version, 1);
assert.match(catalog.residency, /Canadian/);
const requiredProducts = new Set(["context-postgresql-postgis", "immutable-object-manifests", "nats-fact-streams", "audit-evidence", "identity-policy-revocation", "gitops-and-infrastructure-state"]);
for (const product of catalog.products) {
  requiredProducts.delete(product.id);
  for (const key of ["owner", "classification", "method", "frequency", "retention", "isolation", "rpo_seconds", "rto_seconds", "restore_order"]) assert.ok(Object.hasOwn(product, key), `${product.id} missing ${key}`);
  assert.ok(product.rto_seconds > 0 && product.rpo_seconds >= 0);
}
assert.equal(requiredProducts.size, 0, `catalog products missing: ${[...requiredProducts]}`);
console.log(`validated ${steps.length} ordered recovery steps and ${catalog.products.length} backup products; authority resurrection and blind replay prohibited`);
