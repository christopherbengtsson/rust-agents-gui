import {
  act,
  cleanup,
  fireEvent,
  render,
  renderHook,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { workspaceRequest } from "./lib/workspaces";
import { useWorkspaceChanges } from "./hooks/use-workspace-changes";
import { WorkspaceInspection } from "./components/blocks/workspace-changes";
import { WorkspacePanel } from "./components/blocks/workspace-panel";
import { WorkspaceChangesReader } from "./lib/workspace-changes-reader";
import { Activity, StrictMode } from "react";
import { useAppNavigation } from "./use-app-navigation";
import { SettingsScreen } from "./components/blocks/settings-screen";
import { SidebarProvider } from "./components/ui/sidebar";
import type { Repositories } from "./hooks/use-repositories";
import type { Workspaces } from "./hooks/use-workspaces";
import type { Workspace } from "./lib/workspaces";
import type { ChangePage, FilePreview, InspectionRequest } from "./lib/changes";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const resizeCallbacks = new Map<Element, (width: number) => void>();
const workspace_id = "00000000-0000-4000-8000-000000000001";
const comparison_id = "00000000-0000-4000-8000-000000000002";
const file_id = "00000000-0000-4000-8000-000000000003";
const other = "00000000-0000-4000-8000-000000000004";
const page: ChangePage = {
  comparison_id,
  head: "a".repeat(40),
  files: [
    {
      id: file_id,
      path: "src/file.txt",
      index_status: ".",
      worktree_status: "M",
      unsupported: null,
    },
  ],
  total: 1,
  next: null,
};
const preview: FilePreview = {
  comparison_id,
  head: page.head,
  rows: [{ kind: "added", old_line: null, new_line: 7, text: "new content" }],
  unsupported: null,
  limited: null,
  note: null,
};
const target = { workspace_id, comparison_id, file_id };
function changes(
  phase = "ready",
  data: ChangePage | null = page,
  error: unknown = null,
) {
  return {
    kind: "changes",
    observation: { workspace_id, phase, page: data, error },
  };
}
function request(args: unknown) {
  return (args as { request: InspectionRequest }).request;
}

function InspectionNavigation({ reader }: { reader: WorkspaceChangesReader }) {
  const {
    screen: destination,
    settingsTrigger,
    openSettings,
    visit,
    canRestoreFocus,
    settingsHeading,
    back,
  } = useAppNavigation();
  return (
    <SidebarProvider>
      <Activity mode={destination === "workbench" ? "visible" : "hidden"}>
        <button ref={settingsTrigger} onClick={openSettings}>
          Settings
        </button>
        <WorkspaceInspection
          owner="owner"
          workspaceId={workspace_id}
          reader={reader}
          disclosureKey={visit}
          canRestoreFocus={canRestoreFocus}
        >
          <p>Details</p>
        </WorkspaceInspection>
      </Activity>
      {destination === "settings" && (
        <SettingsScreen headingRef={settingsHeading} onBack={back} />
      )}
    </SidebarProvider>
  );
}
beforeEach(() => {
  Object.defineProperty(document, "visibilityState", {
    configurable: true,
    value: "visible",
  });
  Object.defineProperty(Element.prototype, "getAnimations", {
    configurable: true,
    value: () => [],
  });
  vi.stubGlobal(
    "ResizeObserver",
    class implements ResizeObserver {
      private targets = new Set<Element>();
      constructor(private callback: ResizeObserverCallback) {}
      observe(target: Element) {
        this.targets.add(target);
        resizeCallbacks.set(target, (width) => {
          Object.defineProperty(target, "offsetWidth", {
            configurable: true,
            value: width,
          });
          const size = [{ inlineSize: width, blockSize: 600 }];
          this.callback(
            [
              {
                target,
                borderBoxSize: size,
                contentBoxSize: size,
                devicePixelContentBoxSize: size,
                contentRect: new DOMRect(0, 0, width, 600),
              },
            ],
            this,
          );
        });
      }
      unobserve(target: Element) {
        resizeCallbacks.delete(target);
      }
      disconnect() {
        for (const target of this.targets) resizeCallbacks.delete(target);
      }
    },
  );
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: vi.fn(() => ({
      matches: false,
      addEventListener() {},
      removeEventListener() {},
    })),
  });
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const r = request(args);
    return r.operation === "changes"
      ? changes()
      : {
          kind: "change_preview",
          observation: {
            target: r.target,
            phase: "ready",
            preview,
            error: null,
          },
        };
  });
});
afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.unstubAllGlobals();
  vi.resetAllMocks();
  vi.restoreAllMocks();
});

