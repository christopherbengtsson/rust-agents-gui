import { incarnationBound } from "./test-support/incarnation-bound-invoke";
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
import { listen } from "@tauri-apps/api/event";
import { App } from "./test-support/app";
import { CoreConnection } from "./components/blocks/core-connection";
import { decodeSnapshot } from "./lib/observation-contracts";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  Channel: class {
    onmessage: (value: unknown) => void;
    constructor(receive: (value: unknown) => void) {
      this.onmessage = receive;
    }
  },
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}));
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
    .mock.calls.filter(
      ([command]) => command === "core_status" || command === "desktop_pid",
    );
}
function mockConnection(implementation: (command: string) => Promise<unknown>) {
  vi.mocked(invoke).mockImplementation(
    incarnationBound((command) =>
      command === "observation_stop" || command === "observation_acknowledge"
        ? Promise.resolve(undefined)
        : command === "repository_request"
          ? Promise.resolve({
              kind: "repositories",
              page: { repositories: [], next: null },
            })
          : implementation(command),
    ),
  );
}

const snapshot = {
  status: { state: "ready", native_pid: 41, application_version: "0.1.0" },
  cursor: { incarnation: "incarnation-41", revision: 1 },
  reset: false,
};

test("late shutdown-listener setup is physically released after the root unmounts", async () => {
  let finish: ((stop: () => void) => void) | undefined;
  vi.mocked(listen).mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve;
    }),
  );
  mockConnection(async (command) =>
    command === "desktop_pid" ? 42 : snapshot,
  );
  const mounted = render(<App />);
  await waitFor(() => expect(listen).toHaveBeenCalledTimes(1));
  mounted.unmount();
  const stop = vi.fn();
  await act(async () => {
    finish?.(stop);
  });
  expect(stop).toHaveBeenCalledTimes(1);
});

test("a failed shutdown-listener registration remains visible independently of core status", async () => {
  vi.mocked(listen).mockRejectedValueOnce(
    new Error("controlled registration failure"),
  );
  mockConnection(async (command) =>
    command === "desktop_pid" ? 42 : snapshot,
  );
  render(<App />);
  expect(await screen.findByText("Owned shutdown")).toBeTruthy();
  expect(
    screen.getByText(/Desktop shutdown updates are unavailable/),
  ).toBeTruthy();
});

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
      "core_status",
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
    name: "Native core diagnostics",
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
  ).toEqual(["core_status", "desktop_pid"]);

  await user.click(screen.getByRole("button", { name: "Connection details" }));
  expect(screen.getByRole("dialog").textContent).toContain(
    "Unsafe socket <details>",
  );
  await user.keyboard("{Escape}");
  await waitFor(() =>
    expect(document.activeElement).toBe(
      screen.getByRole("button", { name: "Connection details" }),
    ),
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
  expect(screen.getByRole("dialog").textContent).toContain("incarnation-41");
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
  rejectStatus("Connection lost; reopen to reconcile the incarnation");
  await screen.findByRole("alert");
  await user.click(screen.getByRole("button", { name: "Connection details" }));
  expect(
    screen.getByRole("region", { name: "Last successful observation" })
      .textContent,
  ).toContain("may be stale");
  expect(screen.getByRole("dialog").textContent).toContain("incarnation-41");
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
    "core_status",
    "desktop_pid",
    "core_status",
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
    "Cannot reach the core",
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
    await waitFor(() => expect(statusReads).toBe(1)); // Cleanup cannot release a physical status exchange.
    await act(async () => {
      if (outcome === "success") resolveOld(snapshot);
      else rejectOld("Obsolete setup failed");
    });
    expect(screen.getByRole("status").textContent).toContain("Checking");
    expect(screen.queryByRole("alert")).toBeNull();
    await user.click(screen.getByRole("button", { name: "Checking…" }));
    expect(connectionCalls().map(([command]) => command)).toEqual([
      "core_status",
      "desktop_pid",
      "core_status",
      "desktop_pid",
    ]);

    await act(async () => {
      resolveCurrent({
        ...snapshot,
        cursor: { incarnation: "current-incarnation", revision: 2 },
      });
    });
    expect(screen.getByRole("status").textContent).toContain("Connected");
    await user.click(
      screen.getByRole("button", { name: "Connection details" }),
    );
    expect(screen.getByRole("dialog").textContent).toContain(
      "current-incarnation",
    );
    expect(screen.getByRole("dialog").textContent).not.toContain(
      "incarnation-41",
    );
  },
);

test.each(["success", "failure"])(
  "a drained obsolete %s cannot replace the subsequent current observation",
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
            cursor: { incarnation: "current-incarnation", revision: 2 },
          });
    });
    render(
      <StrictMode>
        <App />
      </StrictMode>,
    );
    await waitFor(() => expect(reads).toBe(1));
    await act(async () => {
      if (outcome === "success") resolveOld(snapshot);
      else rejectOld("Obsolete failure");
    });
    await screen.findByRole("button", { name: "Refresh status" });
    expect(screen.queryByRole("alert")).toBeNull();
    const user = userEvent.setup();
    await user.click(
      screen.getByRole("button", { name: "Connection details" }),
    );
    expect(screen.getByRole("dialog").textContent).toContain(
      "current-incarnation",
    );
    expect(screen.getByRole("dialog").textContent).not.toContain(
      "incarnation-41",
    );
    expect(connectionCalls().map(([command]) => command)).toEqual([
      "core_status",
      "desktop_pid",
      "core_status",
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
    throw new Error("Incompatible core versions");
  });
  const view = render(<App />);
  await screen.findByRole("button", { name: "Retry connection" });
  expect(screen.getByText("Repository list unavailable")).toBeDefined();
  expect(
    view.container.querySelector('[data-slot="sidebar-menu-skeleton"]'),
  ).toBeNull();
});

test("the app mounts one feed and exposes native delivery failure without replaying actions", async () => {
  mockConnection(async (command) => {
    if (command === "desktop_pid") return 42;
    if (command === "observation_subscribe")
      throw new Error("Controlled subscription failure");
    return snapshot;
  });
  render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
  await screen.findByText("Observation delivery stopped");
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === "observation_subscribe"),
  ).toHaveLength(1);
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Connection details" }));
  expect(screen.getByRole("alert").textContent).toContain(
    "Controlled subscription failure",
  );
  expect(
    vi
      .mocked(invoke)
      .mock.calls.some(([command]) => command === "workspace_request"),
  ).toBe(false);
});

test("feed failures stay visible while native cleanup is still pending", async () => {
  render(
    <CoreConnection
      connection={{
        state: "connected",
        observation: { snapshot: decodeSnapshot(snapshot), desktopPid: 42 },
        feed: {
          state: "stopping",
          message: "Observation delivery stalled. Native cleanup pending.",
        },
      }}
    />,
  );
  expect(screen.getByRole("status").textContent).toContain(
    "Observation delivery stopped",
  );
  await userEvent
    .setup()
    .click(screen.getByRole("button", { name: "Connection details" }));
  expect(screen.getByRole("alert").textContent).toContain(
    "Native cleanup pending",
  );
});

test("presentation storage failure stays visible without a mounted inspection view", async () => {
  localStorage.setItem("agent-workspace.presentation", "{invalid");
  mockConnection(async (command) =>
    command === "desktop_pid" ? 42 : snapshot,
  );
  render(<App />);
  expect(
    await screen.findByText(
      /could not read or migrate presentation preferences/,
    ),
  ).toBeDefined();
  expect(screen.queryByLabelText("Selected file changes")).toBeNull();
  expect(localStorage.getItem("agent-workspace.presentation")).toBe("{invalid");
});
