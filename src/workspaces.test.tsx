import { cleanup, renderHook, act, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { object } from "./lib/repositories";
import { useWorkspaces as useWorkspaceObservations } from "./hooks/use-workspaces";
import {
  WORKSPACE_PENDING,
  workspaceRequest,
  type Workspace,
} from "./lib/workspaces";
import { useAppNavigation } from "./use-app-navigation";

function useWorkspaces(
  owner: string | undefined,
  repositoryId: string | undefined,
  repositoryIds: string,
  initial: Parameters<typeof useWorkspaceObservations>[3],
) {
  const navigation = useAppNavigation();
  return useWorkspaceObservations(
    owner,
    repositoryId,
    repositoryIds,
    initial,
    navigation.selection,
  );
}

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
  legacy: false,
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
  vi.useRealTimers();
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

test.each(["create-first", "lookup-first", "lookup-before-admission"])(
  "creation in an unselected repository retains selection: %s",
  async (schedule) => {
    const other = "00000000-0000-4000-8000-000000000009";
    const created = { ...w, repository_id: other };
    let finishCreate: (() => void) | undefined;
    let finishLookup: (() => void) | undefined;
    let creates = 0;
    let accepted = false;
    let saved: unknown;
    vi.mocked(invoke).mockImplementation(async (_, args) => {
      const request = (object(args) ? args.request : undefined) as {
        operation: string;
        action_id: string;
        repository_id: string;
      };
      const reply = () => ({
        kind: "action",
        action: saved,
        workspace: {
          ...created,
          creation_action_id: object(saved)
            ? saved.action_id
            : creation_action_id,
        },
      });
      if (request.operation === "create") {
        creates++;
        saved = request;
        return new Promise((resolve) => {
          finishCreate = () => {
            accepted = true;
            resolve(reply());
          };
        });
      }
      if (request.operation === "action")
        return new Promise((resolve) => {
          finishLookup = () =>
            resolve(
              schedule === "lookup-before-admission"
                ? { kind: "action", action: null, workspace: null }
                : reply(),
            );
        });
      if (request.operation === "get")
        return { kind: "workspace", workspace: created };
      return {
        kind: "workspaces",
        page: {
          workspaces:
            accepted && request.repository_id === other ? [created] : [],
          next: null,
        },
      };
    });
    const { result, rerender } = renderHook(
      ({ repository }) =>
        useWorkspaces("owner", repository, `${repository_id},${other}`, null),
      { initialProps: { repository: repository_id } },
    );
    await act(async () => {});
    let submission: Promise<void>;
    act(() => {
      submission = result.current.create(other);
    });
    rerender({ repository: other });
    await waitFor(() => expect(finishLookup).toBeDefined());
    if (schedule === "create-first") {
      await act(async () => {
        finishCreate?.();
        await submission;
      });
      await act(async () => finishLookup?.());
    } else {
      await act(async () => finishLookup?.());
      await act(async () => {
        finishCreate?.();
        await submission;
      });
    }
    expect(creates).toBe(1);
    expect(result.current.pending).toBeNull();
    expect(result.current.selected?.id).toBe(id);
    expect(result.current.selected?.repository_id).toBe(other);
  },
);

test("an old owner's creation reply cannot select a workspace after reconnect", async () => {
  let finish: (() => void) | undefined;
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
            action: { ...request, repository_id },
            workspace: { ...w, creation_action_id: request.action_id },
          });
      });
    if (request.operation === "action")
      return { kind: "action", action: null, workspace: null };
    return { kind: "workspaces", page: { workspaces: [], next: null } };
  });
  const { result, rerender } = renderHook(
    ({ owner }) => useWorkspaces(owner, repository_id, repository_id, null),
    { initialProps: { owner: "old" } },
  );
  await act(async () => {});
  let submission: Promise<void>;
  act(() => {
    submission = result.current.create(repository_id);
  });
  rerender({ owner: "new" });
  await act(async () => {
    finish?.();
    await submission;
  });
  expect(result.current.selected).toBeUndefined();
  expect(result.current.pending).not.toBeNull();
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