test("inspection validates exact workspace/comparison/file targets and display bounds", async () => {
  const r = {
    operation: "changes" as const,
    workspace_id,
    refresh: false,
    comparison_id: null,
    after: null,
  };
  vi.mocked(invoke).mockResolvedValue({
    ...changes(),
    observation: { ...changes().observation, workspace_id: other },
  });
  await expect(workspaceRequest(r)).rejects.toThrow(
    "Invalid workspace observation",
  );
  vi.mocked(invoke).mockResolvedValue(
    changes("ready", { ...page, files: Array(33).fill(page.files[0]) }),
  );
  await expect(workspaceRequest(r)).rejects.toThrow();
  vi.mocked(invoke).mockResolvedValue({
    kind: "change_preview",
    observation: {
      target: { ...target, file_id: other },
      phase: "ready",
      preview,
      error: null,
    },
  });
  await expect(
    workspaceRequest({ operation: "change_preview", target, load: true }),
  ).rejects.toThrow();
  vi.mocked(invoke).mockResolvedValue({
    kind: "change_preview",
    observation: {
      target,
      phase: "ready",
      preview: { ...preview, rows: Array(201).fill(preview.rows[0]) },
      error: null,
    },
  });
  await expect(
    workspaceRequest({ operation: "change_preview", target, load: true }),
  ).rejects.toThrow();
});

test("manual refresh submits once; observation polling does not requeue work", async () => {
  vi.useFakeTimers();
  let submitted = 0;
  let observed = 0;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const r = request(args);
    if (r.operation !== "changes") throw new Error("unexpected preview");
    if (r.refresh) {
      submitted++;
      return changes("queued", null);
    }
    observed++;
    return observed === 1
      ? changes("idle", null)
      : observed === 2
        ? changes("running", null)
        : changes();
  });
  const { result } = renderHook(() =>
    useWorkspaceChanges("owner", workspace_id, false),
  );
  await act(async () => {});
  await act(async () => {
    await result.current.refresh();
  });
  await act(async () => {
    await vi.advanceTimersByTimeAsync(250);
  });
  await act(async () => {
    await vi.advanceTimersByTimeAsync(250);
  });
  expect(submitted).toBe(1);
  expect(result.current.observation?.phase).toBe("ready");
});

test("changed owner discards late observations and reconnect only reads", async () => {
  let finish: ((value: unknown) => void) | undefined;
  vi.mocked(invoke).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  const { result, rerender } = renderHook(
    ({ owner }) => useWorkspaceChanges(owner, workspace_id, false),
    { initialProps: { owner: "old" } },
  );
  rerender({ owner: "new" });
  expect(invoke).toHaveBeenCalledTimes(1); // The obsolete physical read still owns the lane.
  await act(async () => finish?.(changes("idle", null)));
  await waitFor(() => expect(result.current.observation?.phase).toBe("ready"));
  expect(result.current.observation?.phase).toBe("ready");
  expect(
    vi
      .mocked(invoke)
      .mock.calls.every(
        ([, args]) =>
          request(args).operation === "changes" &&
          !(request(args) as { refresh: boolean }).refresh,
      ),
  ).toBe(true);
});

test("failed refresh retains the comparison and preview; tab switching unmounts diff and preserves selection", async () => {
  render(
    <WorkspaceInspection
      reader={new WorkspaceChangesReader()}
      owner="owner"
      workspaceId={workspace_id}
    >
      <p>Workspace details remain reachable</p>
    </WorkspaceInspection>,
  );
  await screen.findByRole("button", { name: /src\/file.txt/ });
  fireEvent.click(screen.getByRole("button", { name: /src\/file.txt/ }));
  await screen.findByText("new content");
  const viewport = screen
    .getByLabelText("Unified file diff")
    .closest("[data-slot=scroll-area-viewport]") as HTMLElement;
  viewport.scrollTop = 37;
  fireEvent.scroll(viewport);
  fireEvent.click(screen.getByRole("tab", { name: "Workspace" }));
  await waitFor(() => expect(screen.queryByText("new content")).toBeNull());
  expect(screen.getByText("Workspace details remain reachable")).toBeTruthy();
  fireEvent.click(screen.getByRole("tab", { name: "Changes" }));
  await screen.findByText("new content");
  expect(
    screen
      .getByLabelText("Unified file diff")
      .closest("[data-slot=scroll-area-viewport]")?.scrollTop,
  ).toBe(37);
  vi.mocked(invoke).mockImplementation(async (_, args) =>
    request(args).operation === "changes"
      ? changes("failed", page, {
          code: "git",
          message: "Injected Git failure",
        })
      : {
          kind: "change_preview",
          observation: { target, phase: "ready", preview, error: null },
        },
  );
  fireEvent.click(screen.getByRole("button", { name: "Refresh changes" }));
  await screen.findByText("Injected Git failure");
  expect(screen.getByText(/Showing the last-good comparison/)).toBeTruthy();
  expect(screen.getByText("new content")).toBeTruthy();
});

