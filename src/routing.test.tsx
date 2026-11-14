import {
  validateRepositoryRequest,
  validateWorkspaceRequest,
} from "./generated/contracts/validators";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { createMemoryHistory } from "@tanstack/react-router";
import { App } from "./App";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription,
} from "./components/ui/dialog";
import { ownerBound } from "./test-support/owner-bound-invoke";
import { object } from "./lib/observation-contracts";
import { PRESENTATION_KEY } from "./lib/presentation-store";
import type { RepositoryReply, RepositoryRequest } from "./lib/repositories";
import type {
  WorkspaceReply,
  WorkspaceRequest,
  Workspace,
} from "./lib/workspaces";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  Channel: class {
    constructor(public onmessage: (value: unknown) => void) {}
  },
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
const repositoryId = "00000000-0000-4000-8000-000000000001";
const workspaceId = "00000000-0000-4000-8000-000000000002";
const otherId = "00000000-0000-4000-8000-000000000003";
const actionId = "00000000-0000-4000-8000-000000000004";
const repository = {
  id: repositoryId,
  name: "exact repository",
  path: "/fixture/exact",
  availability: { state: "available" },
} as const;
const workspace: Workspace = {
  id: workspaceId,
  repository_id: repositoryId,
  creation_action_id: actionId,
  name: "exact workspace",
  branch: "workspace/exact",
  path: "/fixture/workspace",
  provider: "codex",
  base: { reference: "refs/heads/main", commit: "a".repeat(40) },
  artifacts: { branch: "owned", folder: "worktree" },
  status: { state: "ready" },
  cleanup_unconfirmed: false,
  legacy: false,
};
const repositoryPath = `/repository/${repositoryId}`;
const workspacePath = `${repositoryPath}/workspace/${workspaceId}`;
let repositoryRead: (request: RepositoryRequest) => Promise<RepositoryReply>;
let workspaceRead: (request: WorkspaceRequest) => Promise<WorkspaceReply>;
let connected: boolean;
beforeEach(() => {
  localStorage.clear();
  vi.stubGlobal(
    "ResizeObserver",
    class implements ResizeObserver {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
  connected = true;
  vi.stubGlobal("matchMedia", (media: string) =>
    Object.assign(new EventTarget(), {
      media,
      matches: false,
      addListener() {},
      removeListener() {},
    }),
  );
  repositoryRead = async (request) =>
    request.operation === "get"
      ? {
          kind: "repository",
          repository: request.id === repositoryId ? repository : null,
        }
      : request.operation === "action"
        ? { kind: "action", registration: null }
        : { kind: "repositories", page: { repositories: [], next: null } };
  workspaceRead = async (request) =>
    request.operation === "get"
      ? { kind: "workspace", workspace }
      : { kind: "workspaces", page: { workspaces: [], next: null } };
  vi.mocked(invoke).mockImplementation(
    ownerBound(async (command, args) => {
      if (command === "supervisor_status") {
        if (!connected) throw new Error("Controlled connection unavailable");
        return {
          status: {
            state: "ready",
            supervisor_pid: 41,
            application_version: "0.1.0",
          },
          cursor: { owner: "owner-41", revision: 1 },
          reset: true,
        };
      }
      if (command === "desktop_pid") return 42;
      if (command.startsWith("observation_")) return;
      if (!object(args) || !object(args.request))
        throw new Error("Missing request");
      // The fixture supplies dependency outcomes; production decoders still validate them.
      if (
        command === "repository_request" &&
        validateRepositoryRequest(args.request)
      )
        return repositoryRead(args.request);
      if (
        command === "workspace_request" &&
        validateWorkspaceRequest(args.request)
      )
        return workspaceRead(args.request);
      throw new Error(`Unexpected command ${command}`);
    }),
  );
});
afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.resetAllMocks();
  vi.unstubAllGlobals();
});
function mount(path: string, entries = [path]) {
  const history = createMemoryHistory({ initialEntries: entries });
  render(<App history={history} />);
  return history;
}
function remember(view: "workspace" | "changes" = "workspace") {
  localStorage.setItem(
    PRESENTATION_KEY,
    JSON.stringify({
      version: 2,
      state: {
        inspection: {},
        contexts: {},
        rememberedMain: { kind: "workspace", repositoryId, workspaceId, view },
      },
    }),
  );
}

