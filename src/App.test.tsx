import { StrictMode } from "react";
import {
  act,
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { App } from "./App";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => {
  localStorage.clear();
  vi.stubGlobal("matchMedia", (media: string): MediaQueryList =>
    Object.assign(new EventTarget(), {
      media,
      matches: window.innerWidth < 768,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
    }),
  );
});
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

function connectionCalls() {
  return vi
    .mocked(invoke)
    .mock.calls.filter(([command]) => command !== "repository_request");
}
function mockConnection(implementation: (command: string) => Promise<unknown>) {
  vi.mocked(invoke).mockImplementation((command) =>
    command === "repository_request"
      ? Promise.resolve({
          kind: "repositories",
          page: { repositories: [], next: null },
        })
      : implementation(command),
  );
}

const snapshot = {
  status: { state: "ready", supervisor_pid: 41, application_version: "0.1.0" },
  cursor: { owner: "owner-41", revision: 1 },
  reset: false,
};

test.each([600, 900])(
  "the fixed shell preserves landmarks and empty navigation at width %i without global collapse or cookies",
  async (width) => {
    vi.stubGlobal("innerWidth", width);
    const writeCookie = vi.spyOn(document, "cookie", "set");
    mockConnection(async (command) =>
      command === "desktop_pid" ? 42 : snapshot,
    );
    render(<App />);
    await screen.findByRole("button", { name: "Refresh status" });

    const sidebar = screen.getByRole("complementary", {
      name: "Repository sidebar",
    });
    const navigation = within(sidebar).getByRole("navigation", {
      name: "Repositories",
    });
    expect(
      within(navigation).getByRole("heading", { level: 2 }).textContent,
    ).toBe("Repositories");
    expect(
      await within(navigation).findByText("No repositories yet"),
    ).toBeDefined();
    expect(within(navigation).queryAllByRole("link")).toHaveLength(0);
    expect(within(navigation).queryAllByRole("button")).toHaveLength(0);
    const workspace = screen.getByRole("main");
    expect(workspace.id).toBe("workspace");
    expect(workspace.tabIndex).toBe(-1);
    expect(
      screen
        .getByRole("link", { name: "Skip to workspace" })
        .getAttribute("href"),
    ).toBe("#workspace");
    expect(
      within(workspace).getByRole("heading", { level: 2 }).textContent,
    ).toBe("A place for your next change");
    expect(
      screen.getAllByRole("button").map((button) => button.textContent),
    ).toEqual(["Add repository", "Connection details", "Refresh status"]);

    for (const modifier of ["metaKey", "ctrlKey"]) {
      const shortcut = new KeyboardEvent("keydown", {
        key: "b",
        [modifier]: true,
        bubbles: true,
        cancelable: true,
      });
      window.dispatchEvent(shortcut);
      expect(shortcut.defaultPrevented).toBe(false);
    }
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(writeCookie).not.toHaveBeenCalled();
    expect(connectionCalls().map(([command]) => command)).toEqual([
      "supervisor_status",
      "desktop_pid",
    ]);
  },
);

test("diagnostics opens from the keyboard and Escape returns focus without reading status again", async () => {
  const user = userEvent.setup();
  mockConnection(async (command) =>
    command === "desktop_pid" ? 42 : snapshot,
  );
  render(<App />);
  await screen.findByRole("button", { name: "Refresh status" });
  await user.tab();
  expect(document.activeElement).toBe(
    screen.getByRole("link", { name: "Skip to workspace" }),
  );
  await user.tab();
  await user.tab();
  const trigger = screen.getByRole("button", { name: "Connection details" });
  expect(document.activeElement).toBe(trigger);
  await user.keyboard("{Enter}");
  const dialog = await screen.findByRole("dialog", {
    name: "Supervisor connection",
  });
  await waitFor(() =>
    expect(dialog.contains(document.activeElement)).toBe(true),
  );
  await user.keyboard("{Escape}");
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(document.activeElement).toBe(trigger);
  expect(connectionCalls()).toHaveLength(2);
});

