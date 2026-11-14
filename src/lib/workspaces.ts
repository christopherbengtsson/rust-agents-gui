import type { ObservationScope } from "./observation-coordinator";
import { invoke } from "@tauri-apps/api/core";
import {
  identity,
  object,
  bounded,
  domainError,
  absent,
  decodeOwnedReply,
  decodeRequestFailure,
  replyUnavailable,
} from "./observation-contracts";
import { validInspectionReply } from "./changes";

export type {
  Workspace,
  WorkspaceAction,
  WorkspaceRequest,
  WorkspacePage,
  WorkspaceReply,
} from "@/generated/contracts/types";
export type WorkspaceStatus = CreationStatus;
import type {
  CreationStatus,
  Workspace,
  WorkspaceAction,
  WorkspaceRequest,
  WorkspaceReply,
} from "@/generated/contracts/types";
import {
  validateWorkspaceAction,
  validateWorkspaceReply,
} from "@/generated/contracts/validators.js";

function normalizedWorkspacePath(path: string): boolean {
  return (
    path === "/" ||
    (path.startsWith("/") &&
      path
        .slice(1)
        .split("/")
        .every((part) => part.length > 0 && part !== "." && part !== ".."))
  );
}
function workspaceSemantics(value: Workspace): boolean {
  const allocated =
    value.name !== null &&
    value.name.length > 0 &&
    value.branch !== null &&
    value.branch.length > 0 &&
    value.path !== null &&
    value.path.length > 0;
  const pending =
    value.name === null && value.branch === null && value.path === null;
  const status = value.status;
  return (
    (allocated || pending) &&
    value.provider === "codex" &&
    (value.path === null || normalizedWorkspacePath(value.path)) &&
    (status.state === "running" || absent(status, "stage")) &&
    (status.state === "failed" || status.state === "uncertain"
      ? domainError(status.error)
      : absent(status, "error")) &&
    (allocated ||
      (status.state !== "ready" &&
        (status.state !== "running" || status.stage === "allocating_name"))) &&
    (value.base === null ||
      /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/i.test(value.base.commit))
  );
}
export function readWorkspace(
  scope: ObservationScope,
  expectedOwner: string,
  request: Exclude<WorkspaceRequest, WorkspaceAction>,
) {
  return scope.read(["workspace", expectedOwner, request], () =>
    workspaceRequest(expectedOwner, request),
  );
}

export async function workspaceRequest(
  expectedOwner: string,
  request: WorkspaceRequest,
): Promise<WorkspaceReply> {
  let value: unknown;
  try {
    value = await invoke<unknown>("workspace_request", {
      expectedOwner,
      request,
    });
  } catch (error) {
    throw decodeRequestFailure(error);
  }
  return decodeOwnedReply(expectedOwner, value, (reply) =>
    decodeWorkspaceReply(request, reply),
  );
}
export function decodeWorkspaceReply(
  request: WorkspaceRequest,
  value: unknown,
): WorkspaceReply {
  let valid = false;
  if (validateWorkspaceReply(value) && bounded(value, 48 * 1024)) {
    if (
      request.operation === "changes" ||
      request.operation === "change_preview"
    )
      valid = validInspectionReply(request, value);
    else if (request.operation === "list" && value.kind === "workspaces")
      valid =
        value.page.workspaces.length <= 32 &&
        value.page.workspaces.every(
          (w) =>
            workspaceSemantics(w) && w.repository_id === request.repository_id,
        );
    else if (request.operation === "get" && value.kind === "workspace")
      valid =
        value.workspace === null ||
        (workspaceSemantics(value.workspace) &&
          value.workspace.id === request.id);
    else if (
      request.operation !== "get" &&
      request.operation !== "list" &&
      value.kind === "action"
    ) {
      const action = value.action;
      const workspace = value.workspace;
      valid =
        (request.operation === "action" &&
          action === null &&
          workspace === null) ||
        (action !== null &&
          workspace !== null &&
          workspaceSemantics(workspace) &&
          action.action_id === request.action_id &&
          (action.operation === "create"
            ? action.repository_id === workspace.repository_id &&
              workspace.creation_action_id === action.action_id
            : action.workspace_id === workspace.id) &&
          (request.operation === "action" ||
            (request.operation === action.operation &&
              (request.operation === "create"
                ? request.repository_id === workspace.repository_id
                : request.workspace_id === workspace.id))));
    }
    if (valid) return value;
  }
  throw replyUnavailable(
    "Invalid workspace observation or target identity. Refresh recorded status using a matching desktop/supervisor build.",
  );
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
export type PendingWorkspace = { request: WorkspaceAction; uncertain: boolean };
export function readWorkspacePending(): PendingWorkspace | null {
  try {
    const v: unknown = JSON.parse(
      localStorage.getItem(WORKSPACE_PENDING) ?? "null",
    );
    if (object(v) && validateWorkspaceAction(v.request))
      return { request: v.request, uncertain: v.uncertain !== false };
    if (validateWorkspaceAction(v)) return { request: v, uncertain: true };
  } catch {
    /* Reconnect never invents a request or removes stored recovery evidence. */
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