test("narrow-window Sheet selects the single Changes tab and returns focus", async () => {
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: vi.fn(() => ({
      matches: true,
      addEventListener() {},
      removeEventListener() {},
    })),
  });
  render(
    <WorkspaceInspection
      reader={new WorkspaceChangesReader()}
      owner="owner"
      workspaceId={workspace_id}
    >
      <p>Workspace details</p>
    </WorkspaceInspection>,
  );
  expect(invoke).not.toHaveBeenCalled(); // Both changes surfaces are initially hidden.
  expect(
    screen.queryByRole("separator", { name: "Resize changed files" }),
  ).toBeNull();
  const trigger = screen.getByRole("button", { name: "Changed files" });
  fireEvent.click(trigger);
  await screen.findByRole("dialog");
  fireEvent.click(screen.getByRole("button", { name: /src\/file.txt/ }));
  await screen.findByText("new content");
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(
    screen.getByRole("tab", { name: "Changes" }).getAttribute("aria-selected"),
  ).toBe("true");
  await waitFor(() => expect(document.activeElement).toBe(trigger));
});

test("a removed selection remains named and an old preview is never replaced by another file", async () => {
  const { result } = renderHook(() =>
    useWorkspaceChanges("owner", workspace_id, true),
  );
  await waitFor(() => expect(result.current.observation?.phase).toBe("ready"));
  await act(async () => result.current.select(page.files[0]));
  await waitFor(() => expect(result.current.preview?.phase).toBe("ready"));
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    if (request(args).operation === "changes")
      return changes("ready", {
        ...page,
        comparison_id: other,
        files: [],
        total: 0,
      });
    throw {
      code: "not_found",
      message: "The selected file no longer appears in this comparison.",
    };
  });
  await act(async () => {
    await result.current.refresh();
  });
  expect(result.current.selected?.id).toBe(file_id);
  expect(result.current.error).toMatch(/no longer appears/);
  expect(result.current.preview?.preview?.comparison_id).toBe(comparison_id);
});

test("visible summaries refresh every two seconds without overlapping a slow sequence", async () => {
  vi.useFakeTimers();
  let finish: ((value: unknown) => void) | undefined;
  const { result } = renderHook(() =>
    useWorkspaceChanges("owner", workspace_id, false),
  );
  await act(async () => {});
  vi.mocked(invoke).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  await act(async () => {
    await vi.advanceTimersByTimeAsync(2000);
  });
  expect(vi.mocked(invoke).mock.calls.map(([, args]) => request(args))).toEqual(
    [
      {
        operation: "changes",
        workspace_id,
        refresh: false,
        comparison_id: null,
        after: null,
      },
      {
        operation: "changes",
        workspace_id,
        refresh: true,
        comparison_id: null,
        after: null,
      },
    ],
  );
  await act(async () => {
    await vi.advanceTimersByTimeAsync(10000);
  });
  expect(invoke).toHaveBeenCalledTimes(2);
  await act(async () => finish?.(changes()));
  expect(result.current.observation?.phase).toBe("ready");
});

test("collapsed changes surfaces pause automatic reads until shown again", async () => {
  vi.useFakeTimers();
  render(
    <WorkspaceInspection
      reader={new WorkspaceChangesReader()}
      owner="owner"
      workspaceId={workspace_id}
    >
      <p>Details</p>
    </WorkspaceInspection>,
  );
  await act(async () => {});
  fireEvent.click(screen.getByRole("button", { name: "Hide changed files" }));
  await act(async () => {
    Object.defineProperty(
      document.getElementById("inspection-main")!,
      "offsetWidth",
      { configurable: true, value: 680 },
    );
    resizeCallbacks.get(document.getElementById("inspection-files")!)?.(0);
  });
  const calls = vi.mocked(invoke).mock.calls.length;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(6000);
  });
  expect(invoke).toHaveBeenCalledTimes(calls);
  fireEvent.click(screen.getByRole("tab", { name: "Changes" }));
  await act(async () => {});
  expect(invoke).toHaveBeenCalledTimes(calls + 1);
  expect(request(vi.mocked(invoke).mock.calls.at(-1)?.[1])).toMatchObject({
    refresh: false,
  });
});

test("document visibility, disconnect and unmount stop timers; reconnect only observes", async () => {
  vi.useFakeTimers();
  const { rerender, unmount } = renderHook(
    ({ owner }) => useWorkspaceChanges(owner, workspace_id, false),
    { initialProps: { owner: "owner" as string | undefined } },
  );
  await act(async () => {});
  Object.defineProperty(document, "visibilityState", {
    configurable: true,
    value: "hidden",
  });
  act(() => document.dispatchEvent(new Event("visibilitychange")));
  await act(async () => {
    await vi.advanceTimersByTimeAsync(10000);
  });
  expect(invoke).toHaveBeenCalledTimes(1);
  Object.defineProperty(document, "visibilityState", {
    configurable: true,
    value: "visible",
  });
  act(() => document.dispatchEvent(new Event("visibilitychange")));
  await act(async () => {});
  expect(invoke).toHaveBeenCalledTimes(2);
  expect(request(vi.mocked(invoke).mock.calls[1][1])).toMatchObject({
    refresh: false,
  });
  rerender({ owner: undefined });
  await act(async () => {
    await vi.advanceTimersByTimeAsync(10000);
  });
  expect(invoke).toHaveBeenCalledTimes(2);
  rerender({ owner: "new-owner" });
  await act(async () => {});
  expect(invoke).toHaveBeenCalledTimes(3);
  unmount();
  await vi.advanceTimersByTimeAsync(10000);
  expect(invoke).toHaveBeenCalledTimes(3);
});

