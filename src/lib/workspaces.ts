import { invoke } from "@tauri-apps/api/core";
import { identity, object, type RepositoryError } from "./repositories";
import {
  validInspectionReply,
  type InspectionRequest,
  type InspectionReply,
} from "./changes";

export type WorkspaceStatus =
  | { state: "queued" | "ready" }
  | {
      state: "running";
      stage:
        | "allocating_name"
        | "resolving_base"
        | "reserving_folder"
        | "creating_branch"
        | "creating_worktree";
    }
  | { state: "failed" | "uncertain"; error: RepositoryError };
export type Workspace = {
  id: string;
  repository_id: string;
  creation_action_id: string;
  name: string | null;
  branch: string | null;
  path: string | null;
  legacy: boolean;
  provider: "codex";
  base: { reference: string; commit: string } | null;
  artifacts: {
    branch: "unknown" | "absent" | "owned" | "conflict";
    folder:
      | "unknown"
      | "absent"
      | "reserved"
      | "worktree"
      | "partial"
      | "conflict";
  };
  status: WorkspaceStatus;
  cleanup_unconfirmed: boolean;
};
export type WorkspaceAction =
  | { operation: "create"; action_id: string; repository_id: string }
  | { operation: "reconcile"; action_id: string; workspace_id: string };
export type WorkspaceRequest =
  | WorkspaceAction
  | InspectionRequest
  | { operation: "action"; action_id: string }
  | { operation: "get"; id: string }
  | { operation: "list"; repository_id: string; after: string | null };
export type WorkspacePage = { workspaces: Workspace[]; next: string | null };
export type WorkspaceReply =
  | InspectionReply
  | { kind: "workspace"; workspace: Workspace | null }
  | {
      kind: "action";
      action: WorkspaceAction | null;
      workspace: Workspace | null;
    }
  | { kind: "workspaces"; page: WorkspacePage };
function error(value: unknown): value is RepositoryError {
  return (
    object(value) &&
    typeof value.code === "string" &&
    typeof value.message === "string"
  );
}
function action(value: unknown): value is WorkspaceAction {
  return (
    object(value) &&
    identity(value.action_id) &&
    ((value.operation === "create" && identity(value.repository_id)) ||
      (value.operation === "reconcile" && identity(value.workspace_id)))
  );
}
function workspace(value: unknown): value is Workspace {
  if (
    !object(value) ||
    !identity(value.id) ||
    !identity(value.repository_id) ||
    !identity(value.creation_action_id) ||
    !(
      (value.name === null && value.branch === null && value.path === null) ||
      (typeof value.name === "string" &&
        value.name.length > 0 &&
        typeof value.branch === "string" &&
        value.branch.length > 0 &&
        typeof value.path === "string" &&
        value.path.length > 0)
    ) ||
    typeof value.legacy !== "boolean" ||
    value.provider !== "codex" ||
    typeof value.cleanup_unconfirmed !== "boolean" ||
    !object(value.status) ||
    !object(value.artifacts)
  )
    return false;
  return (
    (value.name !== null ||
      (value.status.state !== "ready" &&
        (value.status.state !== "running" ||
          value.status.stage === "allocating_name"))) &&
    (value.base === null ||
      (object(value.base) &&
        typeof value.base.reference === "string" &&
        typeof value.base.commit === "string" &&
        /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/i.test(value.base.commit))) &&
    ["unknown", "absent", "owned", "conflict"].includes(
      String(value.artifacts.branch),
    ) &&
    [
      "unknown",
      "absent",
      "reserved",
      "worktree",
      "partial",
      "conflict",
    ].includes(String(value.artifacts.folder)) &&
    (value.status.state === "queued" ||
      value.status.state === "ready" ||
      (value.status.state === "running" &&
        [
          "allocating_name",
          "resolving_base",
          "reserving_folder",
          "creating_branch",
          "creating_worktree",
        ].includes(String(value.status.stage))) ||
      ((value.status.state === "failed" ||
        value.status.state === "uncertain") &&
        error(value.status.error)))
  );
}
export async function workspaceRequest(
  request: WorkspaceRequest,
): Promise<WorkspaceReply> {
  const value = await invoke<unknown>("workspace_request", { request });
  let valid = false;
  if (object(value)) {
    if (
      request.operation === "changes" ||
      request.operation === "change_preview"
    )
      valid = validInspectionReply(request, value);
    else if (request.operation === "list")
      valid =
        value.kind === "workspaces" &&
        object(value.page) &&
        Array.isArray(value.page.workspaces) &&
        value.page.workspaces.length <= 32 &&
        value.page.workspaces.every(
          (w: unknown) =>
            workspace(w) && w.repository_id === request.repository_id,
        ) &&
        (value.page.next === null || identity(value.page.next));
    else if (request.operation === "get")
      valid =
        value.kind === "workspace" &&
        (value.workspace === null ||
          (workspace(value.workspace) && value.workspace.id === request.id));
    else
      valid =
        value.kind === "action" &&
        ((value.action === null && value.workspace === null) ||
          (action(value.action) &&
            workspace(value.workspace) &&
            value.action.action_id === request.action_id &&
            (value.action.operation === "create"
              ? value.action.repository_id === value.workspace.repository_id &&
                value.workspace.creation_action_id === value.action.action_id
              : value.action.workspace_id === value.workspace.id) &&
            (request.operation === "action" ||
              (request.operation === value.action.operation &&
                (request.operation === "create"
                  ? request.repository_id === value.workspace.repository_id
                  : request.workspace_id === value.workspace.id)))));
  }
  if (!valid)
    throw new Error(
      "Invalid workspace observation or target identity. Refresh recorded status using a matching desktop/supervisor build.",
    );
  return value as WorkspaceReply; // Runtime contracts and exact targets checked above.
}
export const WORKSPACE_SELECTION = "agent-workspace.selected-workspace";
export const WORKSPACE_PENDING = "agent-workspace.pending-workspace";
export function readWorkspaceSelection(): {
  repository_id: string;
  workspace_id: string;
} | null {
  try {
    const v: unknown = JSON.parse(
      localStorage.getItem(WORKSPACE_SELECTION) ?? "null",
    );
    if (object(v) && identity(v.repository_id) && identity(v.workspace_id))
      return { repository_id: v.repository_id, workspace_id: v.workspace_id };
  } catch {
    /* Preferences grant no execution authority. */
  }
  return null;
}
export function readWorkspacePending(): WorkspaceAction | null {
  try {
    const v: unknown = JSON.parse(
      localStorage.getItem(WORKSPACE_PENDING) ?? "null",
    );
    if (action(v)) return v;
  } catch {
    /* Reconnect never invents a request. */
  }
  return null;
}
export function workspaceStatus(status: WorkspaceStatus): string {
  switch (status.state) {
    case "queued":
      return "Waiting to create workspace…";
    case "running":
      return {
        allocating_name: "Choosing a workspace name…",
        resolving_base: "Resolving local default branch…",
        reserving_folder: "Reserving workspace folder…",
        creating_branch: "Creating branch…",
        creating_worktree: "Creating worktree…",
      }[status.stage];
    case "ready":
      return "Workspace ready";
    case "failed":
      return "Creation needs attention";
    case "uncertain":
      return "Creation outcome uncertain";
  }
}
