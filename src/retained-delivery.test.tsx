import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useRepositories } from "./test-support/feature-hooks";
import { useWorkspaces } from "./test-support/feature-hooks";
import {
  object,
  PENDING_KEY,
  type RegistrationRequest,
} from "./lib/repositories";
import {
  WORKSPACE_PENDING,
  type WorkspaceAction,
  type Workspace,
} from "./lib/workspaces";
import { ownerBound } from "./test-support/owner-bound-invoke";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const repositoryId = "00000000-0000-4000-8000-000000000001";
const workspaceId = "00000000-0000-4000-8000-000000000002";
const rejection = [
  {
    kind: "transport",
    error: { code: "busy", delivery: "not_sent", message: "Busy" },
  },
  {
    kind: "transport",
    error: {
      code: "owner_changed",
      delivery: "rejected",
      message: "Refresh owner",
    },
  },
  {
    kind: "transport",
    error: {
      code: "not_connected",
      delivery: "not_sent",
      message: "Disconnected",
    },
  },
  {
    kind: "rejected",
    error: { code: "capacity", message: "Local capacity rejection" },
  },
];
beforeEach(() => localStorage.clear());
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

test.each(rejection)(
  "registration retains earlier uncertainty after $error.code until matching status",
  async (failure) => {
    const submissions: RegistrationRequest[] = [];
    let recorded = false;
    vi.mocked(invoke).mockImplementation(
      ownerBound(async (_, args) => {
        if (!object(args) || !object(args.request))
          throw new Error("Missing request");
        expect(args.expectedOwner).toBe("owner-a");
        const request = args.request;
        if (request.operation === "register") {
          const pending = JSON.parse(localStorage.getItem(PENDING_KEY)!);
          expect(pending.uncertain).toBe(true);
          submissions.push(request.request as RegistrationRequest);
          if (submissions.length === 1)
            throw new Error("Reply lost after transmission");
          throw failure;
        }
        if (request.operation === "action")
          return {
            kind: "action",
            registration: recorded
              ? {
                  ...submissions[0],
                  initial_workspace_id: null,
                  state: "completed",
                  repository_id: repositoryId,
                }
              : null,
          };
        return { kind: "repositories", page: { repositories: [], next: null } };
      }),
    );
    const first = renderHook(() => useRepositories("owner-a"));
    await waitFor(() => expect(first.result.current.loading).toBe(false));
    await act(async () => {
      await first.result.current.register("/projects/one");
    });
    expect(first.result.current.pending?.uncertain).toBe(true);
    await act(async () => {
      await first.result.current.register("/projects/one");
    });
    expect(submissions).toHaveLength(2);
    expect(submissions[1]).toEqual(submissions[0]);
    expect(first.result.current.actionError?.message).toContain(
      "earlier attempt remains uncertain",
    );
    expect(first.result.current.pending?.request).toEqual(submissions[0]);
    first.unmount();
    const restored = renderHook(() => useRepositories("owner-a"));
    await waitFor(() => expect(restored.result.current.loading).toBe(false));
    expect(restored.result.current.pending?.uncertain).toBe(true);
    expect(submissions).toHaveLength(2);
    recorded = true;
    await act(async () => {
      await restored.result.current.refresh(false);
    });
    expect(restored.result.current.pending).toBeNull();
    expect(localStorage.getItem(PENDING_KEY)).toBeNull();
    expect(submissions).toHaveLength(2);
  },
);

test.each(rejection)(
  "workspace retains earlier uncertainty after $error.code until matching status",
  async (failure) => {
    const submissions: WorkspaceAction[] = [];
    let recorded = false;
    vi.mocked(invoke).mockImplementation(
      ownerBound(async (_, args) => {
        if (!object(args) || !object(args.request))
          throw new Error("Missing request");
        expect(args.expectedOwner).toBe("owner-a");
        const request = args.request;
        if (request.operation === "create") {
          expect(
            JSON.parse(localStorage.getItem(WORKSPACE_PENDING)!).uncertain,
          ).toBe(true);
          submissions.push(request as WorkspaceAction);
          if (submissions.length === 1)
            throw new Error("Reply lost after transmission");
          throw failure;
        }
        if (request.operation === "action") {
          const workspace: Workspace = {
            id: workspaceId,
            repository_id: repositoryId,
            creation_action_id: submissions[0].action_id,
            name: null,
            branch: null,
            path: null,
            legacy: false,
            provider: "codex",
            base: null,
            artifacts: { branch: "unknown", folder: "unknown" },
            status: { state: "queued" },
            cleanup_unconfirmed: false,
          };
          return {
            kind: "action",
            action: recorded ? submissions[0] : null,
            workspace: recorded ? workspace : null,
          };
        }
        if (request.operation === "get")
          return { kind: "workspace", workspace: null };
        return { kind: "workspaces", page: { workspaces: [], next: null } };
      }),
    );
    const first = renderHook(() =>
      useWorkspaces("owner-a", repositoryId, repositoryId, null),
    );
    await act(async () => {});
    await act(async () => {
      await first.result.current.create(repositoryId);
    });
    await act(async () => {
      await first.result.current.retry();
    });
    expect(submissions).toHaveLength(2);
    expect(submissions[1]).toEqual(submissions[0]);
    expect(first.result.current.error?.message).toContain(
      "earlier attempt remains uncertain",
    );
    expect(first.result.current.pending).toEqual(submissions[0]);
    first.unmount();
    const restored = renderHook(() =>
      useWorkspaces("owner-a", repositoryId, repositoryId, null),
    );
    await act(async () => {});
    expect(restored.result.current.pending).toEqual(submissions[0]);
    expect(submissions).toHaveLength(2);
    recorded = true;
    await act(async () => {
      await restored.result.current.refresh();
    });
    expect(restored.result.current.pending).toBeNull();
    expect(localStorage.getItem(WORKSPACE_PENDING)).toBeNull();
    expect(submissions).toHaveLength(2);
  },
);