test("same-status edits refresh the selected preview only while Changes is active", async () => {
  vi.useFakeTimers();
  let comparison = comparison_id;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const r = request(args);
    if (r.operation === "changes") {
      if (r.refresh) comparison = other;
      return changes("ready", { ...page, comparison_id: comparison });
    }
    return {
      kind: "change_preview",
      observation: {
        target: r.target,
        phase: "ready",
        preview: {
          ...preview,
          comparison_id: comparison,
          rows: [
            {
              ...preview.rows[0],
              text: comparison === other ? "further edit" : "new content",
            },
          ],
        },
        error: null,
      },
    };
  });
  const { result, rerender } = renderHook(
    ({ active }) => useWorkspaceChanges("owner", workspace_id, active),
    { initialProps: { active: true } },
  );
  await act(async () => {});
  await act(async () => result.current.select(page.files[0]));
  expect(result.current.preview?.preview?.rows[0].text).toBe("new content");
  await act(async () => {
    await vi.advanceTimersByTimeAsync(2000);
  });
  expect(result.current.preview?.preview?.rows[0].text).toBe("further edit");
  rerender({ active: false });
  await act(async () => {});
  const count = vi
    .mocked(invoke)
    .mock.calls.filter(
      ([, args]) => request(args).operation === "change_preview",
    ).length;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(4000);
  });
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(
        ([, args]) => request(args).operation === "change_preview",
      ),
  ).toHaveLength(count);
});

test("tab changes drain an old read and preserve a queued manual refresh", async () => {
  vi.useFakeTimers();
  let finish: ((value: unknown) => void) | undefined;
  vi.mocked(invoke).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  const { result, rerender } = renderHook(
    ({ active }) => useWorkspaceChanges("owner", workspace_id, active),
    { initialProps: { active: false } },
  );
  rerender({ active: true });
  await act(async () => result.current.refresh());
  rerender({ active: false });
  expect(invoke).toHaveBeenCalledTimes(1);
  await act(async () =>
    finish?.(
      changes("failed", null, { code: "git", message: "obsolete failure" }),
    ),
  );
  expect(result.current.error).toBeUndefined();
  expect(result.current.observation?.phase).toBe("ready");
  expect(request(vi.mocked(invoke).mock.calls[1][1])).toMatchObject({
    refresh: true,
  });
});

test("late selected-file replies cannot replace a new selection, including A to B to A", async () => {
  vi.useFakeTimers();
  let finish: ((value: unknown) => void) | undefined;
  const { result } = renderHook(() =>
    useWorkspaceChanges("owner", workspace_id, true),
  );
  await act(async () => {});
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const r = request(args);
    if (r.operation === "changes") return changes();
    return new Promise((resolve) => {
      finish = resolve;
    });
  });
  await act(async () => result.current.select(page.files[0]));
  await act(async () => {
    result.current.select({ ...page.files[0], id: other, path: "B.txt" });
    result.current.select(page.files[0]);
  });
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const r = request(args);
    return r.operation === "changes"
      ? changes()
      : {
          kind: "change_preview",
          observation: {
            target: r.target,
            phase: "ready",
            preview,
            error: null,
          },
        };
  });
  await act(async () =>
    finish?.({
      kind: "change_preview",
      observation: {
        target,
        phase: "failed",
        preview: null,
        error: { code: "git", message: "obsolete preview" },
      },
    }),
  );
  expect(result.current.selected?.id).toBe(file_id);
  expect(result.current.preview?.phase).toBe("ready");
  expect(result.current.preview?.error).toBeNull();
});

test("inspection failures retain last-good data, pause timers and recover with manual Refresh", async () => {
  vi.useFakeTimers();
  const { result } = renderHook(() =>
    useWorkspaceChanges("owner", workspace_id, false),
  );
  await act(async () => {});
  vi.mocked(invoke).mockResolvedValue(
    changes("failed", page, { code: "git", message: "Git unavailable" }),
  );
  await act(async () => {
    await vi.advanceTimersByTimeAsync(2000);
  });
  expect(result.current.observation?.page).toEqual(page);
  expect(result.current.observation?.error?.message).toBe("Git unavailable");
  const calls = vi.mocked(invoke).mock.calls.length;
  await act(async () => {
    await vi.advanceTimersByTimeAsync(10000);
  });
  expect(invoke).toHaveBeenCalledTimes(calls);
  vi.mocked(invoke).mockResolvedValue(changes());
  await act(async () => result.current.refresh());
  expect(result.current.observation?.error).toBeNull();
  await act(async () => {
    await vi.advanceTimersByTimeAsync(2000);
  });
  expect(invoke).toHaveBeenCalledTimes(calls + 2);
});

