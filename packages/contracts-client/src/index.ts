/** Maximum default size of a canonical internal envelope. */
export const DEFAULT_MAX_MESSAGE_BYTES = 1_048_576;

/** Rejects oversized wire input before a generated decoder allocates or parses. */
export function decodeBounded<T>(
  bytes: Uint8Array,
  maximumBytes: number,
  decoder: (bytes: Uint8Array) => T,
): T {
  if (!Number.isSafeInteger(maximumBytes) || maximumBytes < 0) {
    throw new RangeError("maximumBytes must be a non-negative safe integer");
  }
  if (bytes.byteLength > maximumBytes) {
    throw new RangeError(`message is ${bytes.byteLength} bytes; maximum is ${maximumBytes}`);
  }
  return decoder(bytes);
}

// Generated exports are written to src/gen by contracts/generate.sh. Keeping
// that output in this package prevents browser applications importing domain code.
export * from "./gen/wildfire/v1/canonical_pb.js";
export * from "./gen/wildfire/v1/registry_events_pb.js";