test("connection failure stays visible until an explicit status retry; diagnostics never replay work", async () => {
  const user = userEvent.setup();
  mockConnection(async (command) => {
    if (command === "desktop_pid") return 42;
    throw "Unsafe socket <details>: restore mode 600";
  });
  render(<App />);
  expect((await screen.findByRole("alert")).textContent).toContain(
    "Unsafe socket <details>",
  );
  expect(screen.getByRole("status").textContent).toContain("Connection error");
  expect(
    connectionCalls()
      .map(([command]) => command)
      .sort(),
  ).toEqual(["desktop_pid", "supervisor_status"]);

  await user.click(screen.getByRole("button", { name: "Connection details" }));
  expect(screen.getByRole("dialog").textContent).toContain(
    "Unsafe socket <details>",
  );
  await user.keyboard("{Escape}");
  expect(document.activeElement).toBe(
    screen.getByRole("button", { name: "Connection details" }),
  );
  expect(connectionCalls()).toHaveLength(2);

  mockConnection(async (command) =>
    command === "desktop_pid" ? 42 : snapshot,
  );
  await user.click(screen.getByRole("button", { name: "Retry connection" }));
  expect((await screen.findByRole("status")).textContent).toContain(
    "Connected",
  );
  expect(screen.queryByRole("alert")).toBeNull();
  await user.click(screen.getByRole("button", { name: "Connection details" }));
  expect(screen.getByRole("dialog").textContent).toContain("owner-41");
  expect(connectionCalls()).toHaveLength(4);
});

test("refresh exposes pending feedback, blocks duplicate reads, and labels retained diagnostics as stale after failure", async () => {
  const user = userEvent.setup();
  mockConnection(async (command) =>
    command === "desktop_pid" ? 42 : snapshot,
  );
  render(<App />);
  await screen.findByRole("button", { name: "Refresh status" });
  let rejectStatus!: (error: string) => void;
  mockConnection((command) =>
    command === "desktop_pid"
      ? Promise.resolve(42)
      : new Promise((_, reject) => {
          rejectStatus = reject;
        }),
  );
  await user.click(screen.getByRole("button", { name: "Refresh status" }));
  const checking = screen.getByRole("button", { name: "Checking…" });
  expect(checking.hasAttribute("disabled")).toBe(true);
  await user.click(checking);
  expect(connectionCalls()).toHaveLength(4);
  rejectStatus("Connection lost; reopen to reconcile the owner");
  await screen.findByRole("alert");
  await user.click(screen.getByRole("button", { name: "Connection details" }));
  expect(
    screen.getByRole("region", { name: "Last successful observation" })
      .textContent,
  ).toContain("may be stale");
  expect(screen.getByRole("dialog").textContent).toContain("owner-41");
});

test("remounting retrieves observations and never submits execution actions", async () => {
  mockConnection(async (command) =>
    command === "desktop_pid" ? 42 : snapshot,
  );
  const first = render(<App />);
  await screen.findByRole("button", { name: "Refresh status" });
  first.unmount();
  render(<App />);
  await screen.findByRole("button", { name: "Refresh status" });
  expect(connectionCalls().map(([command]) => command)).toEqual([
    "supervisor_status",
    "desktop_pid",
    "supervisor_status",
    "desktop_pid",
  ]);
  expect(screen.queryByRole("button", { name: /send|resume/i })).toBeNull();
});

test("a connection failure with no error text still exposes the alert and retry", async () => {
  mockConnection(async (command) => {
    if (command === "desktop_pid") return 42;
    throw "";
  });
  render(<App />);
  expect((await screen.findByRole("alert")).textContent).toContain(
    "Cannot reach the supervisor",
  );
  expect(
    screen.getByRole("button", { name: "Retry connection" }),
  ).toBeDefined();
});

