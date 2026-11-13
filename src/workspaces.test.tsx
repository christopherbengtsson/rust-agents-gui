import { cleanup, renderHook, act, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { object } from "./lib/repositories";
import { useWorkspaces } from "./hooks/use-workspaces";
import { WORKSPACE_PENDING, type Workspace } from "./lib/workspaces";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const repository_id = "00000000-0000-4000-8000-000000000001";
const id = "00000000-0000-4000-8000-000000000002";
const creation_action_id = "00000000-0000-4000-8000-000000000003";
const w: Workspace = {
  id,
  repository_id,
  creation_action_id,
  name: "kyoto",
  branch: "workspace/kyoto",
  path: "/state/kyoto/checkout",
  provider: "codex",
  base: { reference: "refs/heads/main", commit: "a".repeat(40) },
  artifacts: { branch: "owned", folder: "worktree" },
  status: { state: "ready" },
  cleanup_unconfirmed: false,
};
beforeEach(() => {
  localStorage.clear();
  vi.mocked(invoke).mockResolvedValue({
    kind: "workspaces",
    page: { workspaces: [], next: null },
  });
});
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});
test("a restored uncertain action is read without replaying creation", async () => {
  localStorage.setItem(
    WORKSPACE_PENDING,
    JSON.stringify({
      operation: "create",
      action_id: creation_action_id,
      repository_id,
    }),
  );
  vi.mocked(invoke).mockImplementation(async (_, args) =>
    (object(args) ? args.request : undefined) &&
    ((object(args) ? args.request : undefined) as { operation: string })
      .operation === "action"
      ? {
          kind: "action",
          action: {
            operation: "create",
            action_id: creation_action_id,
            repository_id,
          },
          workspace: w,
        }
      : { kind: "workspaces", page: { workspaces: [w], next: null } },
  );
  const { result } = renderHook(() =>
    useWorkspaces("owner", repository_id, [repository_id].join(","), null),
  );
  await waitFor(() => expect(result.current.pending).toBeNull());
  expect(
    vi
      .mocked(invoke)
      .mock.calls.some(
        ([, a]) =>
          ((object(a) ? a.request : undefined) as { operation?: string })
            ?.operation === "create",
      ),
  ).toBe(false);
});
test("explicit creation sends once with a retained action identity", async () => {
  let actions = 0;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const request = (object(args) ? args.request : undefined) as {
      operation: string;
      action_id: string;
    };
    if (request.operation === "create") {
      actions++;
      return {
        kind: "action",
        action: {
          operation: "create",
          action_id: request.action_id,
          repository_id,
        },
        workspace: { ...w, creation_action_id: request.action_id },
      };
    }
    return { kind: "workspaces", page: { workspaces: [], next: null } };
  });
  const { result } = renderHook(() =>
    useWorkspaces("owner", repository_id, repository_id, null),
  );
  await act(async () => {
    await result.current.create(repository_id);
  });
  expect(actions).toBe(1);
  expect(result.current.selected?.id).toBe(id);
});

test("a lost creation reply retrieves the same action and selects its failed workspace", async () => {
  let createCount = 0;
  let savedAction: unknown;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const request = (object(args) ? args.request : undefined) as {
      operation: string;
      action_id: string;
    };
    if (request.operation === "create") {
      createCount++;
      savedAction = {
        operation: "create",
        action_id: request.action_id,
        repository_id,
      };
      throw new Error("Reply lost");
    }
    const failed = {
      ...w,
      creation_action_id: object(savedAction)
        ? savedAction.action_id
        : creation_action_id,
      status: {
        state: "failed",
        error: { code: "git", message: "Default branch unavailable" },
      },
    };
    if (request.operation === "action")
      return { kind: "action", action: savedAction, workspace: failed };
    if (request.operation === "get")
      return { kind: "workspace", workspace: failed };
    return {
      kind: "workspaces",
      page: { workspaces: savedAction ? [failed] : [], next: null },
    };
  });
  const { result } = renderHook(() =>
    useWorkspaces("owner", repository_id, repository_id, null),
  );
  await act(async () => {
    await result.current.create(repository_id);
  });
  await waitFor(() =>
    expect(result.current.selected?.status.state).toBe("failed"),
  );
  expect(createCount).toBe(1);
  expect(result.current.pending).toBeNull();
});
test("switching selection prevents a late creation reply from stealing context", async () => {
  let finish: ((v: unknown) => void) | undefined;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const request = (object(args) ? args.request : undefined) as {
      operation: string;
      action_id: string;
    };
    if (request.operation === "create")
      return new Promise((resolve) => {
        finish = () =>
          resolve({
            kind: "action",
            action: {
              operation: "create",
              action_id: request.action_id,
              repository_id,
            },
            workspace: { ...w, creation_action_id: request.action_id },
          });
      });
    if (request.operation === "get") return { kind: "workspace", workspace: w };
    return { kind: "workspaces", page: { workspaces: [w], next: null } };
  });
  const { result } = renderHook(() =>
    useWorkspaces("owner", repository_id, repository_id, null),
  );
  let sending: Promise<void>;
  act(() => {
    sending = result.current.create(repository_id);
  });
  act(() => result.current.selectRepository());
  await act(async () => {
    finish?.(null);
    await sending;
  });
  expect(result.current.selected).toBeUndefined();
});
test("registration's recorded initial workspace is observed without a create request", async () => {
  vi.mocked(invoke).mockImplementation(async (_, args) =>
    (object(args) ? args.request : (undefined as unknown)) &&
    (object(args) ? (args.request as { operation: string }) : undefined)
      ?.operation === "get"
      ? { kind: "workspace", workspace: w }
      : { kind: "workspaces", page: { workspaces: [w], next: null } },
  );
  const initial = { repository_id, workspace_id: id };
  const { result } = renderHook(() =>
    useWorkspaces("owner", repository_id, repository_id, initial),
  );
  await waitFor(() => expect(result.current.selected?.id).toBe(id));
  expect(
    vi
      .mocked(invoke)
      .mock.calls.every(([, args]) =>
        ["get", "list"].includes(
          ((object(args) ? args.request : undefined) as { operation: string })
            .operation,
        ),
      ),
  ).toBe(true);
});

