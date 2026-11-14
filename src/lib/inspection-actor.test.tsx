import { afterEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { AppObservations } from "./app-observations";
import { ownerBound } from "@/test-support/owner-bound-invoke";
import { act, cleanup, renderHook } from "@testing-library/react";
import { useWorkspaceChanges } from "@/hooks/use-workspace-changes";
import { object } from "./observation-contracts";
import { validateWorkspaceRequest } from "@/generated/contracts/validators.js";
import type { ChangePage, InspectionRequest, InspectionReply } from "./changes";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const workspaceId = "00000000-0000-4000-8000-000000000001";
const flush = async () => {
  for (let i = 0; i < 40; i++) await Promise.resolve();
};
test("inspection waits for accepted baseline and follows push progress without pending polling", async () => {
  vi.useFakeTimers();
  const app = new AppObservations();
  apps.push(app);
  let phase = "queued";
  vi.mocked(invoke).mockImplementation(
    ownerBound(async () => ({
      kind: "changes",
      observation: {
        workspace_id: workspaceId,
        phase,
        page: null,
        error: null,
      },
    })),
  );
  app.connect("owner-a");
  app.inspection.configure({
    owner: "owner-a",
    workspaceId,
    active: false,
    visible: true,
  });
  await flush();
  expect(invoke).not.toHaveBeenCalled();
  const hint = (sequence: number) =>
    app.accept({
      subscription_id: "feed",
      batch_id: sequence,
      cursor: { owner: "owner-a", sequence },
      sample: {
        kind: "hints",
        watermark: sequence,
        targets: [{ kind: "inspection", workspace_id: workspaceId }],
      },
    });
  hint(1);
  await flush();
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(app.inspection.getView().observation?.phase).toBe("queued");
  await vi.advanceTimersByTimeAsync(1000);
  expect(invoke).toHaveBeenCalledTimes(1);
  phase = "running";
  hint(2);
  await flush();
  expect(invoke).toHaveBeenCalledTimes(2);
  app.inspection.release();
  vi.useRealTimers();
});

const comparison = "00000000-0000-4000-8000-000000000002";
const fileId = "00000000-0000-4000-8000-000000000003";
const otherWorkspace = "00000000-0000-4000-8000-000000000004";
const nextComparison = "00000000-0000-4000-8000-000000000005";
const apps: AppObservations[] = [];
afterEach(async () => {
  cleanup();
  for (const app of apps) {
    await app.inspection.dispose();
    app.client.clear();
  }
  apps.length = 0;
  vi.useRealTimers();
  vi.resetAllMocks();
});
function request(value: unknown): InspectionRequest {
  if (
    !object(value) ||
    !validateWorkspaceRequest(value.request) ||
    (value.request.operation !== "changes" &&
      value.request.operation !== "change_preview")
  )
    throw new Error("Unexpected request");
  return value.request;
}
function deferred<T>() {
  let resolve: (value: T) => void = () => {
    throw new Error("Deferred executor did not run");
  };
  const promise = new Promise<T>((finish) => {
    resolve = finish;
  });
  return { promise, resolve };
}
function summary(
  workspace_id = workspaceId,
  comparison_id = comparison,
): InspectionReply {
  const page: ChangePage = {
    comparison_id,
    head: "a".repeat(40),
    files: [
      {
        id: fileId,
        path: "src/a.txt",
        index_status: ".",
        worktree_status: "M",
        unsupported: null,
      },
    ],
    total: 1,
    next: null,
  };
  return {
    kind: "changes",
    observation: { workspace_id, phase: "ready", page, error: null },
  };
}
function scenario() {
  const app = new AppObservations();
  apps.push(app);
  app.connect("owner-a");
  let sequence = 0;
  const hint = (workspace_id = workspaceId) => {
    sequence++;
    app.accept({
      subscription_id: "feed",
      batch_id: sequence,
      cursor: { owner: "owner-a", sequence },
      sample: {
        kind: "hints",
        watermark: sequence,
        targets: [{ kind: "inspection", workspace_id }],
      },
    });
  };
  const configure = (workspace = workspaceId, visible = true, active = false) =>
    app.inspection.configure({
      owner: "owner-a",
      workspaceId: workspace,
      visible,
      active,
    });
  vi.mocked(invoke).mockImplementation(
    ownerBound(async (_, args) => {
      const r = request(args);
      return r.operation === "changes"
        ? summary(r.workspace_id)
        : {
            kind: "change_preview",
            observation: {
              target: r.target,
              phase: "ready",
              preview: {
                comparison_id: r.target.comparison_id,
                head: "a".repeat(40),
                rows: [],
                unsupported: null,
                limited: null,
                note: null,
              },
              error: null,
            },
          };
    }),
  );
  return { app, hint, configure };
}
test("hints before baseline and during summary reads remain owed after each captured generation", async () => {
  const s = scenario();
  const first = deferred<InspectionReply>();
  const second = deferred<InspectionReply>();
  vi.mocked(invoke)
    .mockImplementationOnce(ownerBound(() => first.promise))
    .mockImplementationOnce(ownerBound(() => second.promise));
  s.configure();
  s.hint();
  await flush();
  s.hint();
  first.resolve(summary());
  await flush();
  expect(invoke).toHaveBeenCalledTimes(2);
  s.hint();
  second.resolve(summary());
  await flush();
  expect(invoke).toHaveBeenCalledTimes(3);
  expect(s.app.inspection.getView().observation?.phase).toBe("ready");
});
test("hidden and released interests drain physical summary work without starting a preview or competitor", async () => {
  const s = scenario();
  const outstanding = deferred<InspectionReply>();
  vi.mocked(invoke).mockImplementationOnce(
    ownerBound(() => outstanding.promise),
  );
  s.hint();
  s.configure(workspaceId, true, true);
  await flush();
  s.app.inspection.select({
    id: fileId,
    path: "src/a.txt",
    index_status: ".",
    worktree_status: "M",
    unsupported: null,
  });
  s.configure(otherWorkspace, false, true);
  s.app.inspection.release();
  expect(s.app.inspection.actor.getSnapshot().matches("draining")).toBe(true);
  s.hint(otherWorkspace);
  outstanding.resolve(summary());
  await flush();
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(s.app.inspection.getView().observation).toBeUndefined();
  s.configure(otherWorkspace);
  await flush();
  expect(invoke).toHaveBeenCalledTimes(2);
  expect(s.app.inspection.getView().observation?.workspace_id).toBe(
    otherWorkspace,
  );
});
test("failed inspection stays paused across hide/remount and recovers on a newer hint", async () => {
  const s = scenario();
  s.hint();
  s.configure();
  await flush();
  vi.mocked(invoke).mockRejectedValueOnce(new Error("Read unavailable"));
  s.app.inspection.refresh();
  await flush();
  expect(s.app.inspection.getView().error).toMatch(/Read unavailable/);
  const calls = vi.mocked(invoke).mock.calls.length;
  s.app.inspection.release();
  s.configure();
  await flush();
  expect(invoke).toHaveBeenCalledTimes(calls);
  expect(s.app.inspection.getView().observation?.phase).toBe("ready");
  s.hint();
  await flush();
  expect(invoke).toHaveBeenCalledTimes(calls + 1);
  expect(s.app.inspection.getView().error).toBeUndefined();
});
test("restored file context targets the newly observed comparison after workspace and Settings-style release", async () => {
  const s = scenario();
  s.hint();
  s.configure(workspaceId, true, true);
  await flush();
  s.app.inspection.select({
    id: fileId,
    path: "src/a.txt",
    index_status: ".",
    worktree_status: "M",
    unsupported: null,
  });
  await flush();
  s.app.inspection.release();
  s.configure(otherWorkspace);
  await flush();
  vi.mocked(invoke).mockImplementationOnce(
    ownerBound(() => summary(workspaceId, nextComparison)),
  );
  s.app.inspection.release();
  s.configure(workspaceId, true, true);
  await flush();
  expect(s.app.inspection.getView().selected?.id).toBe(fileId);
  expect(s.app.inspection.getView().preview?.target).toEqual({
    workspace_id: workspaceId,
    comparison_id: nextComparison,
    file_id: fileId,
  });
  expect(request(vi.mocked(invoke).mock.calls.at(-1)?.[1])).toEqual({
    operation: "change_preview",
    target: {
      workspace_id: workspaceId,
      comparison_id: nextComparison,
      file_id: fileId,
    },
    load: true,
  });
});
test("paging pins comparison identity and explicit refresh returns to First", async () => {
  const s = scenario();
  const first = summary();
  if (first.kind !== "changes" || !first.observation.page)
    throw new Error("Fixture summary");
  first.observation.page.next = fileId;
  first.observation.page.total = 2;
  vi.mocked(invoke).mockImplementationOnce(ownerBound(() => first));
  s.hint();
  s.configure();
  await flush();
  s.app.inspection.next();
  await flush();
  expect(request(vi.mocked(invoke).mock.calls[1][1])).toMatchObject({
    operation: "changes",
    comparison_id: comparison,
    after: fileId,
    refresh: false,
  });
  s.app.inspection.refresh();
  await flush();
  expect(request(vi.mocked(invoke).mock.calls.at(-1)?.[1])).toMatchObject({
    operation: "changes",
    comparison_id: null,
    after: null,
    refresh: true,
  });
});
test("unrelated presentation and freshness updates do not render inspection consumers", async () => {
  const s = scenario();
  s.hint();
  let renders = 0;
  const { result } = renderHook(() => {
    renders++;
    return useWorkspaceChanges(
      "owner-a",
      workspaceId,
      false,
      true,
      s.app.inspection,
    );
  });
  await act(async () => {});
  const before = renders;
  await act(async () => {
    s.app.presentation.setState({ repository: "another-repository" });
    s.app.freshness.mark("repositories");
    s.app.freshness.accept("owner-a");
    await flush();
  });
  expect(renders).toBe(before);
  expect(result.current.observation?.phase).toBe("ready");
});

test("hiding during the first physical read does not convert disposal into a paused application failure", async () => {
  const s = scenario();
  const outstanding = deferred<InspectionReply>();
  vi.mocked(invoke).mockImplementationOnce(
    ownerBound(() => outstanding.promise),
  );
  s.hint();
  s.configure();
  await flush();
  s.app.inspection.release();
  outstanding.resolve(summary());
  await flush();
  expect(s.app.inspection.getView().error).toBeUndefined();
  s.configure();
  await flush();
  expect(invoke).toHaveBeenCalledTimes(2);
  expect(s.app.inspection.getView().observation?.phase).toBe("ready");
});

test("paging during drain cannot downgrade a queued explicit refresh", async () => {
  const s = scenario();
  const outstanding = deferred<InspectionReply>();
  vi.mocked(invoke).mockImplementationOnce(
    ownerBound(() => outstanding.promise),
  );
  s.hint();
  s.configure();
  await flush();
  s.app.inspection.refresh();
  s.app.inspection.first();
  expect(invoke).toHaveBeenCalledTimes(1);
  outstanding.resolve(summary());
  await flush();
  expect(request(vi.mocked(invoke).mock.calls.at(-1)?.[1])).toMatchObject({
    operation: "changes",
    refresh: true,
    comparison_id: null,
    after: null,
  });
});

test("unrelated accepted batches cannot postpone the visible two-second external Git cadence", async () => {
  vi.useFakeTimers();
  const s = scenario();
  s.hint();
  s.configure();
  await flush();
  expect(invoke).toHaveBeenCalledTimes(1);
  for (let tick = 0; tick < 4; tick++) {
    await vi.advanceTimersByTimeAsync(1000);
    s.app.freshness.mark("repositories");
    s.app.freshness.accept("owner-a");
    await flush();
  }
  expect(invoke).toHaveBeenCalledTimes(3);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.slice(1)
      .map(([, args]) => request(args)),
  ).toEqual([
    {
      operation: "changes",
      workspace_id: workspaceId,
      comparison_id: null,
      after: null,
      refresh: true,
    },
    {
      operation: "changes",
      workspace_id: workspaceId,
      comparison_id: null,
      after: null,
      refresh: true,
    },
  ]);
});

