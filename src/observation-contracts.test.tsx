import { afterEach, expect, test, vi } from "vitest";
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import corpusText from "../contracts/observations-v13.json?raw";
import {
  decodeOwnedReply,
  decodeRequestFailure,
  object,
} from "./lib/observation-contracts";
import {
  decodeRepositoryReply,
  repositoryRequest,
  type RepositoryRequest,
} from "./lib/repositories";
import {
  decodeWorkspaceReply,
  workspaceRequest,
  type WorkspaceRequest,
} from "./lib/workspaces";
import { useRepositories } from "./test-support/feature-hooks";
import { useSupervisorConnection } from "./hooks/use-supervisor-connection";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => {
  cleanup();
  localStorage.clear();
  vi.resetAllMocks();
});
const corpus: {
  protocol: number;
  owner: string;
  snapshot: unknown;
  desktop_pid: unknown;
  repositories: {
    request: RepositoryRequest;
    reply: { owner: string; result: Record<string, unknown> };
  }[];
  workspaces: {
    request: WorkspaceRequest;
    reply: { owner: string; result: Record<string, unknown> };
  }[];
  failures: unknown[];
} = JSON.parse(corpusText);
import { decodeDesktopPid, decodeSnapshot } from "./lib/observation-contracts";

const snapshot = {
  status: { state: "ready", supervisor_pid: 41, application_version: "0.1.0" },
  cursor: { owner: "owner-41", revision: 1 },
  reset: true,
};

test("connection observations are decoded before they become presentation truth", () => {
  expect(decodeSnapshot(snapshot)).toEqual(snapshot);
  expect(decodeDesktopPid(42)).toBe(42);
  for (const value of [
    null,
    [],
    {},
    { ...snapshot, reset: "true" },
    { ...snapshot, cursor: { owner: "", revision: 1 } },
    {
      ...snapshot,
      cursor: { owner: "owner", revision: Number.MAX_SAFE_INTEGER + 1 },
    },
    { ...snapshot, status: { ...snapshot.status, supervisor_pid: 0 } },
    { ...snapshot, status: { ...snapshot.status, state: "running" } },
  ]) {
    expect(() => decodeSnapshot(value)).toThrow(/Invalid supervisor snapshot/);
  }
  for (const value of [null, "42", 0, -1, 1.5, 2 ** 32, NaN])
    expect(() => decodeDesktopPid(value)).toThrow(/Invalid desktop identity/);
});

test("the shared application corpus passes production TypeScript decoders", () => {
  expect(corpus.protocol).toBe(13);
  expect(decodeSnapshot(corpus.snapshot).cursor.owner).toBe(corpus.owner);
  expect(decodeDesktopPid(corpus.desktop_pid)).toBe(42);
  for (const { request, reply } of corpus.repositories) {
    if ("Err" in reply.result) {
      try {
        decodeOwnedReply(corpus.owner, reply, (v) =>
          decodeRepositoryReply(request, v),
        );
        expect.fail("domain failure accepted");
      } catch (error) {
        expect(error).toMatchObject({
          code: "storage",
          source: "domain",
          delivery: "unknown",
        });
      }
    } else
      expect(
        decodeOwnedReply(corpus.owner, reply, (v) =>
          decodeRepositoryReply(request, v),
        ),
      ).toEqual(reply.result.Ok);
  }
  for (const { request, reply } of corpus.workspaces)
    expect(
      decodeOwnedReply(corpus.owner, reply, (v) =>
        decodeWorkspaceReply(request, v),
      ),
    ).toEqual(reply.result.Ok);
  for (const failure of corpus.failures) {
    if (!object(failure) || !object(failure.error))
      throw new Error("Invalid corpus failure");
    expect(decodeRequestFailure(failure)).toMatchObject({
      code: failure.error.code,
      message: failure.error.message,
      delivery:
        failure.kind === "rejected" ? "not_sent" : failure.error.delivery,
    });
  }
});

