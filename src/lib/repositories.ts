import { invoke } from "@tauri-apps/api/core";

export type RepositoryError = { code: string; message: string };
export type Availability =
  | { state: "checking" | "available" }
  | { state: "unavailable"; error: RepositoryError };
export type Repository = {
  id: string;
  name: string;
  path: string;
  availability: Availability;
};
export type RepositoryPage = {
  repositories: Repository[];
  next: string | null;
};
export type Registration = {
  action_id: string;
  path: string;
  initial_workspace_id?: string | null;
  create_initial_workspace?: boolean;
} & (
  | { state: "pending" }
  | { state: "completed"; repository_id: string }
  | { state: "failed"; error: RepositoryError }
);
export type RegistrationRequest = {
  action_id: string;
  path: string;
  create_initial_workspace?: boolean;
};
export type RepositoryRequest =
  | { operation: "register"; request: RegistrationRequest }
  | { operation: "action"; action_id: string }
  | { operation: "list"; after: string | null; refresh: boolean }
  | { operation: "get"; id: string; refresh: boolean };
export type RepositoryReply =
  | { kind: "registration"; registration: Registration }
  | { kind: "action"; registration: Registration | null }
  | { kind: "repositories"; page: RepositoryPage }
  | { kind: "repository"; repository: Repository | null };

export function object(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
export function identity(value: unknown): value is string {
  return (
    typeof value === "string" &&
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(
      value,
    )
  );
}
function error(value: unknown): value is RepositoryError {
  return (
    object(value) &&
    typeof value.code === "string" &&
    typeof value.message === "string"
  );
}
function availability(value: unknown): value is Availability {
  return (
    object(value) &&
    (value.state === "available" ||
      value.state === "checking" ||
      (value.state === "unavailable" && error(value.error)))
  );
}
function repository(value: unknown): value is Repository {
  return (
    object(value) &&
    identity(value.id) &&
    typeof value.name === "string" &&
    typeof value.path === "string" &&
    availability(value.availability)
  );
}
function registration(value: unknown): value is Registration {
  return (
    object(value) &&
    identity(value.action_id) &&
    typeof value.path === "string" &&
    (value.initial_workspace_id === undefined ||
      value.initial_workspace_id === null ||
      identity(value.initial_workspace_id)) &&
    (value.state === "pending" ||
      (value.state === "completed" && identity(value.repository_id)) ||
      (value.state === "failed" && error(value.error)))
  );
}
function reply(value: unknown): value is RepositoryReply {
  if (!object(value)) return false;
  switch (value.kind) {
    case "registration":
      return registration(value.registration);
    case "action":
      return value.registration === null || registration(value.registration);
    case "repository":
      return value.repository === null || repository(value.repository);
    case "repositories":
      return (
        object(value.page) &&
        Array.isArray(value.page.repositories) &&
        value.page.repositories.length <= 32 &&
        value.page.repositories.every(repository) &&
        (value.page.next === null || identity(value.page.next))
      );
    default:
      return false;
  }
}
export async function repositoryRequest(
  request: RepositoryRequest,
): Promise<RepositoryReply> {
  const value = await invoke<unknown>("repository_request", { request });
  const expected = {
    register: "registration",
    action: "action",
    list: "repositories",
    get: "repository",
  }[request.operation];
  if (!reply(value) || value.kind !== expected)
    throw new Error(
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
        value.registration.path !== request.request.path)
    )
      throw new Error(
        "Repository action observation has the wrong identity. Refresh its recorded status.",
      );
  }
  if (
    request.operation === "get" &&
    value.kind === "repository" &&
    value.repository &&
    value.repository.id !== request.id
  )
    throw new Error("Repository observation has the wrong identity.");
  return value;
}
export function repositoryError(value: unknown): RepositoryError {
  return error(value)
    ? value
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
    const text = localStorage.getItem(PENDING_KEY);
    const value: unknown = text ? JSON.parse(text) : null;
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
      };
    }
  } catch {
    /* A malformed presentation preference grants no execution authority. */
  }
  return null;
}
