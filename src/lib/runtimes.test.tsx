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