test("a direct target resolves exact details outside both listed pages", async () => {
  const history = mount(workspacePath);
  await screen.findByRole("heading", { name: "exact workspace", level: 1 });
  expect(history.location.pathname).toBe(workspacePath);
  expect(
    screen
      .getByRole("tab", { name: "Workspace" })
      .getAttribute("aria-selected"),
  ).toBe("true");
  expect(screen.queryByRole("button", { name: "exact repository" })).toBeNull();
});
test("invalid search preserves the explicit URL and offers typed valid views", async () => {
  const history = mount(`${workspacePath}?view=other`);
  await screen.findByRole("heading", { name: "Invalid workspace view" });
  expect(history.location.search).toBe("?view=other");
  await userEvent
    .setup()
    .click(screen.getByRole("link", { name: "Workspace" }));
  await screen.findByRole("heading", { name: "exact workspace", level: 1 });
  expect(history.location.search).toBe("?view=workspace");
});
test("wrong repository membership leaves the explicit target unavailable", async () => {
  workspaceRead = async (request) =>
    request.operation === "get"
      ? {
          kind: "workspace",
          workspace: { ...workspace, repository_id: otherId },
        }
      : { kind: "workspaces", page: { workspaces: [], next: null } };
  const history = mount(workspacePath);
  await screen.findByRole("heading", { name: "Destination unavailable" });
  expect(history.location.pathname).toBe(workspacePath);
  expect(screen.queryByRole("tab", { name: "Workspace" })).toBeNull();
});
test("an explicit absent repository never substitutes a listed record", async () => {
  repositoryRead = async (request) =>
    request.operation === "get"
      ? { kind: "repository", repository: null }
      : {
          kind: "repositories",
          page: { repositories: [repository], next: null },
        };
  const history = mount(repositoryPath);
  await screen.findByRole("heading", { name: "Destination unavailable" });
  expect(history.location.pathname).toBe(repositoryPath);
  expect(
    screen.queryByRole("region", { name: "Selected repository" }),
  ).toBeNull();
});
test("startup restoration is one-time and a later Home visit stays home", async () => {
  remember();
  const history = mount("/");
  await screen.findByRole("heading", { name: "exact workspace", level: 1 });
  await act(async () => history.push("/"));
  await screen.findByRole("heading", { name: "Workspace", level: 1 });
  expect(history.location.pathname).toBe("/");
});
test("an explicit destination takes precedence over remembered workspace and view", async () => {
  remember("changes");
  const history = mount(repositoryPath);
  await screen.findByRole("heading", { name: "exact repository", level: 1 });
  expect(history.location.pathname).toBe(repositoryPath);
  expect(screen.queryByRole("tab", { name: "Changes" })).toBeNull();
});
test("confirmed stale restoration falls home without selecting a listed substitute", async () => {
  remember();
  repositoryRead = async (request) =>
    request.operation === "get"
      ? { kind: "repository", repository: null }
      : {
          kind: "repositories",
          page: { repositories: [repository], next: null },
        };
  const history = mount("/");
  await waitFor(() =>
    expect(
      JSON.parse(localStorage.getItem(PRESENTATION_KEY) ?? "null").state
        .rememberedMain,
    ).toEqual({ kind: "home" }),
  );
  await screen.findByRole("heading", { name: "Workspace", level: 1 });
  expect(history.location.pathname).toBe("/");
});
test("temporary connection failure retains restoration and explicit retry confirms its exact target", async () => {
  connected = false;
  remember();
  const history = mount("/");
  await screen.findByRole("heading", { name: "Destination unavailable" });
  expect(history.location.pathname).toBe(workspacePath);
  connected = true;
  await userEvent
    .setup()
    .click(screen.getByRole("button", { name: "Retry destination" }));
  await screen.findByRole("heading", { name: "exact workspace", level: 1 });
});
test("settings replacement preserves one feed and returning retains workspace context", async () => {
  const history = mount(workspacePath);
  await screen.findByRole("heading", { name: "exact workspace", level: 1 });
  const subscriptions = () =>
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === "observation_subscribe")
      .length;
  expect(subscriptions()).toBe(1);
  await act(async () => history.push("/settings"));
  await screen.findByRole("heading", { name: "Settings", level: 1 });
  expect(history.location.pathname).toBe("/settings/general");
  expect(screen.queryByRole("tab", { name: "Workspace" })).toBeNull();
  await act(async () => history.back());
  await screen.findByRole("heading", { name: "exact workspace", level: 1 });
  expect(subscriptions()).toBe(1);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === "observation_stop"),
  ).toHaveLength(0);
});