test("manual Refresh settles a queued comparison after a failed preview", async () => {
  vi.useFakeTimers();
  let recovering = false;
  let requested = false;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const r = request(args);
    if (r.operation === "changes") {
      if (r.refresh) {
        recovering = true;
        requested = true;
        return changes("queued");
      }
      return changes(
        "ready",
        recovering ? { ...page, comparison_id: other } : page,
      );
    }
    return {
      kind: "change_preview",
      observation: {
        target: r.target,
        phase: recovering ? "ready" : "failed",
        preview: recovering ? { ...preview, comparison_id: other } : preview,
        error: recovering ? null : { code: "git", message: "Read failed" },
      },
    };
  });
  const { result } = renderHook(() =>
    useWorkspaceChanges("owner", workspace_id, true),
  );
  await act(async () => {});
  await act(async () => result.current.select(page.files[0]));
  expect(result.current.preview?.phase).toBe("failed");
  await act(async () => result.current.refresh());
  expect(requested).toBe(true);
  await act(async () => {
    await vi.advanceTimersByTimeAsync(250);
  });
  expect(result.current.preview?.phase).toBe("ready");
  expect(result.current.preview?.error).toBeNull();
});

test("pending previews poll observations without repeated loads or summary refreshes", async () => {
  vi.useFakeTimers();
  let previewReads = 0;
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const r = request(args);
    if (r.operation === "changes") return changes();
    previewReads++;
    return {
      kind: "change_preview",
      observation: {
        target: r.target,
        phase:
          previewReads === 1
            ? "queued"
            : previewReads === 2
              ? "running"
              : "ready",
        preview: previewReads === 3 ? preview : null,
        error: null,
      },
    };
  });
  const { result } = renderHook(() =>
    useWorkspaceChanges("owner", workspace_id, true),
  );
  await act(async () => {});
  await act(async () => result.current.select(page.files[0]));
  await act(async () => {
    await vi.advanceTimersByTimeAsync(500);
  });
  expect(result.current.preview?.phase).toBe("ready");
  const requests = vi
    .mocked(invoke)
    .mock.calls.map(([, args]) => request(args));
  expect(
    requests.filter((r) => r.operation === "change_preview").map((r) => r.load),
  ).toEqual([true, false, false]);
  expect(
    requests.filter((r) => r.operation === "changes").every((r) => !r.refresh),
  ).toBe(true);
});

test("workspace changes isolate retained data and drain late replies before reading the new target", async () => {
  vi.useFakeTimers();
  let finish: ((value: unknown) => void) | undefined;
  vi.mocked(invoke).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  const { result, rerender, unmount } = renderHook(
    ({ workspace }) => useWorkspaceChanges("owner", workspace, true),
    { initialProps: { workspace: workspace_id } },
  );
  vi.mocked(invoke).mockResolvedValue({
    ...changes(),
    observation: { ...changes().observation, workspace_id: other },
  });
  rerender({ workspace: other });
  expect(result.current.observation).toBeUndefined();
  expect(invoke).toHaveBeenCalledTimes(1);
  await act(async () => finish?.(changes()));
  expect(result.current.observation?.workspace_id).toBe(other);
  expect(request(vi.mocked(invoke).mock.calls[1][1])).toMatchObject({
    workspace_id: other,
    refresh: false,
  });
  unmount();
  expect(vi.getTimerCount()).toBe(0);
});

test("transport failure during a pending job keeps manual Refresh usable", async () => {
  render(
    <WorkspaceInspection
      reader={new WorkspaceChangesReader()}
      owner="owner"
      workspaceId={workspace_id}
    >
      <p>Details</p>
    </WorkspaceInspection>,
  );
  await screen.findByText(/Compared with HEAD/);
  vi.mocked(invoke).mockResolvedValueOnce(changes("queued"));
  fireEvent.click(screen.getByRole("button", { name: "Refresh changes" }));
  vi.mocked(invoke).mockRejectedValueOnce(new Error("Connection interrupted"));
  await screen.findByText("Connection interrupted");
  expect(
    (
      screen.getByRole("button", {
        name: "Refresh changes",
      }) as HTMLButtonElement
    ).disabled,
  ).toBe(false);
  expect(screen.getByText(/Showing the last-good comparison/)).toBeTruthy();
});

