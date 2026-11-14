// Generated structure precedes local authority, state and byte-budget assertions.
import {
  validateDesktopPid,
  validateOwnedObservation,
  validateRepositoryError,
  validateRepositoryId,
  validateRequestFailure,
  validateSupervisorSnapshot,
} from "@/generated/contracts/validators.js";
import type {
  Delivery,
  RepositoryError,
  SupervisorSnapshot,
} from "@/generated/contracts/types";
export type { Delivery, SupervisorSnapshot } from "@/generated/contracts/types";
export type ObservationError = {
  code: string;
  message: string;
  delivery?: Delivery;
  source?: "domain" | "local" | "transport";
};
export function object(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
export function identity(value: unknown): value is string {
  // UUID formats also accept URNs; serialized application identities use 36 characters.
  return validateRepositoryId(value) && value.length === 36;
}
export function absent(value: object, ...keys: string[]): boolean {
  return keys.every((key) => Reflect.get(value, key) === undefined);
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
export function domainError(value: unknown): value is RepositoryError {
  return validateRepositoryError(value) && text(value.message, 2051);
}
function pidRange(value: number): boolean {
  return value > 0 && value <= 0xffff_ffff;
}
export function decodeDesktopPid(value: unknown): number {
  if (!validateDesktopPid(value) || !pidRange(value))
    throw new Error(
      "Invalid desktop identity. Refresh using a matching build.",
    );
  return value;
}
export function decodeSnapshot(value: unknown): SupervisorSnapshot {
  if (
    !validateSupervisorSnapshot(value) ||
    !bounded(value, 4096) ||
    !pidRange(value.status.supervisor_pid) ||
    !text(value.status.application_version, 128) ||
    !value.status.application_version.length ||
    !owner(value.cursor.owner) ||
    !Number.isSafeInteger(value.cursor.revision)
  )
    throw new Error(
      "Invalid supervisor snapshot. Refresh using a matching build.",
    );
  return value;
}
export function replyUnavailable(message: string): ObservationError {
  return Object.assign(new Error(message), {
    code: "reply_unavailable",
    source: "transport" as const,
    delivery: "unknown" as const,
  });
}
export function decodeRequestFailure(value: unknown): ObservationError {
  if (validateRequestFailure(value)) {
    if (value.kind === "rejected" && domainError(value.error))
      return { ...value.error, source: "local", delivery: "not_sent" };
    if (value.kind === "transport") {
      const error = value.error;
      const delivery = {
        busy: "not_sent",
        not_connected: "not_sent",
        owner_changed: "rejected",
        reply_unavailable: "unknown",
        incompatible: "rejected",
      }[error.code];
      if (error.delivery === delivery && text(error.message, 2051))
        return { ...error, source: "transport" };
    }
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
    !validateOwnedObservation(value) ||
    !bounded(value, 64 * 1024) ||
    value.owner !== expectedOwner ||
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
