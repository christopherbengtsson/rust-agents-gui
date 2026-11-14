import type { invoke } from "@tauri-apps/api/core";
import { domainError, object } from "../lib/observation-contracts";

// Adapt existing feature scenarios to the application-owned Tauri envelope.
// New wire-contract tests supply raw envelopes directly instead.
export function ownerBound(
  handler: (command: string, args?: Parameters<typeof invoke>[1]) => unknown,
) {
  return async (command: string, args?: Parameters<typeof invoke>[1]) => {
    if (command !== "repository_request" && command !== "workspace_request")
      return handler(command, args);
    if (!object(args) || typeof args.expectedOwner !== "string")
      throw new Error("Missing caller-owner context in fixture");
    const owner = args.expectedOwner;
    try {
      return { owner, result: { Ok: await handler(command, args) } };
    } catch (error) {
      if (domainError(error)) return { owner, result: { Err: error } };
      throw error;
    }
  };
}
