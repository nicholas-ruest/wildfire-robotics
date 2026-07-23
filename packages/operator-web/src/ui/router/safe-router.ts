import {WORKSPACE_GROUPS, type UiWorkspaceId} from "../navigation/workspaces";

const IDS: ReadonlySet<string> = new Set(WORKSPACE_GROUPS.flatMap(group => group.workspaces.map(([id]) => id)));

export interface SafeRoute {
  readonly workspace: UiWorkspaceId;
  readonly view?: string;
  readonly filters?: readonly string[];
  readonly timeRange?: string;
  readonly selection?: string;
}

export type UnsafeRouteInput = SafeRoute & {
  readonly token?: string;
  readonly confirmation?: boolean;
  readonly draft?: unknown;
};

export class OperatorRouteCodec {
  serialize(route: UnsafeRouteInput): string {
    const query = new URLSearchParams({workspace: route.workspace});
    if (route.view) query.set("view", route.view);
    route.filters?.forEach(filter => query.append("filter", filter));
    if (route.timeRange) query.set("time", route.timeRange);
    if (route.selection) query.set("selection", route.selection);
    return `/operator?${query}`;
  }

  parse(value: string): SafeRoute {
    const url = new URL(value, "https://operator.invalid");
    const candidate = url.searchParams.get("workspace") ?? "incident";
    const workspace = (IDS.has(candidate) ? candidate : "incident") as UiWorkspaceId;
    const view = clean(url.searchParams.get("view"));
    const filters = url.searchParams.getAll("filter").map(item => item.slice(0, 100));
    const timeRange = clean(url.searchParams.get("time"));
    const selection = clean(url.searchParams.get("selection"));
    return {
      workspace,
      ...(view ? {view} : {}),
      ...(filters.length ? {filters} : {}),
      ...(timeRange ? {timeRange} : {}),
      ...(selection ? {selection} : {}),
    };
  }
}

export class SafeRouter {
  private readonly listeners = new Set<(route: SafeRoute) => void>();
  private readonly onPopState = () => {
    const route = this.codec.parse(this.target.location.href);
    this.listeners.forEach(listener => listener(route));
  };

  constructor(private readonly target: Window, private readonly codec: OperatorRouteCodec) {
    target.addEventListener("popstate", this.onPopState);
  }

  navigate(route: UnsafeRouteInput, replace = false): void {
    const url = this.codec.serialize(route);
    this.target.history[replace ? "replaceState" : "pushState"]({}, "", url);
  }

  subscribe(listener: (route: SafeRoute) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  dispose(): void {
    this.target.removeEventListener("popstate", this.onPopState);
    this.listeners.clear();
  }
}

function clean(value: string | null): string | undefined {
  if (!value) return undefined;
  return value.slice(0, 160);
}