test("the footer gear opens full-page General settings by keyboard", async () => {
  const history = mount(workspacePath);
  await screen.findByRole("heading", { name: "exact workspace", level: 1 });
  const gear = screen.getByRole("link", { name: "Settings" });
  gear.focus();
  await userEvent.setup().keyboard("{Enter}");
  const heading = await screen.findByRole("heading", {
    name: "Settings",
    level: 1,
  });
  expect(history.location.pathname).toBe("/settings/general");
  expect(document.activeElement).toBe(heading);
  expect(screen.getByRole("navigation", { name: "Settings" })).toBeTruthy();
  expect(
    screen.getByRole("link", { name: "General" }).getAttribute("aria-current"),
  ).toBe("page");
  expect(screen.getByRole("main", { name: "General" })).toBeTruthy();
  expect(
    screen.getByText("General settings will be available here."),
  ).toBeTruthy();
  expect(
    screen.queryByRole("complementary", { name: "Repository sidebar" }),
  ).toBeNull();
  expect(screen.queryByRole("tab", { name: "Workspace" })).toBeNull();
  expect(
    screen.getByRole("link", { name: "Skip to settings" }).getAttribute("href"),
  ).toBe("#settings-main");
});

test.each(["Back", "Escape", "history"])(
  "Settings %s returns to the preceding workspace and restores gear focus",
  async (method) => {
    const history = mount(`${workspacePath}?view=changes`);
    await screen.findByRole("heading", { name: "exact workspace", level: 1 });
    const user = userEvent.setup();
    await user.click(screen.getByRole("link", { name: "Settings" }));
    await screen.findByRole("heading", { name: "Settings", level: 1 });
    if (method === "Back")
      await user.click(screen.getByRole("button", { name: "Back" }));
    else if (method === "Escape") await user.keyboard("{Escape}");
    else await act(async () => history.back());
    await screen.findByRole("heading", { name: "exact workspace", level: 1 });
    expect(history.location.href).toBe(`${workspacePath}?view=changes`);
    expect(document.activeElement).toBe(
      screen.getByRole("link", { name: "Settings" }),
    );
    await act(async () => history.forward());
    await screen.findByRole("heading", { name: "Settings", level: 1 });
    await user.keyboard("{Escape}");
    await screen.findByRole("heading", { name: "exact workspace", level: 1 });
    expect(history.location.href).toBe(`${workspacePath}?view=changes`);
  },
);

test.each([false, true])(
  "direct Settings entry falls home even with remembered context: %s",
  async (saved) => {
    if (saved) remember("changes");
    const history = mount("/settings/general");
    await screen.findByRole("heading", { name: "Settings", level: 1 });
    await userEvent.setup().click(screen.getByRole("button", { name: "Back" }));
    await screen.findByRole("heading", { name: "Workspace", level: 1 });
    expect(history.location.pathname).toBe("/");
    expect(history.length).toBe(1);
    expect(document.activeElement).toBe(screen.getByRole("main"));
  },
);

test("the settings redirect retains the preceding app entry and returns focus to main", async () => {
  const history = mount(repositoryPath);
  await screen.findByRole("heading", { name: "exact repository", level: 1 });
  await act(async () => history.push("/settings"));
  await screen.findByRole("heading", { name: "Settings", level: 1 });
  expect(history.location.pathname).toBe("/settings/general");
  expect(history.length).toBe(2);
  await userEvent.setup().keyboard("{Escape}");
  await screen.findByRole("heading", { name: "exact repository", level: 1 });
  expect(history.location.pathname).toBe(repositoryPath);
  expect(document.activeElement).toBe(screen.getByRole("main"));
});

test("a nested Base UI overlay consumes Escape before Settings navigation", async () => {
  const history = mount(repositoryPath);
  await screen.findByRole("heading", { name: "exact repository", level: 1 });
  const user = userEvent.setup();
  await user.click(screen.getByRole("link", { name: "Settings" }));
  await screen.findByRole("heading", { name: "Settings", level: 1 });
  // A real portaled overlay exercises the installed dismissal contract; the
  // placeholder itself adds no artificial controls just to open one.
  const overlay = render(
    <Dialog defaultOpen>
      <DialogContent>
        <DialogTitle>Nested overlay</DialogTitle>
        <DialogDescription>Controlled overlay fixture.</DialogDescription>
      </DialogContent>
    </Dialog>,
  );
  await screen.findByRole("dialog", { name: "Nested overlay" });
  await user.keyboard("{Escape}");
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(history.location.pathname).toBe("/settings/general");
  overlay.unmount();
  await user.keyboard("{Escape}");
  await screen.findByRole("heading", { name: "exact repository", level: 1 });
});

