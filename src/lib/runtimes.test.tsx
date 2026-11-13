import { expect, test } from "vitest";
import { validateRuntimeReply, type RuntimeReply } from "./runtimes";

const target = { owner: "owner", runtime: "codex" as const };
export const readyRuntime: RuntimeReply = {
  target,
  configuration: { revision: 1, executable: null },
  busy: false,
  active: null,
  lookup: null,
  diagnostic: null,
  blocked: null,
};
test("runtime observations reject wrong owners, malformed fields and unrelated action IDs", () => {
  const request = { operation: "observe" as const, target };
  expect(validateRuntimeReply(readyRuntime, request)).toEqual(readyRuntime);
  for (const invalid of [
    { ...readyRuntime, target: { ...target, owner: "old" } },
    { ...readyRuntime, busy: "false" },
    { ...readyRuntime, configuration: { revision: -1, executable: null } },
  ])
    expect(() => validateRuntimeReply(invalid, request)).toThrow(
      /Invalid runtime/,
    );
  expect(() =>
    validateRuntimeReply(
      {
        ...readyRuntime,
        lookup: {
          action_id: "00000000-0000-0000-0000-000000000001",
          complete: true,
          action: null,
        },
      },
      {
        operation: "action",
        target,
        action_id: "00000000-0000-0000-0000-000000000002",
      },
    ),
  ).toThrow();
});

test("repeated mutation replies validate the exact recorded action rather than the latest active action", () => {
  const request = {
    operation: "check" as const,
    target,
    action_id: "00000000-0000-0000-0000-000000000001",
    revision: 1,
  };
  const reply = {
    ...readyRuntime,
    configuration: { revision: 2, executable: null },
    lookup: {
      action_id: request.action_id,
      complete: true,
      action: {
        action_id: request.action_id,
        request,
        phase: "completed",
        error: null,
      },
    },
  };
  expect(validateRuntimeReply(reply, request).lookup?.action?.action_id).toBe(
    request.action_id,
  );
  expect(() =>
    validateRuntimeReply(
      {
        ...reply,
        lookup: {
          ...reply.lookup,
          action_id: "00000000-0000-0000-0000-000000000002",
        },
      },
      request,
    ),
  ).toThrow();
});