test.each(["success", "failure"])(
  "Strict Mode ignores a late %s from the cleaned-up observation request",
  async (outcome) => {
    const user = userEvent.setup();
    let resolveOld!: (value: typeof snapshot) => void;
    let rejectOld!: (error: string) => void;
    let resolveCurrent!: (value: typeof snapshot) => void;
    const oldStatus = new Promise<typeof snapshot>((resolve, reject) => {
      resolveOld = resolve;
      rejectOld = reject;
    });
    const currentStatus = new Promise<typeof snapshot>((resolve) => {
      resolveCurrent = resolve;
    });
    let statusReads = 0;
    mockConnection((command) => {
      if (command === "desktop_pid") return Promise.resolve(42);
      return statusReads++ === 0 ? oldStatus : currentStatus;
    });
    render(
      <StrictMode>
        <App />
      </StrictMode>,
    );
    await act(async () => {
      if (outcome === "success") resolveOld(snapshot);
      else rejectOld("Obsolete setup failed");
    });
    expect(screen.getByRole("status").textContent).toContain("Checking");
    expect(screen.queryByRole("alert")).toBeNull();
    await user.click(screen.getByRole("button", { name: "Checking…" }));
    expect(connectionCalls().map(([command]) => command)).toEqual([
      "supervisor_status",
      "desktop_pid",
      "supervisor_status",
      "desktop_pid",
    ]);

    await act(async () => {
      resolveCurrent({
        ...snapshot,
        cursor: { owner: "current-owner", revision: 2 },
      });
    });
    expect(screen.getByRole("status").textContent).toContain("Connected");
    await user.click(
      screen.getByRole("button", { name: "Connection details" }),
    );
    expect(screen.getByRole("dialog").textContent).toContain("current-owner");
    expect(screen.getByRole("dialog").textContent).not.toContain("owner-41");
  },
);

test.each(["success", "failure"])(
  "an obsolete %s cannot overwrite a completed current observation",
  async (outcome) => {
    let resolveOld!: (value: typeof snapshot) => void;
    let rejectOld!: (error: string) => void;
    const oldStatus = new Promise<typeof snapshot>((resolve, reject) => {
      resolveOld = resolve;
      rejectOld = reject;
    });
    let reads = 0;
    mockConnection((command) => {
      if (command === "desktop_pid") return Promise.resolve(42);
      return reads++ === 0
        ? oldStatus
        : Promise.resolve({
            ...snapshot,
            cursor: { owner: "current-owner", revision: 2 },
          });
    });
    render(
      <StrictMode>
        <App />
      </StrictMode>,
    );
    await screen.findByRole("button", { name: "Refresh status" });
    await act(async () => {
      if (outcome === "success") resolveOld(snapshot);
      else rejectOld("Obsolete failure");
    });
    expect(screen.queryByRole("alert")).toBeNull();
    const user = userEvent.setup();
    await user.click(
      screen.getByRole("button", { name: "Connection details" }),
    );
    expect(screen.getByRole("dialog").textContent).toContain("current-owner");
    expect(screen.getByRole("dialog").textContent).not.toContain("owner-41");
    expect(connectionCalls().map(([command]) => command)).toEqual([
      "supervisor_status",
      "desktop_pid",
      "supervisor_status",
      "desktop_pid",
    ]);
  },
);

test("a failed desktop identity read releases the pending guard for explicit retry", async () => {
  mockConnection(async (command) => {
    if (command === "desktop_pid") throw "Identity unavailable";
    return snapshot;
  });
  render(<App />);
  await screen.findByRole("alert");
  mockConnection(async (command) =>
    command === "desktop_pid" ? 42 : snapshot,
  );
  await userEvent
    .setup()
    .click(screen.getByRole("button", { name: "Retry connection" }));
  await screen.findByRole("button", { name: "Refresh status" });
  expect(screen.queryByRole("alert")).toBeNull();
  expect(connectionCalls()).toHaveLength(4);
});

test("connection failure replaces initial repository skeletons with unavailable guidance", async () => {
  mockConnection(async () => {
    throw new Error("Incompatible supervisor versions");
  });
  const view = render(<App />);
  await screen.findByRole("button", { name: "Retry connection" });
  expect(screen.getByText("Repository list unavailable")).toBeDefined();
  expect(
    view.container.querySelector('[data-slot="sidebar-menu-skeleton"]'),
  ).toBeNull();
});
