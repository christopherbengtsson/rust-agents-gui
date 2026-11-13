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
import type { ChangePage, FilePreview, InspectionRequest } from "./lib/changes";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
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
beforeEach(() => {
  Object.defineProperty(Element.prototype, "getAnimations", {
    configurable: true,
    value: () => [],
  });
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
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
  await waitFor(() => expect(result.current.observation?.phase).toBe("ready"));
  await act(async () => finish?.(changes("idle", null)));
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
    <WorkspaceInspection owner="owner" workspaceId={workspace_id}>
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
    <WorkspaceInspection owner="owner" workspaceId={workspace_id}>
      <p>Workspace details</p>
    </WorkspaceInspection>,
  );
  await screen.findByText(/Compared with HEAD/);
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
