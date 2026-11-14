import { invoke } from "@tauri-apps/api/core";

import {
  object,
  identity,
  bounded,
  absent,
  text,
  domainError,
  decodeOwnedReply,
  decodeRequestFailure,
  replyUnavailable,
  type ObservationError,
} from "./observation-contracts";
export { object, identity } from "./observation-contracts";
export type RepositoryError = ObservationError;
export type {
  Availability,
  Repository,
  RepositoryPage,
  Registration,
  RepositoryRequest,
  RepositoryReply,
} from "@/generated/contracts/types";
export type RegistrationRequest = RegisterRepository;
import type {
  RegisterRepository,
  Repository,
  Registration,
  RepositoryRequest,
  RepositoryReply,
} from "@/generated/contracts/types";
import { validateRepositoryReply } from "@/generated/contracts/validators.js";

function repositorySemantics(value: Repository): boolean {
  return (
    text(value.name, 24 * 1024) &&
    value.name.length > 0 &&
    text(value.path, 24 * 1024) &&
    bounded(value, 24 * 1024) &&
    (value.availability.state === "unavailable"
      ? domainError(value.availability.error)
      : absent(value.availability, "error"))
  );
}
function registrationSemantics(value: Registration): boolean {
  return (
    text(value.path, 4096) &&
    (value.state === "completed" || value.initial_workspace_id === null) &&
    (value.initial_workspace_id === null || value.create_initial_workspace) &&
    (value.state === "pending"
      ? absent(value, "repository_id", "error")
      : value.state === "completed"
        ? absent(value, "error")
        : absent(value, "repository_id") && domainError(value.error))
  );
}
function replySemantics(value: RepositoryReply): boolean {
  if (!bounded(value, 48 * 1024)) return false;
  switch (value.kind) {
    case "registration":
      return registrationSemantics(value.registration);
    case "action":
      return (
        value.registration === null || registrationSemantics(value.registration)
      );
    case "repository":
      return value.repository === null || repositorySemantics(value.repository);
    case "repositories":
      return (
        value.page.repositories.length <= 32 &&
        value.page.repositories.every(repositorySemantics)
      );
  }
}
export async function repositoryRequest(
  expectedOwner: string,
  request: RepositoryRequest,
): Promise<RepositoryReply> {
  let value: unknown;
  try {
    value = await invoke<unknown>("repository_request", {
      expectedOwner,
      request,
    });
  } catch (error) {
    throw decodeRequestFailure(error);
  }
  return decodeOwnedReply(expectedOwner, value, (reply) =>
    decodeRepositoryReply(request, reply),
  );
}
export function decodeRepositoryReply(
  request: RepositoryRequest,
  value: unknown,
): RepositoryReply {
  const expected = {
    register: "registration",
    action: "action",
    list: "repositories",
    get: "repository",
  }[request.operation];
  if (
    !validateRepositoryReply(value) ||
    !replySemantics(value) ||
    value.kind !== expected
  )
    throw replyUnavailable(
      "The supervisor returned an invalid repository observation. Refresh to reconnect to a matching build.",
    );
  if (
    (request.operation === "register" || request.operation === "action") &&
    (value.kind === "registration" || value.kind === "action") &&
    value.registration
  ) {
    const id =
      request.operation === "register"
        ? request.request.action_id
        : request.action_id;
    if (
      value.registration.action_id !== id ||
      (request.operation === "register" &&
        (value.registration.path !== request.request.path ||
          value.registration.create_initial_workspace !==
            (request.request.create_initial_workspace ?? false)))
    )
      throw replyUnavailable(
        "Repository action observation has the wrong identity. Refresh its recorded status.",
      );
  }
  if (
    request.operation === "get" &&
    value.kind === "repository" &&
    value.repository &&
    value.repository.id !== request.id
  )
    throw replyUnavailable("Repository observation has the wrong identity.");
  return value;
}
export function repositoryError(value: unknown): RepositoryError {
  return object(value) &&
    typeof value.code === "string" &&
    typeof value.message === "string"
    ? (value as RepositoryError)
    : {
        code: "connection",
        message: value instanceof Error ? value.message : String(value),
      };
}

export const SELECTION_KEY = "agent-workspace.selected-repository";
export const PENDING_KEY = "agent-workspace.pending-registration";
export type PendingRegistration = {
  request: RegistrationRequest;
  selection: string | null;
  uncertain?: boolean;
};
export function readSelection(): string | null {
  try {
    const value = localStorage.getItem(SELECTION_KEY);
    return identity(value) ? value : null;
  } catch {
    return null;
  }
}
export function readPending(): PendingRegistration | null {
  try {
    const stored = localStorage.getItem(PENDING_KEY);
    const value: unknown = stored ? JSON.parse(stored) : null;
    if (
      object(value) &&
      object(value.request) &&
      identity(value.request.action_id) &&
      typeof value.request.path === "string" &&
      (value.request.create_initial_workspace === undefined ||
        typeof value.request.create_initial_workspace === "boolean") &&
      (value.selection === null || identity(value.selection))
    ) {
      return {
        request: {
          action_id: value.request.action_id,
          path: value.request.path,
          create_initial_workspace:
            value.request.create_initial_workspace === true,
        },
        selection: value.selection,
        // Legacy records may already have crossed IPC. An over-budget path could never be sent.
        uncertain: value.uncertain !== false && text(value.request.path, 4096),
      };
    }
  } catch {
    /* A malformed presentation preference grants no execution authority. */
  }
  return null;
}
