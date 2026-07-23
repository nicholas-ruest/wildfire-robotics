export class StatePartitions {
  readonly authoritative = new Map<string, unknown>();
  readonly view = new Map<string, unknown>();
  readonly drafts = new Map<string, unknown>();
  readonly commands = new Map<string, unknown>();
}