test.each([
  ["settings", "reply"],
  ["settings and back", "reply"],
  ["away and back", "reply"],
  ["settings", "lookup"],
  ["settings and back", "lookup"],
  ["away and back", "lookup"],
])(
  "creation settles without redirecting after %s through %s",
  async (destination, settlement) => {
    const created = {
      ...w,
      id: "00000000-0000-4000-8000-000000000009",
      name: "new",
    };
    let finish!: () => void;
    let saved!: {
      operation: "create";
      action_id: string;
      repository_id: string;
    };
    let completed = false;
    let creates = 0;
    vi.mocked(invoke).mockImplementation(async (_, args) => {
      const request = (object(args) ? args.request : undefined) as {
        operation: string;
        id?: string;
      };
      const reply = () => ({
        kind: "action",
        action: saved,
        workspace: { ...created, creation_action_id: saved.action_id },
      });
      if (request.operation === "create") {
        creates++;
        saved = (object(args) ? args.request : undefined) as typeof saved;
        return new Promise((resolve) => {
          finish = () => resolve(reply());
        });
      }
      if (request.operation === "action")
        return completed
          ? reply()
          : { kind: "action", action: null, workspace: null };
      if (request.operation === "get")
        return { kind: "workspace", workspace: w };
      return {
        kind: "workspaces",
        page: { workspaces: completed ? [w, created] : [w], next: null },
      };
    });
    const { result } = renderHook(() => {
      const navigation = useAppNavigation();
      const workspaces = useWorkspaceObservations(
        "owner",
        repository_id,
        repository_id,
        null,
        navigation.selection,
      );
      return { navigation, workspaces };
    });
    act(() => result.current.workspaces.select(w));
    let sending!: Promise<void>;
    act(() => {
      sending = result.current.workspaces.create(repository_id);
    });
    act(() => {
      if (destination === "away and back") {
        result.current.workspaces.selectRepository();
        result.current.workspaces.select(w);
      } else {
        result.current.navigation.openSettings();
        if (destination === "settings and back")
          result.current.navigation.back();
      }
    });
    completed = true;
    if (settlement === "lookup")
      await act(async () => {
        await result.current.workspaces.refresh();
      });
    await act(async () => {
      finish();
      await sending;
    });
    await waitFor(() => expect(result.current.workspaces.pending).toBeNull());
    expect(result.current.workspaces.selected?.id).toBe(w.id);
    expect(
      result.current.workspaces.pages[repository_id].workspaces.map(
        (item) => item.id,
      ),
    ).toContain(created.id);
    expect(localStorage.getItem(WORKSPACE_PENDING)).toBeNull();
    expect(creates).toBe(1);
  },
);

test("retrying the same uncertain creation does not renew selection permission", async () => {
  let first = true;
  const submitted: string[] = [];
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const request = (object(args) ? args.request : undefined) as {
      operation: string;
      action_id: string;
    };
    if (request.operation === "create") {
      submitted.push(request.action_id);
      if (first) {
        first = false;
        throw new Error("Reply unavailable");
      }
      return {
        kind: "action",
        action: request,
        workspace: {
          ...w,
          id: "00000000-0000-4000-8000-000000000009",
          creation_action_id: request.action_id,
        },
      };
    }
    if (request.operation === "action")
      return { kind: "action", action: null, workspace: null };
    if (request.operation === "get") return { kind: "workspace", workspace: w };
    return { kind: "workspaces", page: { workspaces: [w], next: null } };
  });
  const { result } = renderHook(() => {
    const navigation = useAppNavigation();
    const workspaces = useWorkspaceObservations(
      "owner",
      repository_id,
      repository_id,
      null,
      navigation.selection,
    );
    return { navigation, workspaces };
  });
  act(() => result.current.workspaces.select(w));
  await act(async () => {
    await result.current.workspaces.create(repository_id);
  });
  act(() => {
    result.current.navigation.openSettings();
    result.current.navigation.back();
  });
  await act(async () => {
    await result.current.workspaces.retry();
  });
  expect(result.current.workspaces.pending).toBeNull();
  expect(result.current.workspaces.selected?.id).toBe(w.id);
  expect(submitted).toHaveLength(2);
  expect(submitted[0]).toBe(submitted[1]);
});

