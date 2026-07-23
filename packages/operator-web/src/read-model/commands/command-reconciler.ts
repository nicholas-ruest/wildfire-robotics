export const COMMAND_LIFECYCLE_STAGES = [
  "not-submitted", "submission-unknown", "accepted", "rejected", "acknowledged",
  "executing", "outcome-confirmed", "outcome-unknown", "held", "revoked", "failed",
] as const;
export type ReconciledCommandStage = typeof COMMAND_LIFECYCLE_STAGES[number];
export interface CommandRecord {
  readonly idempotencyKey: string;
  readonly receiptId?: string;
  readonly stage: ReconciledCommandStage;
}

export class CommandReconciler {
  readonly stages = COMMAND_LIFECYCLE_STAGES;
  private readonly records = new Map<string, CommandRecord>();

  constructor(private readonly lookup: (idempotencyKey: string) => Promise<CommandRecord>) {}

  recordTimeout(idempotencyKey: string): void {
    this.records.set(idempotencyKey, {idempotencyKey, stage: "submission-unknown"});
  }

  record(record: CommandRecord): void {
    this.records.set(record.idempotencyKey, record);
  }

  async retry(idempotencyKey: string): Promise<CommandRecord> {
    const existing = this.records.get(idempotencyKey);
    if (existing?.stage !== "submission-unknown") return existing ?? {idempotencyKey, stage: "not-submitted"};
    const reconciled = await this.lookup(idempotencyKey);
    this.record(reconciled);
    return reconciled;
  }
}