test("a late registration transport failure cannot resurrect an authoritatively settled action", async () => {
  let sent: RegistrationRequest | undefined;
  let reject!: (error: unknown) => void;
  vi.mocked(invoke).mockImplementation(
    ownerBound(async (_, args) => {
      if (!object(args) || !object(args.request))
        throw new Error("Missing request");
      const request = args.request;
      if (request.operation === "register") {
        sent = request.request as RegistrationRequest;
        return new Promise((_, fail) => {
          reject = fail;
        });
      }
      if (request.operation === "action")
        return {
          kind: "action",
          registration: {
            ...sent,
            initial_workspace_id: null,
            state: "completed",
            repository_id: repositoryId,
          },
        };
      return { kind: "repositories", page: { repositories: [], next: null } };
    }),
  );
  const { result } = renderHook(() => useRepositories("owner-a"));
  await waitFor(() => expect(result.current.loading).toBe(false));
  let submission!: Promise<boolean>;
  act(() => {
    submission = result.current.register("/projects/one");
  });
  await act(async () => {
    await result.current.refresh(false);
  });
  expect(result.current.pending).toBeNull();
  await act(async () => {
    reject(new Error("Late lost reply"));
    await submission;
  });
  expect(result.current.pending).toBeNull();
  expect(result.current.actionError).toBeUndefined();
  expect(localStorage.getItem(PENDING_KEY)).toBeNull();
});

test("a callback captured before owner replacement cannot select its old-owner result when invoked later", async () => {
  let submittedOwner: unknown;
  vi.mocked(invoke).mockImplementation(
    ownerBound((_, args) => {
      if (!object(args) || !object(args.request))
        throw new Error("Missing request");
      const request = args.request;
      if (request.operation === "create") {
        submittedOwner = args.expectedOwner;
        return {
          kind: "action",
          action: request,
          workspace: {
            id: workspaceId,
            repository_id: repositoryId,
            creation_action_id: request.action_id,
            name: null,
            branch: null,
            path: null,
            legacy: false,
            provider: "codex",
            base: null,
            artifacts: { branch: "unknown", folder: "unknown" },
            status: { state: "queued" },
            cleanup_unconfirmed: false,
          },
        };
      }
      if (request.operation === "action")
        return { kind: "action", action: null, workspace: null };
      return { kind: "workspaces", page: { workspaces: [], next: null } };
    }),
  );
  const view = renderHook(
    ({ owner }) => useWorkspaces(owner, repositoryId, repositoryId, null),
    { initialProps: { owner: "old" } },
  );
  await act(async () => {});
  const captured = view.result.current.create;
  view.rerender({ owner: "new" });
  await act(async () => {
    await captured(repositoryId);
  });
  expect(submittedOwner).toBe("old");
  expect(view.result.current.selected).toBeUndefined();
  expect(view.result.current.pending).not.toBeNull();
});

test("retained repository and workspace lookups reject conflicting payloads without clearing identities or submitting", async () => {
  const actionId = "00000000-0000-4000-8000-000000000003";
  const retained = {
    request: {
      action_id: actionId,
      path: "/original",
      create_initial_workspace: true,
    },
    selection: null,
    uncertain: true,
  };
  localStorage.setItem(PENDING_KEY, JSON.stringify(retained));
  const workspaceRequest = {
    operation: "create",
    action_id: actionId,
    repository_id: repositoryId,
  };
  localStorage.setItem(
    WORKSPACE_PENDING,
    JSON.stringify({ request: workspaceRequest, uncertain: true }),
  );
  vi.mocked(invoke).mockImplementation(
    ownerBound((command, args) => {
      if (!object(args) || !object(args.request))
        throw new Error("Missing request");
      if (
        args.request.operation === "action" &&
        command === "repository_request"
      )
        return {
          kind: "action",
          registration: {
            ...retained.request,
            path: "/different",
            initial_workspace_id: null,
            state: "completed",
            repository_id: repositoryId,
          },
        };
      if (args.request.operation === "action")
        return {
          kind: "action",
          action: { ...workspaceRequest, repository_id: workspaceId },
          workspace: {
            id: workspaceId,
            repository_id: workspaceId,
            creation_action_id: actionId,
            name: null,
            branch: null,
            path: null,
            legacy: false,
            provider: "codex",
            base: null,
            artifacts: { branch: "unknown", folder: "unknown" },
            status: { state: "queued" },
            cleanup_unconfirmed: false,
          },
        };
      return command === "repository_request"
        ? { kind: "repositories", page: { repositories: [], next: null } }
        : { kind: "workspaces", page: { workspaces: [], next: null } };
    }),
  );
  const view = renderHook(() => ({
    repositories: useRepositories("owner", undefined, null),
    workspaces: useWorkspaces("owner", repositoryId, repositoryId, null),
  }));
  await waitFor(() => {
    expect(view.result.current.repositories.actionError?.code).toBe("conflict");
    expect(view.result.current.workspaces.error?.code).toBe("conflict");
  });
  expect(view.result.current.repositories.pending).toEqual(retained);
  expect(view.result.current.workspaces.pending).toEqual(workspaceRequest);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.every(
        ([, args]) =>
          object(args) &&
          object(args.request) &&
          ["list", "action"].includes(String(args.request.operation)),
      ),
  ).toBe(true);
});
