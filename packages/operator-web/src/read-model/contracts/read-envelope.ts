export type EnvelopeState = "current" | "stale" | "degraded" | "gap" | "unknown";
export interface ProvenanceRef {readonly source: string; readonly lineage: string}
export interface Uncertainty {readonly description: string}
export interface Limitation {readonly code?: string; readonly description: string}

export interface ReadEnvelope<T> {
  readonly data: T | null;
  readonly sourceTime: string | null;
  readonly receivedTime: string;
  readonly asOfVersion: string | null;
  readonly state: EnvelopeState;
  readonly provenance: readonly ProvenanceRef[];
  readonly uncertainty?: Uncertainty;
  readonly limitations: readonly (Limitation | string)[];
  readonly simulated: boolean;
}

export function adaptEnvelope<T>(envelope: ReadEnvelope<T>): ReadEnvelope<T> {
  return Object.freeze({
    ...envelope,
    provenance: Object.freeze([...envelope.provenance]),
    limitations: Object.freeze([...envelope.limitations]),
  });
}
