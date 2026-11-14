import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useWorkspaces } from "./test-support/feature-hooks";
import { ObservationCoordinator } from "./lib/observation-coordinator";
import { ownerBound } from "./test-support/owner-bound-invoke";
import type { WorkspaceRequest } from "./lib/workspaces";
import { useRepositories } from "./test-support/feature-hooks";
import { useSupervisorConnection } from "./hooks/use-supervisor-connection";
import { useWorkspaceChanges } from "./hooks/use-workspace-changes";
import { AppObservations } from "./lib/app-observations";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => {
  cleanup();
  localStorage.clear();
  vi.clearAllMocks();
});

test("workspace list fan-out waits behind two physical reads while explicit creation bypasses the queue", async () => {
  const reads = new ObservationCoordinator();
  const ids = Array.from(
    { length: 32 },
    (_, index) =>
      `00000000-0000-4000-8000-${String(index + 1).padStart(12, "0")}`,
  );
  const releases: (() => void)[] = [];
  const requests: WorkspaceRequest[] = [];
  vi.mocked(invoke).mockImplementation(
    ownerBound(async (_, args) => {
      const request = (args as { request: WorkspaceRequest }).request;
      requests.push(request);
      if (request.operation === "create")
        throw {
          kind: "transport",
          error: {
            code: "busy",
            delivery: "not_sent",
            message: "Intent lane occupied",
          },
        };
      await new Promise<void>((resolve) => releases.push(resolve));
      return { kind: "workspaces", page: { workspaces: [], next: null } };
    }),
  );
  const first = renderHook(() =>
    useWorkspaces("owner-a", ids[0], ids.join(","), null, reads),
  );
  await waitFor(() => expect(requests.length).toBeGreaterThan(0));
  expect(requests).toHaveLength(2);
  await act(async () => {
    await first.result.current.create(ids[0]);
  });
  expect(
    requests.filter((request) => request.operation === "create"),
  ).toHaveLength(1);
  first.unmount();
  await act(async () => {
    for (const release of releases) release();
  });
  expect(requests).toHaveLength(3); // Disposed queued lists never enter IPC.
});

test("repository, workspace and inspection share a budget while status remains independent", async () => {
  const reads = new ObservationCoordinator();
  const app = new AppObservations(reads);
  const reader = app.inspection;
  const id = "00000000-0000-4000-8000-000000000001";
  const releases: (() => void)[] = [];
  const requests: string[] = [];
  vi.mocked(invoke).mockImplementation(
    ownerBound(async (command, args) => {
      if (command === "desktop_pid") return 42;
      if (command === "supervisor_status")
        return {
          status: {
            state: "ready",
            supervisor_pid: 43,
            application_version: "0.1.0",
          },
          cursor: { owner: "owner-a", revision: 1 },
          reset: true,
        };
      const { request } = args as { request: { operation: string } };
      requests.push(`${command}/${request.operation}`);
      await new Promise<void>((resolve) => releases.push(resolve));
      if (command === "repository_request")
        return { kind: "repositories", page: { repositories: [], next: null } };
      if (request.operation === "changes")
        return {
          kind: "changes",
          observation: {
            workspace_id: id,
            phase: "idle",
            page: null,
            error: null,
          },
        };
      return { kind: "workspaces", page: { workspaces: [], next: null } };
    }),
  );
  const view = renderHook(() => {
    const connection = useSupervisorConnection();
    useRepositories("owner-a", app);
    useWorkspaces("owner-a", id, id, null, app);
    useWorkspaceChanges("owner-a", id, false, true, reader);
    return connection;
  });
  await waitFor(() =>
    expect(view.result.current.connection.state).toBe("connected"),
  );
  expect(requests).toHaveLength(2);
  expect(
    app.client
      .getQueryCache()
      .findAll({ queryKey: ["inspection"] })
      .some((query) => query.state.fetchStatus === "fetching"),
  ).toBe(true);
  view.unmount();
  await act(async () => {
    for (const release of releases) release();
  });
  expect(requests).toHaveLength(2);
  await reader.dispose();
  app.client.clear();
});

test("navigation drops queued old targets while physically stalled reads drain", async () => {
  const reads = new ObservationCoordinator();
  const ids = Array.from(
    { length: 4 },
    (_, index) =>
      `00000000-0000-4000-8000-${String(index + 1).padStart(12, "0")}`,
  );
  const requests: WorkspaceRequest[] = [];
  const releases: (() => void)[] = [];
  vi.mocked(invoke).mockImplementation(
    ownerBound(async (_, args) => {
      requests.push((args as { request: WorkspaceRequest }).request);
      await new Promise<void>((resolve) => releases.push(resolve));
      return { kind: "workspaces", page: { workspaces: [], next: null } };
    }),
  );
  const view = renderHook(
    ({ owner, targets }) =>
      useWorkspaces(owner, targets[0], targets.join(","), null, reads),
    {
      initialProps: { owner: "owner-a", targets: ids },
    },
  );
  await waitFor(() => expect(requests).toHaveLength(2));
  view.rerender({ owner: "owner-b", targets: [ids[3]] });
  expect(requests).toHaveLength(2);
  await act(async () => {
    releases[0]();
  });
  expect(requests).toHaveLength(3);
  expect(requests[2]).toMatchObject({
    operation: "list",
    repository_id: ids[3],
  });
  expect(view.result.current.error).toBeUndefined();
  view.unmount();
  await act(async () => {
    for (const release of releases) release();
  });
  expect(requests).toHaveLength(3);
});

test("a failed desktop identity read cannot release a still-running status request", async () => {
  let finish!: (value: unknown) => void;
  let statusReads = 0;
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "desktop_pid") throw "Desktop identity failed";
    statusReads++;
    return new Promise((resolve) => {
      finish = resolve;
    });
  });
  const view = renderHook(() => useSupervisorConnection());
  await act(async () => {});
  await act(async () => {
    await view.result.current.refreshStatus();
  });
  expect(statusReads).toBe(1);
  expect(view.result.current.connection.state).toBe("checking");
  await act(async () => {
    finish({
      status: {
        state: "ready",
        supervisor_pid: 43,
        application_version: "0.1.0",
      },
      cursor: { owner: "owner-a", revision: 1 },
      reset: true,
    });
  });
  expect(view.result.current.connection).toMatchObject({
    state: "error",
    error: "Desktop identity failed",
  });
});

test("typed status admission errors retain readable transport feedback", async () => {
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "desktop_pid") return 42;
    throw {
      kind: "transport",
      error: {
        code: "busy",
        delivery: "not_sent",
        message: "Status lane occupied",
      },
    };
  });
  const view = renderHook(() => useSupervisorConnection());
  await waitFor(() =>
    expect(view.result.current.connection).toMatchObject({
      state: "error",
      error: "Status lane occupied",
    }),
  );
});