test.each(["pending", "rejected"])(
  "%s summary pauses with a retained selected preview",
  async (outcome) => {
    vi.useFakeTimers();
    const s = scenario();
    s.hint();
    s.configure(workspaceId, true, true);
    await flush();
    s.app.inspection.select({
      id: fileId,
      path: "src/a.txt",
      index_status: ".",
      worktree_status: "M",
      unsupported: null,
    });
    await flush();
    const original = vi.mocked(invoke).getMockImplementation();
    if (!original) throw new Error("Scenario IPC double is missing");
    const before = vi.mocked(invoke).mock.calls.length;
    vi.mocked(invoke).mockImplementation(
      ownerBound(async () => {
        if (outcome === "rejected") throw new Error("Summary read unavailable");
        return {
          kind: "changes",
          observation: {
            workspace_id: workspaceId,
            phase: "running",
            page: null,
            error: null,
          },
        };
      }),
    );
    s.hint();
    await flush();
    expect(vi.mocked(invoke).mock.calls.length - before).toBe(1);
    expect(s.app.inspection.getView().preview?.phase).toBe("ready");
    expect(s.app.inspection.getView().observation?.phase).toBe(
      outcome === "rejected" ? "ready" : "running",
    );
    if (outcome === "rejected")
      expect(s.app.inspection.getView().error).toContain(
        "Summary read unavailable",
      );
    await vi.advanceTimersByTimeAsync(1000);
    expect(vi.mocked(invoke).mock.calls.length - before).toBe(1);
    vi.mocked(invoke).mockImplementation(original);
    s.hint();
    await flush();
    expect(s.app.inspection.getView().observation?.phase).toBe("ready");
    expect(s.app.inspection.getView().preview?.phase).toBe("ready");
    s.app.inspection.release();
  },
);
