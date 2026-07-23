import type {EnvelopeState} from "../contracts/read-envelope";

const MAX_AGE_MS: Readonly<Record<string, number>> = {
  hazard: 60_000,
  fleet: 45_000,
  incident: 120_000,
  mission: 30_000,
  station: 60_000,
  logistics: 300_000,
  safety: 300_000,
  recovery: 300_000,
};
const MAX_FUTURE_SKEW_MS = 30_000;

export function calculateFreshness(context: string, sourceTime: string | null, now: number): EnvelopeState {
  if (!sourceTime) return "unknown";
  const observed = Date.parse(sourceTime);
  if (!Number.isFinite(observed) || observed - now > MAX_FUTURE_SKEW_MS) return "unknown";
  return now - observed <= (MAX_AGE_MS[context] ?? 60_000) ? "current" : "stale";
}
