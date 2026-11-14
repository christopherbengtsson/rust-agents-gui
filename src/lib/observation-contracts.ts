// Pure wire guards. Structural validity does not establish current owner authority.
export type Delivery = "not_sent" | "rejected" | "unknown";
export type ObservationError = {
  code: string;
  message: string;
  delivery?: Delivery;
  source?: "domain" | "local" | "transport";
};
export type SupervisorSnapshot = {
  status: {
    state: "ready";
    supervisor_pid: number;
    application_version: string;
  };
  cursor: { owner: string; revision: number };
  reset: boolean;
};
export function object(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
export function identity(value: unknown): value is string {
  return (
    typeof value === "string" &&
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(
      value,
    )
  );
}
export function bounded(value: unknown, bytes: number): boolean {
  try {
    const json = JSON.stringify(value);
    return (
      json !== undefined && new TextEncoder().encode(json).byteLength <= bytes
    );
  } catch {
    return false;
  }
}
export function text(value: unknown, bytes: number): value is string {
  return (
    typeof value === "string" &&
    new TextEncoder().encode(value).byteLength <= bytes
  );
}
export function owner(value: unknown): value is string {
  return (
    text(value, 128) &&
    value.length > 0 &&
    !/[\s\u0000-\u001f\u007f]/.test(value)
  );
}
export function domainError(value: unknown): value is ObservationError {
  return (
    object(value) &&
    typeof value.code === "string" &&
    [
      "invalid_path",
      "git",
      "storage",
      "conflict",
      "capacity",
      "not_found",
      "unavailable",
      "connection",
      "cleanup_unconfirmed",
    ].includes(value.code) &&
    text(value.message, 2051)
  );
}
function pid(value: unknown): value is number {
  return (
    typeof value === "number" &&
    Number.isInteger(value) &&
    value > 0 &&
    value <= 0xffff_ffff
  );
}
export function decodeDesktopPid(value: unknown): number {
  if (!pid(value))
    throw new Error(
      "Invalid desktop identity. Refresh using a matching build.",
    );
  return value;
}
export function decodeSnapshot(value: unknown): SupervisorSnapshot {
  if (
    !object(value) ||
    !bounded(value, 4096) ||
    !object(value.status) ||
    value.status.state !== "ready" ||
    !pid(value.status.supervisor_pid) ||
    !text(value.status.application_version, 128) ||
    !value.status.application_version.length ||
    !object(value.cursor) ||
    !owner(value.cursor.owner) ||
    typeof value.cursor.revision !== "number" ||
    !Number.isSafeInteger(value.cursor.revision) ||
    value.cursor.revision < 0 ||
    typeof value.reset !== "boolean"
  )
    throw new Error(
      "Invalid supervisor snapshot. Refresh using a matching build.",
    );
  return value as SupervisorSnapshot;
}
export function replyUnavailable(message: string): ObservationError {
  return Object.assign(new Error(message), {
    code: "reply_unavailable",
    source: "transport" as const,
    delivery: "unknown" as const,
  });
}
export function decodeRequestFailure(value: unknown): ObservationError {
  if (object(value) && value.kind === "rejected" && domainError(value.error))
    return { ...value.error, source: "local", delivery: "not_sent" };
  if (object(value) && value.kind === "transport" && object(value.error)) {
    const error = value.error;
    const delivery = {
      busy: "not_sent",
      not_connected: "not_sent",
      owner_changed: "rejected",
      reply_unavailable: "unknown",
      incompatible: "rejected",
    }[String(error.code)];
    if (delivery && error.delivery === delivery && text(error.message, 2051))
      return {
        code: String(error.code),
        message: error.message,
        delivery: error.delivery as Delivery,
        source: "transport",
      };
  }
  return replyUnavailable(
    value instanceof Error
      ? value.message
      : "Invalid or unavailable supervisor reply. Refresh recorded action status.",
  );
}
export function decodeOwnedReply<T>(
  expectedOwner: string,
  value: unknown,
  decode: (value: unknown) => T,
): T {
  if (
    !owner(expectedOwner) ||
    !object(value) ||
    !bounded(value, 64 * 1024) ||
    value.owner !== expectedOwner ||
    !object(value.result) ||
    Object.keys(value.result).length !== 1
  )
    throw replyUnavailable(
      "Invalid observation or serving owner. Refresh recorded status using a matching build.",
    );
  if ("Err" in value.result) {
    if (!domainError(value.result.Err))
      throw replyUnavailable("Invalid domain failure observation.");
    const error = value.result.Err;
    const rejected = [
      "invalid_path",
      "capacity",
      "not_found",
      "conflict",
      "cleanup_unconfirmed",
    ].includes(error.code);
    throw {
      ...error,
      source: "domain",
      delivery: rejected ? "rejected" : "unknown",
    } satisfies ObservationError;
  }
  if (!("Ok" in value.result))
    throw replyUnavailable("Invalid observation result variant.");
  return decode(value.result.Ok);
}
// A later attempt's rejection cannot disprove delivery of an earlier attempt.
export function uncertainAfter(
  previous: boolean,
  failure: ObservationError,
): boolean {
  return (
    previous || failure.delivery === undefined || failure.delivery === "unknown"
  );
}

export function pendingFailure(
  previousUncertain: boolean,
  failure: ObservationError,
): ObservationError {
  return previousUncertain && failure.delivery !== "unknown"
    ? {
        ...failure,
        message: `An earlier attempt remains uncertain. ${failure.message} Refresh its recorded action status.`,
      }
    : failure;
}