test("maximum preview rendering stays bounded, keeps long paths accessible and resets position for a different file", async () => {
  const second = {
    ...page.files[0],
    id: other,
    path: `src/${"long-directory/".repeat(25)}other.txt`,
  };
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const r = request(args);
    if (r.operation === "changes")
      return changes("ready", {
        ...page,
        files: [page.files[0], second],
        total: 2,
      });
    return {
      kind: "change_preview",
      observation: {
        target: r.target,
        phase: "ready",
        error: null,
        preview: {
          ...preview,
          rows: Array.from({ length: 200 }, (_, i) => ({
            kind: "added",
            old_line: null,
            new_line: i + 1,
            text: `row ${i + 1}`,
          })),
          limited: "Preview limited",
        },
      },
    };
  });
  render(
    <WorkspaceInspection
      reader={new WorkspaceChangesReader()}
      owner="owner"
      workspaceId={workspace_id}
    >
      <p>Details</p>
    </WorkspaceInspection>,
  );
  await screen.findByRole("button", { name: /src\/file.txt/ });
  fireEvent.click(screen.getByRole("button", { name: /src\/file.txt/ }));
  await screen.findByText("row 200");
  expect(document.querySelectorAll(".diff-row")).toHaveLength(200);
  const viewport = screen
    .getByLabelText("Unified file diff")
    .closest("[data-slot=scroll-area-viewport]") as HTMLElement;
  viewport.scrollTop = 70;
  viewport.scrollLeft = 20;
  fireEvent.scroll(viewport);
  const button = screen.getByTitle(second.path);
  expect(button.textContent).toContain(second.path);
  fireEvent.click(button);
  await waitFor(() =>
    expect(screen.getByRole("heading", { name: second.path })).toBeTruthy(),
  );
  await screen.findByText("row 200");
  const next = screen
    .getByLabelText("Unified file diff")
    .closest("[data-slot=scroll-area-viewport]") as HTMLElement;
  expect([next.scrollTop, next.scrollLeft]).toEqual([0, 0]);
  expect(document.querySelectorAll(".diff-row")).toHaveLength(200);
});

test("keyboard tab activation and Sheet dismissal preserve selection and restore focus", async () => {
  const user = (await import("@testing-library/user-event")).default.setup();
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: vi.fn(() => ({
      matches: true,
      addEventListener() {},
      removeEventListener() {},
    })),
  });
  render(
    <WorkspaceInspection
      reader={new WorkspaceChangesReader()}
      owner="owner"
      workspaceId={workspace_id}
    >
      <p>Details</p>
    </WorkspaceInspection>,
  );
  const workspaceTab = screen.getByRole("tab", { name: "Workspace" });
  workspaceTab.focus();
  await user.keyboard("{ArrowRight}");
  expect(document.activeElement).toBe(
    screen.getByRole("tab", { name: "Changes" }),
  );
  await user.keyboard("{Enter}");
  await waitFor(() =>
    expect(
      screen
        .getByRole("tab", { name: "Changes" })
        .getAttribute("aria-selected"),
    ).toBe("true"),
  );
  const trigger = screen.getByRole("button", { name: "Changed files" });
  trigger.focus();
  await user.keyboard("{Enter}");
  await screen.findByRole("dialog");
  const file = await screen.findByRole("button", { name: /src\/file.txt/ });
  file.focus();
  await user.keyboard("{Enter}");
  await screen.findByText("new content");
  await waitFor(() => expect(document.activeElement).toBe(trigger));
  await user.keyboard("{Enter}");
  await screen.findByRole("dialog");
  await user.keyboard("{Escape}");
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(document.activeElement).toBe(trigger);
  expect(screen.getByText("new content")).toBeTruthy();
});

test("unmount while IPC is pending ignores its rejection and schedules no reads", async () => {
  vi.useFakeTimers();
  let reject: ((error: Error) => void) | undefined;
  vi.mocked(invoke).mockImplementationOnce(
    () =>
      new Promise((_, fail) => {
        reject = fail;
      }),
  );
  const { unmount } = renderHook(() =>
    useWorkspaceChanges("owner", workspace_id, true),
  );
  unmount();
  await act(async () => reject?.(new Error("Late failure")));
  expect(vi.getTimerCount()).toBe(0);
  expect(invoke).toHaveBeenCalledTimes(1);
});

