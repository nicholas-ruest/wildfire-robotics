import type {ActionGateway, ActionReceipt} from "../contracts/workspace-scene";

export interface ActionDraft {
  readonly idempotencyKey: string;
  readonly scope: string;
  readonly constraints: readonly string[];
  readonly expectedVersion: string;
  readonly approval: string;
  readonly payload: Readonly<Record<string, unknown>>;
}

interface ActionSnapshot {
  readonly draft: ActionDraft | null;
  readonly confirmed: boolean;
  readonly receipt: ActionReceipt | null;
}

export class SemanticActionController {
  private draft: ActionDraft | null = null;
  private confirmed = false;
  private receipt: ActionReceipt | null = null;

  constructor(
    private readonly workspaceId: string,
    private readonly gateway: ActionGateway,
  ) {}

  stage(draft: ActionDraft): void {
    validate(draft);
    this.draft = structuredClone(draft);
    this.confirmed = false;
  }

  confirm(): void {
    if (!this.draft) throw new Error("stage an action before confirmation");
    this.confirmed = true;
  }

  async submit(): Promise<ActionReceipt> {
    if (!this.draft) throw new Error("no staged action");
    if (!this.confirmed) throw new Error("semantic HTML confirmation is required");
    const receipt = await this.gateway.dispatch(this.workspaceId, this.draft);
    this.receipt = receipt;
    this.confirmed = false;
    return receipt;
  }

  suspend(): void {
    this.confirmed = false;
  }

  snapshot(): ActionSnapshot {
    return {
      draft: this.draft ? structuredClone(this.draft) : null,
      confirmed: this.confirmed,
      receipt: this.receipt ? {...this.receipt} : null,
    };
  }
}

export class SemanticFallbackWorkflow {
  constructor(private readonly actions: SemanticActionController) {}
  stage(draft: ActionDraft): void { this.actions.stage(draft); }
  inspect(): ActionSnapshot { return this.actions.snapshot(); }
  confirm(): void { this.actions.confirm(); }
  submit(): Promise<ActionReceipt> { return this.actions.submit(); }
}

function validate(draft: ActionDraft): void {
  if (!draft.scope || !draft.expectedVersion || !draft.approval || draft.constraints.length === 0) {
    throw new Error("scope, constraints, expectedVersion, and approval are required");
  }
}