test("activating General keeps one Settings history entry and its return destination", async () => {
  const history = mount(repositoryPath);
  await screen.findByRole("heading", { name: "exact repository", level: 1 });
  const user = userEvent.setup();
  await user.click(screen.getByRole("link", { name: "Settings" }));
  await screen.findByRole("heading", { name: "Settings", level: 1 });
  await user.click(screen.getByRole("link", { name: "General" }));
  expect(history.length).toBe(2);
  await user.click(screen.getByRole("button", { name: "Back" }));
  await screen.findByRole("heading", { name: "exact repository", level: 1 });
  expect(history.location.pathname).toBe(repositoryPath);
});

test("Settings ignores prevented, composing and repeated Escape events", async () => {
  const history = mount("/settings/general");
  const heading = await screen.findByRole("heading", {
    name: "Settings",
    level: 1,
  });
  const prevent = (event: KeyboardEvent) => event.preventDefault();
  heading.addEventListener("keydown", prevent);
  fireEvent.keyDown(heading, { key: "Escape" });
  heading.removeEventListener("keydown", prevent);
  fireEvent.keyDown(heading, { key: "Escape", isComposing: true });
  fireEvent.keyDown(heading, { key: "Escape", repeat: true });
  expect(history.location.pathname).toBe("/settings/general");
  await userEvent.setup().keyboard("{Escape}");
  await screen.findByRole("heading", { name: "Workspace", level: 1 });
});

test.each([false, true])(
  "routed Settings and workspace switches retain reading context and revalidate changed comparisons: %s",
  async (changed) => {
    const comparison = "00000000-0000-4000-8000-000000000005";
    const replacement = "00000000-0000-4000-8000-000000000006";
    let currentComparison = comparison;
    const ordinary = workspaceRead;
    workspaceRead = async (request) => {
      if (request.operation === "get")
        return {
          kind: "workspace",
          workspace: {
            ...workspace,
            id: request.id,
            name:
              request.id === workspaceId
                ? "exact workspace"
                : "second workspace",
          },
        };
      if (request.operation === "changes")
        return {
          kind: "changes",
          observation: {
            workspace_id: request.workspace_id,
            phase: "ready",
            error: null,
            page: {
              comparison_id: currentComparison,
              head: "a".repeat(40),
              total: 1,
              next: null,
              files: [
                {
                  id: actionId,
                  path: "src/retained.txt",
                  index_status: ".",
                  worktree_status: "M",
                  unsupported: null,
                },
              ],
            },
          },
        };
      if (request.operation === "change_preview")
        return {
          kind: "change_preview",
          observation: {
            target: request.target,
            phase: "ready",
            error: null,
            preview: {
              comparison_id: request.target.comparison_id,
              head: "a".repeat(40),
              unsupported: null,
              limited: null,
              note: null,
              rows: [
                {
                  kind: "added",
                  old_line: null,
                  new_line: 1,
                  text:
                    request.target.workspace_id === workspaceId
                      ? "First workspace diff"
                      : "Second workspace diff",
                },
              ],
            },
          },
        };
      return ordinary(request);
    };
    localStorage.setItem(
      PRESENTATION_KEY,
      JSON.stringify({
        version: 2,
        state: {
          contexts: {
            [workspaceId]: {
              view: "changes",
              filesWidth: 41,
              collapsed: false,
            },
            [otherId]: { view: "changes", filesWidth: 27, collapsed: true },
          },
          inspection: {
            [otherId]: {
              id: actionId,
              path: "src/retained.txt",
              comparison,
              position: { top: 73, left: 4 },
            },
          },
        },
      }),
    );
    const history = mount(`${workspacePath}?view=changes`);
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole("button", { name: /src\/retained.txt/ }),
    );
    await screen.findByText("First workspace diff");
    const viewport = () => {
      const element = screen
        .getByLabelText("Unified file diff")
        .closest<HTMLElement>("[data-slot=scroll-area-viewport]");
      if (!element) throw new Error("Missing diff viewport");
      return element;
    };
    viewport().scrollTop = 37;
    viewport().scrollLeft = 12;
    fireEvent.scroll(viewport());
    await user.click(screen.getByRole("link", { name: "Settings" }));
    await screen.findByRole("heading", { name: "Settings", level: 1 });
    expect(screen.queryByText("First workspace diff")).toBeNull();
    await user.click(screen.getByRole("button", { name: "Back" }));
    await screen.findByText("First workspace diff");
    expect([viewport().scrollTop, viewport().scrollLeft]).toEqual([37, 12]);
    // jsdom has no panel geometry; assert retained policy here. Existing resize
    // scenarios drive controlled dimensions, while F08 qualifies actual layout.
    expect(
      JSON.parse(localStorage.getItem(PRESENTATION_KEY) ?? "null").state
        .contexts[workspaceId],
    ).toEqual({ view: "changes", filesWidth: 41, collapsed: false });
    await act(async () =>
      history.push(`${repositoryPath}/workspace/${otherId}?view=changes`),
    );
    await screen.findByText("Second workspace diff");
    expect([viewport().scrollTop, viewport().scrollLeft]).toEqual([73, 4]);
    expect(
      screen.getByRole("button", { name: "Show changed files" }),
    ).toBeTruthy();
    await act(async () => history.back());
    await screen.findByText("First workspace diff");
    expect([viewport().scrollTop, viewport().scrollLeft]).toEqual([37, 12]);
    await user.click(screen.getByRole("link", { name: "Settings" }));
    await screen.findByRole("heading", { name: "Settings", level: 1 });
    if (changed) currentComparison = replacement;
    await user.keyboard("{Escape}");
    if (changed) {
      await screen.findByText("Select a changed file");
      expect(screen.queryByLabelText("Unified file diff")).toBeNull();
      expect(
        JSON.parse(localStorage.getItem(PRESENTATION_KEY) ?? "null").state
          .inspection[workspaceId],
      ).toBeUndefined();
    } else {
      await screen.findByText("First workspace diff");
      expect([viewport().scrollTop, viewport().scrollLeft]).toEqual([37, 12]);
    }
  },
);