test("the resize separator remains keyboard reachable and changes the panel allocation", async () => {
  const user = (await import("@testing-library/user-event")).default.setup();
  render(
    <WorkspaceInspection
      reader={new WorkspaceChangesReader()}
      owner="owner"
      workspaceId={workspace_id}
    >
      <p>Details</p>
    </WorkspaceInspection>,
  );
  await screen.findByText(/Compared with HEAD/);
  const main = document.getElementById("inspection-main")!;
  const files = document.getElementById("inspection-files")!;
  Object.defineProperty(main, "offsetWidth", {
    configurable: true,
    value: 680,
  });
  Object.defineProperty(files, "offsetWidth", {
    configurable: true,
    value: 320,
  });
  const handle = screen.getByRole("separator", {
    name: "Resize changed files",
  });
  await act(async () => {
    resizeCallbacks.get(main.closest("[data-slot=resizable-panel-group]")!)?.(
      1000,
    );
    resizeCallbacks.get(files)?.(320);
  });
  handle.focus();
  const before = files.style.flexGrow;
  await user.keyboard("{ArrowLeft}");
  expect(document.activeElement).toBe(handle);
  expect(files.style.flexGrow).not.toBe(before);
  expect(Number(files.style.flexGrow)).toBeGreaterThan(32);
});

test.each([false, true])(
  "Settings retains the diff DOM, tab, selection, layout and reading position (Strict Mode: %s)",
  async (strict) => {
    const user = (await import("@testing-library/user-event")).default.setup();
    const content = (
      <InspectionNavigation reader={new WorkspaceChangesReader()} />
    );
    render(strict ? <StrictMode>{content}</StrictMode> : content);
    fireEvent.click(
      await screen.findByRole("button", { name: /src\/file.txt/ }),
    );
    await screen.findByText("new content");
    const viewport = screen
      .getByLabelText("Unified file diff")
      .closest<HTMLElement>("[data-slot=scroll-area-viewport]")!;
    viewport.scrollTop = 70;
    viewport.scrollLeft = 20;
    fireEvent.scroll(viewport);
    const main = document.getElementById("inspection-main")!;
    const files = document.getElementById("inspection-files")!;
    Object.defineProperty(main, "offsetWidth", {
      configurable: true,
      value: 680,
    });
    Object.defineProperty(files, "offsetWidth", {
      configurable: true,
      value: 320,
    });
    await act(async () => {
      resizeCallbacks.get(main.closest("[data-slot=resizable-panel-group]")!)?.(
        1000,
      );
      resizeCallbacks.get(files)?.(320);
    });
    screen.getByRole("separator", { name: "Resize changed files" }).focus();
    await user.keyboard("{ArrowLeft}");
    const width = files.style.flexGrow;
    expect(Number(width)).toBeGreaterThan(32);
    for (let visit = 0; visit < 2; visit++) {
      fireEvent.click(screen.getByRole("button", { name: "Settings" }));
      expect(screen.queryByRole("tab")).toBeNull();
      fireEvent.click(screen.getByRole("button", { name: "Back" }));
      expect(
        screen
          .getByRole("tab", { name: "Changes" })
          .getAttribute("aria-selected"),
      ).toBe("true");
      expect(
        screen
          .getByRole("button", { name: /src\/file.txt/ })
          .getAttribute("aria-pressed"),
      ).toBe("true");
      expect(
        screen
          .getByLabelText("Unified file diff")
          .closest("[data-slot=scroll-area-viewport]"),
      ).toBe(viewport);
      expect([viewport.scrollTop, viewport.scrollLeft]).toEqual([70, 20]);
      expect(files.style.flexGrow).toBe(width);
    }
    fireEvent.click(screen.getByRole("button", { name: "Hide changed files" }));
    await act(async () => {
      resizeCallbacks.get(files)?.(0);
    });
    await screen.findByRole("button", { name: "Show changed files" });
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    fireEvent.click(screen.getByRole("button", { name: "Back" }));
    expect(
      screen
        .getByRole("button", { name: "Show changed files" })
        .getAttribute("aria-expanded"),
    ).toBe("false");
  },
);

test.each([
  ["hidden", "success"],
  ["hidden", "failure"],
  ["revealed", "success"],
  ["revealed", "failure"],
])(
  "Activity pauses inspection and drains one lane when the old %s read settles with %s",
  async (settledWhile, outcome) => {
    vi.useFakeTimers();
    const add = vi.spyOn(document, "addEventListener");
    const remove = vi.spyOn(document, "removeEventListener");
    const listeners = () =>
      add.mock.calls.filter(([name]) => name === "visibilitychange").length -
      remove.mock.calls.filter(([name]) => name === "visibilitychange").length;
    let finish!: (value: unknown) => void;
    let reject!: (error: Error) => void;
    vi.mocked(invoke).mockImplementationOnce(
      () =>
        new Promise((resolve, fail) => {
          finish = resolve;
          reject = fail;
        }),
    );
    render(<InspectionNavigation reader={new WorkspaceChangesReader()} />);
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(listeners()).toBe(1);
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    expect(listeners()).toBe(0);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(10000);
    });
    expect(invoke).toHaveBeenCalledTimes(1);
    if (settledWhile === "revealed")
      fireEvent.click(screen.getByRole("button", { name: "Back" }));
    expect(invoke).toHaveBeenCalledTimes(1);
    await act(async () => {
      if (outcome === "success")
        finish(
          changes("failed", null, {
            code: "git",
            message: "Obsolete inspection failure",
          }),
        );
      else reject(new Error("Obsolete transport failure"));
    });
    if (settledWhile === "hidden") {
      expect(invoke).toHaveBeenCalledTimes(1);
      fireEvent.click(screen.getByRole("button", { name: "Back" }));
      await act(async () => {});
    }
    expect(listeners()).toBe(1);
    expect(screen.getByText(/Compared with HEAD/)).toBeDefined();
    expect(screen.queryByText(/Obsolete/)).toBeNull();
    expect(invoke).toHaveBeenCalledTimes(2);
    expect(request(vi.mocked(invoke).mock.calls[1][1]).operation).toBe(
      "changes",
    );
    expect(request(vi.mocked(invoke).mock.calls[1][1])).toMatchObject({
      refresh: false,
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(2000);
    });
    expect(invoke).toHaveBeenCalledTimes(3);
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(10000);
    });
    expect(invoke).toHaveBeenCalledTimes(3);
    add.mockRestore();
    remove.mockRestore();
  },
);