test("shared workspace polling updates the selected record while Settings is visible", async () => {
  vi.useFakeTimers();
  let ready = false;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const request = (object(args) ? args.request : undefined) as {
      operation: string;
    };
    const observed: Workspace = {
      ...w,
      status: ready ? { state: "ready" } : { state: "queued" },
    };
    if (request.operation === "get")
      return { kind: "workspace", workspace: observed };
    return { kind: "workspaces", page: { workspaces: [observed], next: null } };
  });
  const { result } = renderHook(() => {
    const navigation = useAppNavigation();
    const workspaces = useWorkspaceObservations(
      "owner",
      repository_id,
      repository_id,
      null,
      navigation.selection,
    );
    return { navigation, workspaces };
  });
  await act(async () => {});
  act(() =>
    result.current.workspaces.select({ ...w, status: { state: "queued" } }),
  );
  act(() => result.current.navigation.openSettings());
  ready = true;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(250);
  });
  expect(result.current.navigation.screen).toBe("settings");
  expect(result.current.workspaces.selected?.status.state).toBe("ready");
  expect(result.current.workspaces.selected?.id).toBe(id);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.every(([, args]) =>
        ["list", "get"].includes(
          ((object(args) ? args.request : undefined) as { operation: string })
            .operation,
        ),
      ),
  ).toBe(true);
});
test("registration's recorded initial workspace is observed without a create request", async () => {
  vi.mocked(invoke).mockImplementation(async (_, args) =>
    (object(args) ? args.request : (undefined as unknown)) &&
    (object(args) ? (args.request as { operation: string }) : undefined)
      ?.operation === "get"
      ? { kind: "workspace", workspace: w }
      : { kind: "workspaces", page: { workspaces: [w], next: null } },
  );
  const initial = { repository_id, workspace_id: id, navigationGeneration: 0 };
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
  const initial = { repository_id, workspace_id: id, navigationGeneration: 0 };
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

test.each(["before lookup", "during lookup"])(
  "Settings invalidates registration's initial-workspace handoff %s",
  async (schedule) => {
    let finish!: (value: unknown) => void;
    let gets = 0;
    vi.mocked(invoke).mockImplementation(async (_, args) => {
      const request = (object(args) ? args.request : undefined) as {
        operation: string;
      };
      if (request.operation === "get") {
        gets++;
        return new Promise((resolve) => {
          finish = resolve;
        });
      }
      return { kind: "workspaces", page: { workspaces: [w], next: null } };
    });
    const initial = {
      repository_id,
      workspace_id: id,
      navigationGeneration: 0,
    };
    const { result, rerender } = renderHook(
      ({ handoff }) => {
        const navigation = useAppNavigation();
        const workspaces = useWorkspaceObservations(
          "owner",
          repository_id,
          repository_id,
          handoff ? initial : null,
          navigation.selection,
        );
        return { navigation, workspaces };
      },
      { initialProps: { handoff: schedule === "during lookup" } },
    );
    await act(async () => {});
    act(() => result.current.navigation.openSettings());
    act(() => result.current.navigation.back());
    if (schedule === "before lookup") rerender({ handoff: true });
    else await act(async () => finish({ kind: "workspace", workspace: w }));
    await act(async () => {
      await result.current.workspaces.refresh();
    });
    expect(result.current.workspaces.selected).toBeUndefined();
    expect(gets).toBe(schedule === "during lookup" ? 1 : 0);
    expect(
      result.current.workspaces.pages[repository_id].workspaces[0].id,
    ).toBe(id);
  },
);

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

test("a rejected action retains its error while unrelated workspace progress keeps polling", async () => {
  vi.useFakeTimers();
  let ready = false;
  const observed = () => ({
    ...w,
    status: ready ? w.status : { state: "running", stage: "creating_worktree" },
  });
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const request = (object(args) ? args.request : undefined) as {
      operation: string;
    };
    if (request.operation === "create")
      throw { code: "capacity", message: "Eight operations are pending" };
    if (request.operation === "get")
      return { kind: "workspace", workspace: observed() };
    return {
      kind: "workspaces",
      page: { workspaces: [observed()], next: null },
    };
  });
  try {
    const { result } = renderHook(() =>
      useWorkspaces("owner", repository_id, repository_id, null),
    );
    await act(async () => {});
    act(() =>
      result.current.select({
        ...w,
        status: { state: "running", stage: "creating_worktree" },
      }),
    );
    await act(async () => {
      await result.current.create(repository_id);
    });
    expect(result.current.error?.code).toBe("capacity");
    expect(result.current.pending).toBeNull();
    expect(result.current.selected?.status.state).toBe("running");
    ready = true;
    await act(async () => {
      await vi.advanceTimersByTimeAsync(250);
    });
    expect(result.current.selected?.status.state).toBe("ready");
    expect(result.current.error?.code).toBe("capacity");
  } finally {
    vi.useRealTimers();
  }
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

test("pending name allocation is a valid observation, but partial and ready-without-location replies are rejected", async () => {
  const request = { operation: "get" as const, id };
  const pending = {
    ...w,
    name: null,
    branch: null,
    path: null,
    status: { state: "running", stage: "allocating_name" },
  };
  vi.mocked(invoke).mockResolvedValue({
    kind: "workspace",
    workspace: pending,
  });
  expect((await workspaceRequest(request)).kind).toBe("workspace");
  for (const invalid of [
    { ...pending, branch: "workspace/marta" },
    { ...pending, status: { state: "ready" } },
    { ...pending, status: { state: "running", stage: "creating_branch" } },
  ]) {
    vi.mocked(invoke).mockResolvedValue({
      kind: "workspace",
      workspace: invalid,
    });
    await expect(workspaceRequest(request)).rejects.toThrow(
      "Invalid workspace observation",
    );
  }
});