test.each(["settings", "history", "aba"])(
  "delayed registration cannot redirect after %s navigation",
  async (navigation) => {
    let finish: ((reply: RepositoryReply) => void) | undefined;
    let registration:
      | Extract<RepositoryRequest, { operation: "register" }>
      | undefined;
    const ordinary = repositoryRead;
    repositoryRead = async (request) => {
      if (request.operation === "register") {
        registration = request;
        return new Promise((resolve) => {
          finish = resolve;
        });
      }
      if (request.operation === "action")
        return { kind: "action", registration: null };
      return ordinary(request);
    };
    const history = mount(repositoryPath, ["/", repositoryPath]);
    await screen.findByRole("heading", { name: "exact repository", level: 1 });
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Add repository" }));
    await user.type(
      screen.getByRole("textbox", { name: "Repository folder" }),
      "/fixture/new",
    );
    await user.click(screen.getByRole("button", { name: /^Register$/ }));
    await waitFor(() => expect(finish).toBeDefined());
    await user.keyboard("{Escape}");
    await act(async () => {
      if (navigation === "history") history.back();
      else history.push("/settings/general");
    });
    if (navigation === "aba")
      await act(async () => history.push(repositoryPath));
    const destination = history.location.href;
    const submitted = registration;
    if (!submitted || !finish)
      throw new Error("Registration was not submitted");
    await act(async () =>
      finish?.({
        kind: "registration",
        registration: {
          ...submitted.request,
          state: "completed",
          repository_id: otherId,
          initial_workspace_id: null,
        },
      }),
    );
    await waitFor(() =>
      expect(
        localStorage.getItem("agent-workspace.pending-registration"),
      ).toBeNull(),
    );
    expect(history.location.href).toBe(destination);
    expect(
      vi
        .mocked(invoke)
        .mock.calls.filter(
          ([, args]) =>
            object(args) &&
            object(args.request) &&
            args.request.operation === "register",
        ),
    ).toHaveLength(1);
  },
);

test.each(["settings", "history", "aba"])(
  "delayed creation cannot redirect after %s navigation",
  async (navigation) => {
    let finish: ((reply: WorkspaceReply) => void) | undefined;
    let creation:
      | Extract<WorkspaceRequest, { operation: "create" }>
      | undefined;
    const ordinary = repositoryRead;
    repositoryRead = async (request) =>
      request.operation === "list"
        ? {
            kind: "repositories",
            page: { repositories: [repository], next: null },
          }
        : ordinary(request);
    workspaceRead = async (request) => {
      if (request.operation === "create") {
        creation = request;
        return new Promise((resolve) => {
          finish = resolve;
        });
      }
      if (request.operation === "action")
        return { kind: "action", action: null, workspace: null };
      if (request.operation === "get") return { kind: "workspace", workspace };
      return { kind: "workspaces", page: { workspaces: [], next: null } };
    };
    const history = mount(repositoryPath, ["/", repositoryPath]);
    await screen.findByRole("heading", { name: "exact repository", level: 1 });
    await userEvent.setup().click(
      screen.getByRole("button", {
        name: "Create workspace in exact repository",
      }),
    );
    await waitFor(() => expect(finish).toBeDefined());
    await act(async () => {
      if (navigation === "history") history.back();
      else history.push("/settings/general");
    });
    if (navigation === "aba")
      await act(async () => history.push(repositoryPath));
    const destination = history.location.href;
    const submitted = creation;
    if (!submitted || !finish) throw new Error("Creation was not submitted");
    await act(async () =>
      finish?.({
        kind: "action",
        action: submitted,
        workspace: { ...workspace, creation_action_id: submitted.action_id },
      }),
    );
    await waitFor(() =>
      expect(
        localStorage.getItem("agent-workspace.pending-workspace"),
      ).toBeNull(),
    );
    expect(history.location.href).toBe(destination);
  },
);