test("provider-independent workspace observations retain allocation, recovery and target checks", () => {
  const id = "00000000-0000-0000-0000-000000000003";
  const repositoryId = "00000000-0000-0000-0000-000000000001";
  const actionId = "00000000-0000-0000-0000-000000000002";
  const error = { code: "git", message: "Worktree failed; files retained" };
  for (const status of [
    { state: "queued" },
    { state: "running", stage: "creating_worktree" },
    { state: "ready" },
    { state: "failed", error },
    { state: "uncertain", error },
  ]) {
    const workspace = {
      id,
      repository_id: repositoryId,
      creation_action_id: actionId,
      name: "kyoto",
      branch: "workspace/kyoto",
      path: "/private/state/workspaces/repo/kyoto/checkout",
      legacy: false,
      base: { reference: "refs/heads/main", commit: "a".repeat(40) },
      artifacts: { branch: "owned", folder: "reserved" },
      status,
      cleanup_unconfirmed: status.state === "uncertain",
    };
    const reply = { kind: "workspace", workspace };
    expect(decodeWorkspaceReply({ operation: "get", id }, reply)).toEqual(
      reply,
    );
    const page = {
      kind: "workspaces",
      page: { workspaces: [workspace], next: null },
    };
    expect(
      decodeWorkspaceReply(
        { operation: "list", repository_id: repositoryId, after: null },
        page,
      ),
    ).toEqual(page);
    const action = {
      kind: "action",
      action: {
        operation: "create",
        action_id: actionId,
        repository_id: repositoryId,
      },
      workspace,
    };
    expect(
      decodeWorkspaceReply(
        { operation: "action", action_id: actionId },
        action,
      ),
    ).toEqual(action);
    for (const invalid of [
      { ...workspace, repository_id: actionId },
      { ...workspace, path: "/state/../checkout" },
      { ...workspace, name: null },
    ]) {
      expect(() =>
        decodeWorkspaceReply(
          { operation: "list", repository_id: repositoryId, after: null },
          { ...page, page: { workspaces: [invalid], next: null } },
        ),
      ).toThrow(/Invalid workspace observation/);
    }
  }
});

test("owner and result-variant checks happen before feature decoding, including domain errors", () => {
  const decode = vi.fn();
  for (const reply of [
    { ...corpus.repositories[0].reply, owner: "owner-b" },
    { ...corpus.repositories[3].reply, owner: "owner-b" },
    {
      owner: corpus.owner,
      result: { Ok: {}, Err: { code: "capacity", message: "Both" } },
    },
    { owner: corpus.owner, result: {} },
    { owner: corpus.owner, result: { Ok: "x".repeat(64 * 1024) } },
  ])
    expect(() => decodeOwnedReply(corpus.owner, reply, decode)).toThrow();
  expect(decode).not.toHaveBeenCalled();
  expect(
    decodeRequestFailure({
      kind: "transport",
      error: {
        code: "reply_unavailable",
        delivery: "not_sent",
        message: "Impossible",
      },
    }),
  ).toMatchObject({ delivery: "unknown", code: "reply_unavailable" });
  expect(
    decodeRequestFailure({ code: "capacity", message: "Unwrapped" }),
  ).toMatchObject({ delivery: "unknown", code: "reply_unavailable" });
});

test("replacement replies before the next status refresh cannot overwrite an owner's observations", async () => {
  const fixture = corpus.repositories[0];
  const listed = decodeOwnedReply(corpus.owner, fixture.reply, (value) =>
    decodeRepositoryReply(fixture.request, value),
  );
  if (listed.kind !== "repositories" || !listed.page.repositories[0])
    throw new Error("Missing corpus repository");
  const selected = listed.page.repositories[0];
  let replacement = false;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const reply =
      object(args) && object(args.request) && args.request.operation === "get"
        ? {
            owner: corpus.owner,
            result: { Ok: { kind: "repository", repository: selected } },
          }
        : fixture.reply;
    return replacement ? { ...reply, owner: "owner-b" } : reply;
  });
  const { result } = renderHook(() =>
    useRepositories(corpus.owner, undefined, selected.id),
  );
  await waitFor(() => expect(result.current.selected?.name).toBe("one"));
  replacement = true;
  await act(async () => {
    await result.current.refresh(false);
  });
  expect(result.current.error?.code).toBe("reply_unavailable");
  expect(result.current.selected?.name).toBe("one");
  expect(result.current.page).toEqual(
    (fixture.reply.result.Ok as { page: unknown }).page,
  );
  for (const [command, args] of vi.mocked(invoke).mock.calls) {
    expect(command).toBe("repository_request");
    expect(args).toMatchObject({ expectedOwner: corpus.owner });
  }
});

