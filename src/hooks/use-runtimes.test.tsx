import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useRuntimes } from "./use-runtimes";
import {
  RUNTIME_PENDING,
  type RuntimeMutation,
  type RuntimeReply,
  type RuntimeRequest,
} from "@/lib/runtimes";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => localStorage.clear());
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});
const initial: RuntimeReply = {
  target: { owner: "owner", runtime: "codex" },
  configuration: { revision: 1, executable: null },
  busy: false,
  active: null,
  lookup: null,
  diagnostic: null,
  blocked: null,
};
function calls() {
  return vi
    .mocked(invoke)
    .mock.calls.map(
      ([, args]) => (args as { request: RuntimeRequest }).request,
    );
}

test("mount, Settings visibility and reconnect only read runtime observations", async () => {
  vi.mocked(invoke).mockImplementation(async (_, args) => ({
    ...initial,
    target: (args as { request: RuntimeRequest }).request.target,
  }));
  const view = renderHook(({ owner, visible }) => useRuntimes(owner, visible), {
    initialProps: { owner: "owner", visible: false },
  });
  await waitFor(() =>
    expect(view.result.current.observation?.configuration?.revision).toBe(1),
  );
  view.rerender({ owner: "new-owner", visible: true });
  await waitFor(() =>
    expect(view.result.current.observation?.target.owner).toBe("new-owner"),
  );
  expect(calls().every((r) => r.operation === "observe")).toBe(true);
});

test("lost Save reply is reconciled by exact status lookup without replay", async () => {
  let saved: RuntimeMutation | undefined;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const request = (args as { request: RuntimeRequest }).request;
    if (request.operation === "save") {
      saved = request;
      throw { code: "connection", message: "Lost reply" };
    }
    if (request.operation === "action" && saved)
      return {
        ...initial,
        configuration: { revision: 2, executable: "/new/codex" },
        lookup: {
          action_id: request.action_id,
          complete: true,
          action: {
            action_id: saved.action_id,
            request: saved,
            phase: "completed",
            error: null,
          },
        },
      };
    return initial;
  });
  const { result } = renderHook(() => useRuntimes("owner", true));
  await waitFor(() => expect(result.current.canAct).toBe(true));
  await act(() => result.current.save("/new/codex"));
  expect(result.current.observation?.configuration?.executable).toBe(
    "/new/codex",
  );
  expect(result.current.pending).toBeNull();
  expect(localStorage.getItem(RUNTIME_PENDING)).toBeNull();
  expect(calls().filter((r) => r.operation === "save")).toHaveLength(1);
  expect(calls().filter((r) => r.operation === "action")).toHaveLength(1);
});

test("failed status read keeps evidence stale and retains recovery identity", async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce(initial)
    .mockRejectedValue({ code: "connection", message: "Disconnected" });
  const { result } = renderHook(() => useRuntimes("owner", true));
  await waitFor(() => expect(result.current.canAct).toBe(true));
  await act(() => result.current.check());
  expect(result.current.stale).toBe(true);
  expect(result.current.observation).toEqual(initial);
  expect(result.current.pending?.operation).toBe("check");
  expect(result.current.canAct).toBe(false);
  expect(calls().filter((r) => r.operation === "check")).toHaveLength(1);
});

test("restored pending actions read the original identity under a changed owner", async () => {
  const request: RuntimeMutation = {
    operation: "check",
    target: initial.target,
    action_id: "00000000-0000-0000-0000-000000000007",
    revision: 1,
  };
  localStorage.setItem(RUNTIME_PENDING, JSON.stringify(request));
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const read = (args as { request: RuntimeRequest }).request;
    return {
      ...initial,
      target: read.target,
      blocked: {
        code: "cleanup_unconfirmed",
        message: "Owned cleanup uncertain",
      },
      lookup: {
        action_id: request.action_id,
        complete: true,
        action: {
          action_id: request.action_id,
          request,
          phase: "uncertain",
          error: {
            code: "cleanup_unconfirmed",
            message: "Owned cleanup uncertain",
          },
        },
      },
    };
  });
  const { result } = renderHook(() => useRuntimes("new-owner", true));
  await waitFor(() =>
    expect(result.current.observation?.blocked?.code).toBe(
      "cleanup_unconfirmed",
    ),
  );
  expect(result.current.pending?.action_id).toBe(request.action_id);
  expect(calls().every((r) => r.operation === "action")).toBe(true);
});

test("definite rejection releases local pending work without inventing provider failure", async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce(initial)
    .mockRejectedValueOnce({
      code: "invalid_request",
      message: "Invalid executable",
    })
    .mockResolvedValue(initial);
  const { result } = renderHook(() => useRuntimes("owner", true));
  await waitFor(() => expect(result.current.canAct).toBe(true));
  await act(() => result.current.save("relative"));
  expect(result.current.pending).toBeNull();
  expect(result.current.actionError?.code).toBe("invalid_request");
  expect(result.current.observation?.diagnostic).toBeNull();
});

test("owner change during an in-flight read drains it before observing the new owner", async () => {
  let release: ((value: RuntimeReply) => void) | undefined;
  vi.mocked(invoke)
    .mockImplementationOnce(
      () =>
        new Promise<RuntimeReply>((resolve) => {
          release = resolve;
        }),
    )
    .mockImplementation(async (_, args) => ({
      ...initial,
      target: (args as { request: RuntimeRequest }).request.target,
    }));
  const view = renderHook(({ owner }) => useRuntimes(owner, true), {
    initialProps: { owner: "owner" },
  });
  view.rerender({ owner: "new-owner" });
  await act(async () => {
    release?.(initial);
  });
  await waitFor(() =>
    expect(view.result.current.observation?.target.owner).toBe("new-owner"),
  );
  expect(calls().every((r) => r.operation === "observe")).toBe(true);
});
