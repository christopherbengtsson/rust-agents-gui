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

export function isSettingsPath(pathname: string) {
  return pathname === "/settings" || pathname.startsWith("/settings/");
}

// History supplies the current destination. This owner retains eligibility and
// ephemeral Settings return metadata, never another current selection or body.
export class Navigation {
  private generation = 0;
  private restorationClaimed = false;
  private restorationGeneration?: number;
  private settingsTriggerKey?: string;
  private settingsReturn?: {
    key: string | undefined;
    focus: "settings-entry" | "workspace";
  };
  private returnFocus?: "settings-entry" | "workspace";
  private returningSettings = false;
  constructor(
    readonly history: RouterHistory,
    private navigate: (target: MainDestination, replace?: boolean) => void,
  ) {}
  start() {
    let previous = this.history.location;
    return this.history.subscribe(({ location, action }) => {
      this.generation++;
      this.returningSettings = false;
      this.returnFocus = undefined;
      const wasSettings = isSettingsPath(previous.pathname);
      const settings = isSettingsPath(location.pathname);
      if (settings && !wasSettings && action.type === "PUSH") {
        this.settingsReturn = mainDestination(previous.href)
          ? {
              key: location.state.__TSR_key,
              focus:
                this.settingsTriggerKey === previous.state.__TSR_key
                  ? "settings-entry"
                  : "workspace",
            }
          : undefined;
      }
      if (
        settings &&
        wasSettings &&
        action.type === "REPLACE" &&
        this.settingsReturn &&
        this.settingsReturn.key === previous.state.__TSR_key
      )
        this.settingsReturn.key = location.state.__TSR_key;
      if (wasSettings && !settings) {
        this.returnFocus =
          this.settingsReturn &&
          this.settingsReturn.key === previous.state.__TSR_key &&
          location.state.__TSR_index === previous.state.__TSR_index - 1
            ? this.settingsReturn.focus
            : "workspace";
      }
      this.settingsTriggerKey = undefined;
      previous = location;
    });
  }
  rememberSettingsTrigger = () => {
    this.settingsTriggerKey = this.history.location.state.__TSR_key;
  };
  returnFromSettings = () => {
    if (
      !isSettingsPath(this.history.location.pathname) ||
      this.returningSettings
    )
      return;
    this.returningSettings = true;
    if (
      this.settingsReturn &&
      this.settingsReturn.key === this.history.location.state.__TSR_key &&
      this.history.canGoBack()
    )
      this.history.back();
    else this.go({ kind: "home" }, true);
  };
  takeReturnFocus() {
    const focus = this.returnFocus;
    this.returnFocus = undefined;
    return focus;
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
