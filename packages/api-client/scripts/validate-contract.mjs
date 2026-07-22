import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const contractUrl = new URL("../../../contracts/openapi/wildfire-api-v1.yaml", import.meta.url);
const document = JSON.parse(readFileSync(fileURLToPath(contractUrl), "utf8"));
assert.equal(document.openapi, "3.1.0");
assert.equal(document.info.version, "1.0.0");

function resolve(ref) {
  assert.match(ref, /^#\//, `only local references are allowed: ${ref}`);
  return ref.slice(2).split("/").reduce((value, part) => {
    assert.ok(value !== null && typeof value === "object" && part in value, `unresolved reference: ${ref}`);
    return value[part];
  }, document);
}

function dereference(value) {
  if (value.$ref === undefined) return value;
  const { $ref, ...overrides } = value;
  return { ...dereference(resolve($ref)), ...overrides };
}

function visit(value) {
  if (Array.isArray(value)) return value.forEach(visit);
  if (value === null || typeof value !== "object") return;
  if (typeof value.$ref === "string") resolve(value.$ref);
  Object.values(value).forEach(visit);
}
visit(document);

const methods = new Set(["get", "post", "put", "patch", "delete"]);
const operationIds = new Set();
for (const [path, item] of Object.entries(document.paths)) {
  assert.match(path, /^\/(v1|ogc\/v1)(\/|$)/, `unversioned path: ${path}`);
  for (const [method, raw] of Object.entries(item)) {
    if (!methods.has(method)) continue;
    const operation = dereference(raw);
    assert.equal(typeof operation.operationId, "string", `missing operationId: ${method} ${path}`);
    assert.ok(!operationIds.has(operation.operationId), `duplicate operationId: ${operation.operationId}`);
    operationIds.add(operation.operationId);
    assert.ok(Array.isArray(operation.security) && operation.security.length > 0, `missing security: ${method} ${path}`);
    assert.ok(operation.responses && Object.keys(operation.responses).length > 0, `missing responses: ${method} ${path}`);
  }
}
assert.equal(operationIds.size, 14);
assert.ok(document.components.schemas.Problem.additionalProperties === false);
assert.ok(document.components.schemas.ReadModel.required.includes("freshness"));
assert.ok(document.components.schemas.ReadModel.required.includes("provenance"));
assert.ok(document.components.schemas.ReadModel.required.includes("uncertainty"));
console.log(`validated ${operationIds.size} secured operations and all local schema references`);
