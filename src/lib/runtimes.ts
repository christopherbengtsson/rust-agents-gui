import { invoke } from "@tauri-apps/api/core";

export type RuntimeTarget = { owner: string; runtime: "codex" };
export type RuntimeConfiguration = {
  revision: number;
  executable: string | null;
};
export type RuntimeError = { code: string; message: string };
export type RuntimeMutation =
  | {
      operation: "save";
      target: RuntimeTarget;
      action_id: string;
      revision: number;
      executable: string | null;
    }
  | {
      operation: "check";
      target: RuntimeTarget;
      action_id: string;
      revision: number;
    };
export type RuntimeRequest =
  | RuntimeMutation
  | { operation: "observe"; target: RuntimeTarget }
  | { operation: "action"; target: RuntimeTarget; action_id: string };
export type RuntimePhase =
  | "pending"
  | "saving"
  | "discovering"
  | "version"
  | "initializing"
  | "account"
  | "configuration"
  | "cleaning"
  | "completed"
  | "failed"
  | "uncertain";
export type RuntimeAction = {
  action_id: string;
  request: RuntimeMutation;
  phase: RuntimePhase;
  error: RuntimeError | null;
};
export type RuntimeDiagnostic = {
  check_id: string;
  revision: number;
  timestamp_ms: number;
  executable: string | null;
  version: string | null;
  authentication: string;
  provider: string | null;
  billing: string;
  passed: boolean;
  error: RuntimeError | null;
  cleanup_confirmed: boolean;
  exit_successful: boolean | null;
};
export type RuntimeReply = {
  target: RuntimeTarget;
  configuration: RuntimeConfiguration | null;
  busy: boolean;
  active: RuntimeAction | null;
  lookup: {
    action_id: string;
    complete: boolean;
    action: RuntimeAction | null;
  } | null;
  diagnostic: RuntimeDiagnostic | null;
  blocked: RuntimeError | null;
};

const codes = new Set([
  "invalid_request",
  "stale_target",
  "busy",
  "storage",
  "persistence_uncertain",
  "connection",
  "missing_executable",
  "incompatible_version",
  "protocol",
  "authentication",
  "configuration",
  "timeout",
  "output_limit",
  "process",
  "cleanup_unconfirmed",
]);
const phases = new Set([
  "pending",
  "saving",
  "discovering",
  "version",
  "initializing",
  "account",
  "configuration",
  "cleaning",
  "completed",
  "failed",
  "uncertain",
]);
const object = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);
const string = (v: unknown): v is string =>
  typeof v === "string" && v.length <= 4096;
const nullableString = (v: unknown) => v === null || string(v);
const counter = (v: unknown): v is number =>
  typeof v === "number" && Number.isSafeInteger(v) && v >= 0;
const id = (v: unknown): v is string =>
  typeof v === "string" &&
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(v);
const target = (v: unknown): v is RuntimeTarget =>
  object(v) && string(v.owner) && v.owner.length > 0 && v.runtime === "codex";
export const isRuntimeError = (v: unknown): v is RuntimeError =>
  object(v) &&
  typeof v.code === "string" &&
  codes.has(v.code) &&
  string(v.message);