test("a delayed initial-workspace lookup cannot override explicit repository selection", async () => {
  let resolveDetail: ((v: unknown) => void) | undefined;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const request = (object(args) ? args.request : undefined) as {
      operation: string;
    };
    if (request.operation === "get")
      return new Promise((resolve) => {
        resolveDetail = resolve;
      });
    return { kind: "workspaces", page: { workspaces: [w], next: null } };
  });
  const initial = { repository_id, workspace_id: id };
  const { result } = renderHook(() =>
    useWorkspaces("owner", repository_id, repository_id, initial),
  );
  await waitFor(() => expect(resolveDetail).toBeDefined());
  act(() => result.current.selectRepository());
  await act(async () => resolveDetail?.({ kind: "workspace", workspace: w }));
  expect(result.current.selected).toBeUndefined();
  await act(async () => {
    await result.current.refresh();
  });
  expect(result.current.selected).toBeUndefined();
});

test("definite admission rejection retains the error but does not block another workspace's recovery", async () => {
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const request = (object(args) ? args.request : undefined) as {
      operation: string;
      action_id: string;
      workspace_id: string;
    };
    if (request.operation === "create")
      throw {
        code: "cleanup_unconfirmed",
        message: "Reconcile the earlier workspace first",
      };
    if (request.operation === "reconcile")
      return { kind: "action", action: request, workspace: w };
    if (request.operation === "get") return { kind: "workspace", workspace: w };
    return { kind: "workspaces", page: { workspaces: [w], next: null } };
  });
  const { result } = renderHook(() =>
    useWorkspaces("owner", repository_id, repository_id, null),
  );
  await act(async () => {
    await result.current.create(repository_id);
  });
  expect(result.current.pending).toBeNull();
  expect(localStorage.getItem(WORKSPACE_PENDING)).toBeNull();
  expect(result.current.error?.message).toBe(
    "Reconcile the earlier workspace first",
  );
  await act(async () => {
    await result.current.reconcile(w);
  });
  expect(result.current.error).toBeUndefined();
});

test("definite rejection follows its pending action across repository navigation", async () => {
  let reject: ((error: unknown) => void) | undefined;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const request = (object(args) ? args.request : undefined) as {
      operation: string;
    };
    if (request.operation === "create")
      return new Promise((_, fail) => {
        reject = fail;
      });
    if (request.operation === "action")
      return { kind: "action", action: null, workspace: null };
    return { kind: "workspaces", page: { workspaces: [], next: null } };
  });
  const other = "00000000-0000-4000-8000-000000000009";
  const { result, rerender } = renderHook(
    ({ repository }) =>
      useWorkspaces("owner", repository, `${repository_id},${other}`, null),
    { initialProps: { repository: repository_id } },
  );
  let submitted: Promise<void>;
  act(() => {
    submitted = result.current.create(other);
  });
  rerender({ repository: other });
  await act(async () => {
    reject?.({
      code: "cleanup_unconfirmed",
      message: "Reconcile earlier work",
    });
    await submitted;
  });
  expect(result.current.pending).toBeNull();
  expect(localStorage.getItem(WORKSPACE_PENDING)).toBeNull();
  expect(result.current.error?.code).toBe("cleanup_unconfirmed");
});