test("an explicit navigation wins while startup detail validation is delayed", async () => {
  remember();
  let release: ((reply: WorkspaceReply) => void) | undefined;
  workspaceRead = async (request) =>
    request.operation === "get"
      ? new Promise((resolve) => {
          release = resolve;
        })
      : { kind: "workspaces", page: { workspaces: [], next: null } };
  const history = mount("/");
  await waitFor(() => expect(release).toBeDefined());
  await act(async () => history.push("/settings/general"));
  await screen.findByRole("heading", { name: "Settings", level: 1 });
  await act(async () => release?.({ kind: "workspace", workspace: null }));
  expect(history.location.pathname).toBe("/settings/general");
});

test.each([false, true])(
  "slow inspection drains across Settings without duplicate or hidden reads; return before completion: %s",
  async (returnFirst) => {
    const ordinary = workspaceRead;
    let reads = 0;
    let active = 0;
    let maximum = 0;
    const releases: (() => void)[] = [];
    workspaceRead = async (request) => {
      if (request.operation !== "changes") return ordinary(request);
      reads++;
      active++;
      maximum = Math.max(maximum, active);
      return new Promise((resolve) =>
        releases.push(() => {
          active--;
          resolve({
            kind: "changes",
            observation: {
              workspace_id: workspaceId,
              phase: "idle",
              page: null,
              error: null,
            },
          });
        }),
      );
    };
    const history = mount(workspacePath);
    await waitFor(() => expect(reads).toBe(1));
    await act(async () => history.push("/settings/general"));
    await screen.findByRole("heading", { name: "Settings", level: 1 });
    expect(reads).toBe(1);
    if (!returnFirst) {
      await act(async () => releases[0]());
      vi.useFakeTimers();
      await act(async () => vi.advanceTimersByTimeAsync(10_000));
      expect(reads).toBe(1);
      expect(active).toBe(0);
      vi.useRealTimers();
    }
    await act(async () => history.back());
    await screen.findByRole("heading", { name: "exact workspace", level: 1 });
    if (returnFirst) {
      expect(reads).toBe(1);
      await act(async () => releases[0]());
    }
    await waitFor(() => expect(reads).toBe(2));
    await act(async () => releases[1]());
    expect(maximum).toBe(1);
  },
);

test.each([
  "/repository/not-an-id",
  `${repositoryPath}/workspace/not-an-id`,
  "/missing",
])("malformed explicit destination remains unavailable: %s", async (path) => {
  const history = mount(path);
  await screen.findByRole("heading", { name: "Destination unavailable" });
  expect(history.location.pathname).toBe(path);
});

test("route preloading prepares through feature interests without navigation or retained hidden reads", async () => {
  const { AppObservations } = await import("./lib/app-observations");
  const { createAppRouter } = await import("./router");
  const app = new AppObservations();
  app.connect("owner-41");
  app.accept({
    subscription_id: "fixture",
    batch_id: 1,
    cursor: { owner: "owner-41", sequence: 1 },
    sample: { kind: "resync_required", watermark: 1 },
  });
  const history = createMemoryHistory({ initialEntries: ["/"] });
  const router = createAppRouter(app, history);
  const navigation = router.options.context.navigation;
  const stop = navigation.start();
  const token = navigation.capture();
  await router.preloadRoute({
    to: "/repository/$repositoryId/workspace/$workspaceId",
    params: { repositoryId, workspaceId },
    search: { view: "workspace" },
  });
  await waitFor(() =>
    expect(
      app.workspaces.isCurrent("owner-41", {
        operation: "get",
        id: workspaceId,
      }),
    ).toBe(true),
  );
  await act(async () => {});
  const reads = vi.mocked(invoke).mock.calls.length;
  app.accept({
    subscription_id: "fixture",
    batch_id: 2,
    cursor: { owner: "owner-41", sequence: 2 },
    sample: {
      kind: "hints",
      watermark: 2,
      targets: [{ kind: "workspace", workspace_id: workspaceId }],
    },
  });
  await act(async () => {});
  expect(vi.mocked(invoke).mock.calls.length).toBe(reads);
  expect(navigation.matches(token)).toBe(true);
  expect(history.location.pathname).toBe("/");
  stop();
});