const errorOrNull = (v: unknown) => v === null || isRuntimeError(v);
function mutation(v: unknown): v is RuntimeMutation {
  return (
    object(v) &&
    target(v.target) &&
    id(v.action_id) &&
    counter(v.revision) &&
    (v.operation === "check" ||
      (v.operation === "save" && nullableString(v.executable)))
  );
}
function action(v: unknown): v is RuntimeAction {
  return (
    object(v) &&
    id(v.action_id) &&
    mutation(v.request) &&
    v.action_id === v.request.action_id &&
    typeof v.phase === "string" &&
    phases.has(v.phase) &&
    errorOrNull(v.error)
  );
}
function diagnostic(v: unknown): v is RuntimeDiagnostic {
  return (
    object(v) &&
    id(v.check_id) &&
    counter(v.revision) &&
    counter(v.timestamp_ms) &&
    nullableString(v.executable) &&
    nullableString(v.version) &&
    string(v.authentication) &&
    nullableString(v.provider) &&
    string(v.billing) &&
    typeof v.passed === "boolean" &&
    typeof v.cleanup_confirmed === "boolean" &&
    (v.exit_successful === null || typeof v.exit_successful === "boolean") &&
    errorOrNull(v.error) &&
    (!v.passed || (v.cleanup_confirmed && v.error === null))
  );
}
export function validateRuntimeReply(
  value: unknown,
  request: RuntimeRequest,
  pending?: RuntimeMutation | null,
): RuntimeReply {
  let valid =
    object(value) &&
    target(value.target) &&
    value.target.owner === request.target.owner &&
    value.target.runtime === request.target.runtime &&
    (value.configuration === null ||
      (object(value.configuration) &&
        counter(value.configuration.revision) &&
        nullableString(value.configuration.executable))) &&
    typeof value.busy === "boolean" &&
    (value.active === null || action(value.active)) &&
    (value.diagnostic === null || diagnostic(value.diagnostic)) &&
    errorOrNull(value.blocked);
  if (valid && object(value)) {
    valid =
      value.lookup === null ||
      (object(value.lookup) &&
        id(value.lookup.action_id) &&
        typeof value.lookup.complete === "boolean" &&
        (value.lookup.action === null || action(value.lookup.action)));
    if (request.operation === "action") {
      valid =
        valid &&
        object(value.lookup) &&
        value.lookup.action_id === request.action_id;
      if (
        valid &&
        object(value.lookup) &&
        value.lookup.action !== null &&
        action(value.lookup.action)
      ) {
        valid =
          value.lookup.action.action_id === request.action_id &&
          (!pending || sameMutation(value.lookup.action.request, pending));
      }
    } else valid = valid && value.lookup === null;
    if (
      valid &&
      value.diagnostic !== null &&
      diagnostic(value.diagnostic) &&
      object(value.configuration)
    )
      valid = value.diagnostic.revision === value.configuration.revision;
    if (
      valid &&
      value.active !== null &&
      action(value.active) &&
      !terminal(value.active.phase)
    )
      valid = value.active.request.target.owner === request.target.owner;
  }
  if (!valid)
    throw new Error(
      "Invalid runtime observation or target identity. Refresh with matching desktop/supervisor builds; no setup action was replayed.",
    );
  return value as RuntimeReply; // Shape, discriminants and targeted identities checked above.
}
export function sameMutation(a: RuntimeMutation, b: RuntimeMutation) {
  return (
    a.operation === b.operation &&
    a.action_id === b.action_id &&
    a.target.owner === b.target.owner &&
    a.target.runtime === b.target.runtime &&
    a.revision === b.revision &&
    (a.operation !== "save" ||
      (b.operation === "save" && a.executable === b.executable))
  );
}
export async function runtimeRequest(
  request: RuntimeRequest,
  pending?: RuntimeMutation | null,
) {
  return validateRuntimeReply(
    await invoke<unknown>("runtime_request", { request }),
    request,
    pending,
  );
}
export const terminal = (phase: RuntimePhase) =>
  phase === "completed" || phase === "failed" || phase === "uncertain";
export function runtimeError(value: unknown): RuntimeError {
  return isRuntimeError(value)
    ? value
    : {
        code: "connection",
        message:
          "Runtime observations are unavailable. Refresh saved configuration and action status; no setup action was replayed.",
      };
}
export const RUNTIME_PENDING = "agent-workspace.pending-runtime";
export function readRuntimePending(): RuntimeMutation | null {
  try {
    const v: unknown = JSON.parse(
      localStorage.getItem(RUNTIME_PENDING) ?? "null",
    );
    return mutation(v) ? v : null;
  } catch {
    return null;
  }
}
export const phaseLabel: Record<RuntimePhase, string> = {
  pending: "Waiting for setup…",
  saving: "Saving executable…",
  discovering: "Finding Codex…",
  version: "Checking version…",
  initializing: "Initializing Codex…",
  account: "Reading existing authentication…",
  configuration: "Reading configuration…",
  cleaning: "Closing owned Codex process…",
  completed: "Setup action completed",
  failed: "Setup needs attention",
  uncertain: "Setup outcome uncertain",
};
