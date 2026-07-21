import assert from "node:assert/strict";
import test from "node:test";
import { decodeBounded } from "./index.js";

test("accepts the exact size boundary", () => {
  const bytes = new Uint8Array([1, 2, 3]);
  assert.equal(decodeBounded(bytes, 3, (value) => value.byteLength), 3);
});

test("rejects input above the size boundary before decoding", () => {
  let called = false;
  assert.throws(() => decodeBounded(new Uint8Array(4), 3, () => { called = true; }));
  assert.equal(called, false);
});