test("feature decoders reject wrong kind, full payload mismatch and impossible variants", async () => {
  const registration = corpus.repositories[1];
  const value = registration.reply.result.Ok;
  if (!object(value) || !object(value.registration))
    throw new Error("Missing corpus registration");
  const request = registration.request;
  for (const result of [
    { ...value, kind: "action" },
    {
      ...value,
      registration: { ...value.registration, create_initial_workspace: true },
    },
    {
      ...value,
      registration: {
        ...value.registration,
        state: "pending",
        repository_id: value.registration.repository_id,
      },
    },
    {
      ...value,
      registration: {
        ...value.registration,
        error: { code: "git", message: "Contradictory" },
      },
    },
  ])
    expect(() => decodeRepositoryReply(request, result)).toThrow();
  const workspace = corpus.workspaces[0];
  expect(() =>
    decodeWorkspaceReply(workspace.request, {
      kind: "action",
      action: null,
      workspace: null,
    }),
  ).toThrow();
  const wrongTarget = {
    operation: "get",
    id: "00000000-0000-0000-0000-000000000099",
  } as const;
  const actionReply = workspace.reply.result.Ok;
  if (!object(actionReply)) throw new Error("Missing corpus action");
  expect(() =>
    decodeWorkspaceReply(wrongTarget, {
      kind: "workspace",
      workspace: actionReply.workspace,
    }),
  ).toThrow();
  vi.mocked(invoke).mockResolvedValue(registration.reply);
  await expect(
    repositoryRequest(corpus.owner, {
      operation: "action",
      action_id: "00000000-0000-0000-0000-000000000002",
    }),
  ).rejects.toThrow();
  vi.mocked(invoke).mockResolvedValue({ ...workspace.reply, owner: "owner-b" });
  await expect(
    workspaceRequest(corpus.owner, workspace.request),
  ).rejects.toThrow(/serving owner/);
});

test.each(["snapshot", "desktop"])(
  "malformed %s observations leave connection failure visible and preserve the last good snapshot",
  async (malformed) => {
    let invalid = false;
    vi.mocked(invoke).mockImplementation(async (command) =>
      command === "supervisor_status"
        ? invalid && malformed === "snapshot"
          ? { ...snapshot, reset: "true" }
          : snapshot
        : invalid && malformed === "desktop"
          ? "42"
          : 42,
    );
    const { result } = renderHook(useSupervisorConnection);
    await waitFor(() =>
      expect(result.current.connection.state).toBe("connected"),
    );
    const previous = result.current.connection.observation;
    invalid = true;
    await act(async () => {
      await result.current.refreshStatus();
    });
    expect(result.current.connection.state).toBe("error");
    expect(result.current.connection.observation).toBe(previous);
    expect(vi.mocked(invoke).mock.calls.map(([command]) => command)).toEqual([
      "supervisor_status",
      "desktop_pid",
      "supervisor_status",
      "desktop_pid",
    ]);
  },
);

test.each(["queued", "running"])(
  "inspection retries may retain earlier failure evidence while %s",
  (phase) => {
    for (const fixture of corpus.workspaces.slice(2)) {
      const value = fixture.reply.result.Ok;
      if (!object(value) || !object(value.observation))
        throw new Error("Missing corpus inspection");
      const retry = {
        ...value,
        observation: {
          ...value.observation,
          phase,
          error: { code: "git", message: "Previous inspection failed" },
        },
      };
      expect(decodeWorkspaceReply(fixture.request, retry)).toEqual(retry);
      expect(() =>
        decodeWorkspaceReply(fixture.request, {
          ...retry,
          observation: { ...retry.observation, phase: "ready" },
        }),
      ).toThrow();
      expect(() =>
        decodeWorkspaceReply(fixture.request, {
          ...retry,
          observation: { ...retry.observation, phase: "failed", error: null },
        }),
      ).toThrow();
    }
  },
);
