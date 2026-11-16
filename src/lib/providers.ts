import { invoke } from "@tauri-apps/api/core";
import { validateProviderReply } from "@/generated/contracts/validators.js";
import type {
  ProviderReply,
  ProviderRequest,
  ProviderTarget,
} from "@/generated/contracts/types";
import {
  bounded,
  decodeOwnedReply,
  decodeRequestFailure,
  identity,
  replyUnavailable,
  text,
} from "./observation-contracts";

function targetSemantics(target: ProviderTarget) {
  return (
    identity(target.id) &&
    target.provider === "codex" &&
    [target.codex_home, target.working_directory].every(
      (value) =>
        text(value, 4096) &&
        value.length > 0 &&
        !/[\u0000-\u001f\u007f]/.test(value),
    ) &&
    [target.executable, target.selected_executable].every(
      (value) =>
        value === null ||
        (text(value, 4096) &&
          value.length > 0 &&
          !/[\u0000-\u001f\u007f]/.test(value)),
    ) &&
    (target.discovery_source === null ||
      ["path", "shell", "conventional", "selected", "fixture"].includes(
        target.discovery_source,
      )) &&
    text(target.minimum_supported_version, 128)
  );
}
export function decodeProviderReply(
  request: ProviderRequest,
  value: unknown,
): ProviderReply {
  if (!validateProviderReply(value) || !bounded(value, 24 * 1024))
    throw replyUnavailable(
      "Invalid provider setup observation. Refresh using a matching build.",
    );
  if (
    request.operation === "target" &&
    value.kind === "target" &&
    targetSemantics(value.target)
  )
    return value;
  if ("target_id" in request && value.kind === "observation") {
    const observation = value.observation;
    const evidence = observation.evidence;
    if (
      observation.target.id === request.target_id &&
      (!("action_id" in request) ||
        request.operation === "reconcile_selection" ||
        observation.selection?.id === request.action_id) &&
      (!observation.selection ||
        (identity(observation.selection.id) &&
          text(observation.selection.path, 4096) &&
          observation.selection.path.startsWith("/") &&
          !/[\u0000-\u001f\u007f]/.test(observation.selection.path))) &&
      (!("attempt_id" in request) ||
        request.operation === "reconcile_login" ||
        observation.login?.id === request.attempt_id) &&
      targetSemantics(observation.target) &&
      Number.isSafeInteger(observation.revision) &&
      observation.revision >= 0 &&
      observation.revision <= 0xffff_ffff &&
      (observation.diagnostic === null ||
        (text(observation.diagnostic, 512) &&
          !/[\u0000-\u001f\u007f]/.test(observation.diagnostic))) &&
      (!evidence ||
        [
          evidence.version,
          evidence.account,
          evidence.plan,
          evidence.model_provider,
          evidence.model,
        ].every(
          (value) =>
            value === null ||
            (text(value, 512) && !/[\u0000-\u001f\u007f]/.test(value)),
        )) &&
      (observation.phase !== "cleanup_uncertain" ||
        observation.problem === "cleanup_uncertain") &&
      (!observation.login ||
        (identity(observation.login.id) &&
          (observation.login.diagnostic === null ||
            (text(observation.login.diagnostic, 512) &&
              !/[\u0000-\u001f\u007f]/.test(observation.login.diagnostic)))))
    )
      return value;
  }
  throw replyUnavailable(
    "Provider setup has the wrong configuration target. Refresh its recorded status.",
  );
}
export async function providerRequest(
  expectedOwner: string,
  request: ProviderRequest,
): Promise<ProviderReply> {
  let value: unknown;
  try {
    value = await invoke<unknown>("provider_request", {
      expectedOwner,
      request,
    });
  } catch (error) {
    throw decodeRequestFailure(error);
  }
  return decodeOwnedReply(expectedOwner, value, (reply) =>
    decodeProviderReply(request, reply),
  );
}