test("explicit route views update remembered defaults for later workspace selection", async () => {
  const ordinary = repositoryRead;
  repositoryRead = async (request) =>
    request.operation === "list"
      ? {
          kind: "repositories",
          page: { repositories: [repository], next: null },
        }
      : ordinary(request);
  workspaceRead = async (request) =>
    request.operation === "get"
      ? { kind: "workspace", workspace }
      : request.operation === "changes"
        ? {
            kind: "changes",
            observation: {
              workspace_id: workspaceId,
              phase: "idle",
              page: null,
              error: null,
            },
          }
        : { kind: "workspaces", page: { workspaces: [workspace], next: null } };
  const history = mount(`${workspacePath}?view=changes`);
  await screen.findByRole("heading", { name: "exact workspace", level: 1 });
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "exact repository" }));
  await screen.findByRole("heading", { name: "exact repository", level: 1 });
  await user.click(screen.getByRole("button", { name: "exact workspace" }));
  await screen.findByRole("heading", { name: "exact workspace", level: 1 });
  expect(history.location.search).toBe("?view=changes");
});

test("Settings releases collection interests, retains dirty generations and refreshes them on return", async () => {
  let updated = false;
  const secondRepository = {
    ...repository,
    id: otherId,
    name: "second repository",
  };
  repositoryRead = async (request) => {
    const current = {
      ...repository,
      name: updated ? "updated repository" : repository.name,
    };
    if (request.operation === "list")
      return {
        kind: "repositories",
        page: { repositories: [current, secondRepository], next: null },
      };
    if (request.operation === "get")
      return { kind: "repository", repository: current };
    return { kind: "action", registration: null };
  };
  workspaceRead = async (request) =>
    request.operation === "get"
      ? {
          kind: "workspace",
          workspace: {
            ...workspace,
            name: updated ? "updated workspace" : workspace.name,
          },
        }
      : { kind: "workspaces", page: { workspaces: [], next: null } };
  mount(workspacePath);
  await screen.findByRole("heading", { name: "exact workspace", level: 1 });
  await userEvent.setup().click(screen.getByRole("link", { name: "Settings" }));
  await screen.findByRole("heading", { name: "Settings", level: 1 });
  await act(async () => {});
  const reads = () =>
    vi
      .mocked(invoke)
      .mock.calls.filter(
        ([command, args]) =>
          (command === "repository_request" ||
            command === "workspace_request") &&
          object(args) &&
          object(args.request) &&
          ["get", "list"].includes(String(args.request.operation)),
      );
  const before = reads().length;
  const subscription = vi
    .mocked(invoke)
    .mock.calls.find(([command]) => command === "observation_subscribe")?.[1];
  if (
    !object(subscription) ||
    !object(subscription.onEvent) ||
    typeof subscription.onEvent.onmessage !== "function"
  )
    throw new Error("Missing controlled feed channel");
  const receive = subscription.onEvent.onmessage;
  updated = true;
  await act(async () =>
    receive({
      kind: "batch",
      batch: {
        subscription_id: subscription.subscriptionId,
        batch_id: 2,
        cursor: { owner: "owner-41", sequence: 2 },
        sample: {
          kind: "hints",
          watermark: 2,
          targets: [
            { kind: "repositories" },
            { kind: "workspaces", repository_id: null },
            { kind: "repository", repository_id: repositoryId },
            { kind: "workspace", workspace_id: workspaceId },
          ],
        },
      },
    }),
  );
  expect(reads()).toHaveLength(before);
  await userEvent.setup().keyboard("{Escape}");
  await screen.findByRole("heading", { name: "updated workspace", level: 1 });
  expect(
    screen.getByRole("button", { name: "updated repository" }),
  ).toBeTruthy();
  expect(reads().length).toBeGreaterThan(before);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === "observation_subscribe"),
  ).toHaveLength(1);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === "observation_stop"),
  ).toHaveLength(0);
});

