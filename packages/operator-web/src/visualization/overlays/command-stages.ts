import type {ActionReceipt, ActionStage} from "../contracts/workspace-scene";

export const ACTION_STAGES: readonly ActionStage[] = [
  "accepted", "rejected", "acknowledged", "executing", "outcome-confirmed",
  "outcome-unknown", "held", "revoked", "failed",
];

const SYMBOLS: Record<ActionStage, string> = {
  accepted: "✓", rejected: "⊘", acknowledged: "↔", executing: "…",
  "outcome-confirmed": "◆", "outcome-unknown": "?", held: "Ⅱ",
  revoked: "↶", failed: "!",
};

export function renderCommandStages(host: HTMLElement, receipts: readonly ActionReceipt[]): void {
  const list = document.createElement("ol");
  list.setAttribute("aria-label", "Command lifecycle");
  receipts.forEach(receipt => {
    const item = document.createElement("li");
    item.dataset.commandStage = receipt.stage;
    item.className = `command-stage command-stage-${receipt.stage}`;
    item.textContent = `${SYMBOLS[receipt.stage]} ${receipt.stage}`;
    list.append(item);
  });
  host.replaceChildren(list);
}
