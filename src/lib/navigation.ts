import { defaultParseSearch, type RouterHistory } from "@tanstack/react-router";
import { identity, object } from "./observation-contracts";

export type WorkspaceView = "workspace" | "changes";
export type MainDestination =
  | { kind: "home" }
  | { kind: "repository"; repositoryId: string }
  | {
      kind: "workspace";
      repositoryId: string;
      workspaceId: string;
      view: WorkspaceView;
    };

export function workspaceSearch(search: Record<string, unknown>): {
  view: WorkspaceView;
} {
  if (search.view === undefined) return { view: "workspace" };
  if (search.view === "workspace" || search.view === "changes")
    return { view: search.view };
  throw new Error("Invalid workspace view. Open Workspace or Changes.");
}
export function savedDestination(value: unknown): MainDestination | undefined {
  if (!object(value)) return;
  if (value.kind === "home") return { kind: "home" };
  if (!identity(value.repositoryId)) return;
  if (value.kind === "repository")
    return { kind: "repository", repositoryId: value.repositoryId };
  if (
    value.kind === "workspace" &&
    identity(value.workspaceId) &&
    (value.view === "workspace" || value.view === "changes")
  )
    return {
      kind: "workspace",
      repositoryId: value.repositoryId,
      workspaceId: value.workspaceId,
      view: value.view,
    };
}
export function mainDestination(href: string): MainDestination | undefined {
  const url = new URL(href, "http://app.local");
  let parts: string[];
  try {
    parts = url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  } catch {
    return;
  }
  if (!parts.length) return { kind: "home" };
  if (parts[0] !== "repository" || !identity(parts[1])) return;
  if (parts.length === 2) return { kind: "repository", repositoryId: parts[1] };
  if (parts.length === 4 && parts[2] === "workspace" && identity(parts[3])) {
    let view: WorkspaceView;
    try {
      view = workspaceSearch(defaultParseSearch(url.search)).view;
    } catch {
      return;
    }
    return {
      kind: "workspace",
      repositoryId: parts[1],
      workspaceId: parts[3],
      view,
    };
  }
}

// History supplies the current destination. This owner retains only eligibility,
// never another current selection or observation body.
export class Navigation {
  private generation = 0;
  private restorationClaimed = false;
  private restorationGeneration?: number;
  constructor(
    readonly history: RouterHistory,
    private navigate: (target: MainDestination, replace?: boolean) => void,
  ) {}
  start() {
    return this.history.subscribe(() => {
      this.generation++;
    });
  }
  current = () => mainDestination(this.history.location.href);
  capture = () => this.generation;
  matches = (generation: number) => generation === this.generation;
  go = (target: MainDestination, replace = false) =>
    this.navigate(target, replace);
  complete(generation: number | undefined, target: MainDestination) {
    if (generation === undefined || !this.matches(generation)) return false;
    this.go(target);
    return true;
  }
  restoreOnce(saved: MainDestination | undefined) {
    if (!this.claimRestoration() || !saved || saved.kind === "home") return;
    this.go(saved, true);
    this.restorationGeneration = this.capture();
  }
  finishRestoration(status: "valid" | "pending" | "missing" | "unavailable") {
    const generation = this.restorationGeneration;
    if (generation === undefined) return false;
    if (!this.matches(generation) || status === "valid") {
      this.restorationGeneration = undefined;
      return false;
    }
    if (status !== "missing") return false;
    this.restorationGeneration = undefined;
    this.go({ kind: "home" }, true);
    return true;
  }
  claimRestoration() {
    if (this.restorationClaimed) return false;
    this.restorationClaimed = true;
    return this.history.location.pathname === "/";
  }
}