test("direct Settings keeps retained action status interested without reading hidden collections or replaying", async () => {
  const request = {
    action_id: actionId,
    path: "/fixture/pending",
    create_initial_workspace: false,
  };
  localStorage.setItem(
    "agent-workspace.pending-registration",
    JSON.stringify({ request, selection: null, uncertain: true }),
  );
  let complete = false;
  repositoryRead = async (operation) =>
    operation.operation === "action"
      ? {
          kind: "action",
          registration: complete
            ? {
                ...request,
                state: "completed",
                repository_id: repositoryId,
                initial_workspace_id: null,
              }
            : { ...request, initial_workspace_id: null, state: "pending" },
        }
      : { kind: "repositories", page: { repositories: [], next: null } };
  const history = mount("/settings/general");
  await screen.findByRole("heading", { name: "Settings", level: 1 });
  const actions = () =>
    vi
      .mocked(invoke)
      .mock.calls.filter(
        ([command, args]) =>
          command === "repository_request" &&
          object(args) &&
          object(args.request) &&
          args.request.operation === "action",
      );
  await waitFor(() => expect(actions()).toHaveLength(1));
  const subscription = vi
    .mocked(invoke)
    .mock.calls.find(([command]) => command === "observation_subscribe")?.[1];
  if (
    !object(subscription) ||
    !object(subscription.onEvent) ||
    typeof subscription.onEvent.onmessage !== "function"
  )
    throw new Error("Missing controlled feed channel");
  const receive = subscription.onEvent.onmessage;
  complete = true;
  await act(async () =>
    receive({
      kind: "batch",
      batch: {
        subscription_id: subscription.subscriptionId,
        batch_id: 2,
        cursor: { owner: "owner-41", sequence: 2 },
        sample: {
          kind: "hints",
          watermark: 2,
          targets: [
            { kind: "registration", action_id: actionId },
            { kind: "repositories" },
            { kind: "workspaces", repository_id: null },
          ],
        },
      },
    }),
  );
  await waitFor(() =>
    expect(
      localStorage.getItem("agent-workspace.pending-registration"),
    ).toBeNull(),
  );
  expect(actions()).toHaveLength(2);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) =>
        ["repository_request", "workspace_request"].includes(command),
      ),
  ).toHaveLength(2);
  expect(history.location.pathname).toBe("/settings/general");
});

test("Settings retains uncertain workspace action lookup without hidden reads or replay", async () => {
  const request: Extract<WorkspaceRequest, { operation: "create" }> = {
    operation: "create",
    action_id: actionId,
    repository_id: repositoryId,
  };
  localStorage.setItem(
    "agent-workspace.pending-workspace",
    JSON.stringify({ request, uncertain: true }),
  );
  let complete = false;
  workspaceRead = async (operation) =>
    operation.operation === "action"
      ? {
          kind: "action",
          action: complete ? request : null,
          workspace: complete ? workspace : null,
        }
      : { kind: "workspaces", page: { workspaces: [], next: null } };
  const history = mount("/settings/general");
  await screen.findByRole("heading", { name: "Settings", level: 1 });
  const actions = () =>
    vi
      .mocked(invoke)
      .mock.calls.filter(
        ([command, args]) =>
          command === "workspace_request" &&
          object(args) &&
          object(args.request) &&
          args.request.operation === "action",
      );
  await waitFor(() => expect(actions()).toHaveLength(1));
  const subscription = vi
    .mocked(invoke)
    .mock.calls.find(([command]) => command === "observation_subscribe")?.[1];
  if (
    !object(subscription) ||
    !object(subscription.onEvent) ||
    typeof subscription.onEvent.onmessage !== "function"
  )
    throw new Error("Missing controlled feed channel");
  const receive = subscription.onEvent.onmessage;
  complete = true;
  await act(async () =>
    receive({
      kind: "batch",
      batch: {
        subscription_id: subscription.subscriptionId,
        batch_id: 2,
        cursor: { owner: "owner-41", sequence: 2 },
        sample: {
          kind: "hints",
          watermark: 2,
          targets: [
            { kind: "workspace_action", action_id: actionId },
            { kind: "workspaces", repository_id: repositoryId },
          ],
        },
      },
    }),
  );
  await waitFor(() =>
    expect(
      localStorage.getItem("agent-workspace.pending-workspace"),
    ).toBeNull(),
  );
  expect(actions()).toHaveLength(2);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) =>
        ["repository_request", "workspace_request"].includes(command),
      ),
  ).toHaveLength(2);
  expect(history.location.pathname).toBe("/settings/general");
});
