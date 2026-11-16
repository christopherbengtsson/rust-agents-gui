import type { invoke } from "@tauri-apps/api/core";
import { domainError, object } from "../lib/observation-contracts";

// Adapt existing feature scenarios to the application-owned Tauri envelope.
// New wire-contract tests supply raw envelopes directly instead.
export function incarnationBound(
  handler: (command: string, args?: Parameters<typeof invoke>[1]) => unknown,
) {
  return async (command: string, args?: Parameters<typeof invoke>[1]) => {
    if (command === "observation_subscribe") {
      const result = await handler(command, args);
      if (
        object(args) &&
        object(args.onEvent) &&
        typeof args.onEvent.onmessage === "function"
      ) {
        args.onEvent.onmessage({
          kind: "batch",
          batch: {
            subscription_id: args.subscriptionId,
            batch_id: 1,
            cursor: { incarnation: args.expectedIncarnation, sequence: 1 },
            sample: { kind: "resync_required", watermark: 1 },
          },
        });
      }
      return result;
    }
    if (
      command !== "repository_request" &&
      command !== "workspace_request" &&
      command !== "provider_request"
    )
      return handler(command, args);
    if (!object(args) || typeof args.expectedIncarnation !== "string")
      throw new Error("Missing caller-incarnation context in fixture");
    const incarnation = args.expectedIncarnation;
    try {
      return { incarnation, result: { Ok: await handler(command, args) } };
    } catch (error) {
      if (domainError(error)) return { incarnation, result: { Err: error } };
      throw error;
    }
  };
}