test("Settings dismisses an open changed-file Sheet without a hidden focus trap or reopening it", async () => {
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: () => ({
      matches: true,
      addEventListener() {},
      removeEventListener() {},
    }),
  });
  render(<InspectionNavigation reader={new WorkspaceChangesReader()} />);
  const settings = screen.getByRole("button", { name: "Settings" });
  fireEvent.click(screen.getByRole("button", { name: "Changed files" }));
  await screen.findByRole("dialog");
  fireEvent.click(settings);
  await act(async () => {});
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(document.activeElement).toBe(
    screen.getByRole("heading", { name: "Settings", level: 1 }),
  );
  fireEvent.click(screen.getByRole("button", { name: "Back" }));
  await act(async () => {});
  expect(screen.queryByRole("dialog")).toBeNull();
  expect(document.activeElement).toBe(settings);
  expect(
    screen
      .getByRole("button", { name: "Changed files" })
      .getAttribute("aria-expanded"),
  ).toBe("false");
});

test("WorkspacePanel navigation keeps one physical read lane across keyed inspection views", async () => {
  vi.useFakeTimers();
  let finish: ((value: unknown) => void) | undefined;
  vi.mocked(invoke).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  vi.mocked(invoke).mockImplementation(async (_, args) => {
    const r = request(args);
    if (r.operation !== "changes") throw new Error("No preview selected");
    return {
      ...changes(),
      observation: { ...changes().observation, workspace_id: r.workspace_id },
    };
  });
  const idle = () => {};
  const done = async () => {};
  const workspace: Workspace = {
    id: workspace_id,
    repository_id: comparison_id,
    creation_action_id: file_id,
    name: "first",
    path: "/private/first/checkout",
    branch: "workspace/first",
    provider: "codex",
    base: null,
    artifacts: { branch: "owned", folder: "worktree" },
    status: { state: "ready" },
    cleanup_unconfirmed: false,
    legacy: false,
  };
  const repositories: Repositories = {
    loading: false,
    initialWorkspace: null,
    pending: null,
    submitting: false,
    actionError: undefined,
    preferenceError: undefined,
    register: async () => false,
    refresh: done,
    select: idle,
    next: idle,
    back: idle,
    hasPrevious: false,
  };
  const workspaces: Workspaces = {
    pages: {},
    selected: workspace,
    pending: null,
    error: undefined,
    loading: false,
    submitting: false,
    preferenceError: undefined,
    select: idle,
    selectRepository: idle,
    create: done,
    reconcile: done,
    refresh: done,
    next: idle,
    back: idle,
    hasPrevious: () => false,
    retry: done,
  };
  const panel = (selected: Workspace | undefined) => (
    <WorkspacePanel
      connection={{
        state: "connected",
        observation: {
          desktopPid: 2,
          snapshot: {
            status: {
              state: "ready",
              supervisor_pid: 1,
              application_version: "fixture",
            },
            cursor: { owner: "owner", revision: 1 },
            reset: false,
          },
        },
      }}
      onRefreshStatus={done}
      repositories={repositories}
      workspaces={{ ...workspaces, selected }}
    />
  );
  const { rerender } = render(panel(workspace));
  rerender(panel({ ...workspace, id: other, name: "second" }));
  expect(invoke).toHaveBeenCalledTimes(1);
  rerender(panel(undefined));
  rerender(panel({ ...workspace, id: other, name: "second" }));
  expect(invoke).toHaveBeenCalledTimes(1);
  await act(async () =>
    finish?.(
      changes("failed", null, { code: "git", message: "Old target failure" }),
    ),
  );
  expect(invoke).toHaveBeenCalledTimes(2);
  expect(request(vi.mocked(invoke).mock.calls[1][1])).toMatchObject({
    workspace_id: other,
    refresh: false,
  });
  expect(screen.queryByText("Old target failure")).toBeNull();
});
