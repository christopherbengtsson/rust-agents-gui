import {
  act,
  cleanup,
  fireEvent,
  render,
  renderHook,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { App } from "./App";
import { StrictMode } from "react";
import { useRepositories } from "./hooks/use-repositories";
import {
  PENDING_KEY,
  SELECTION_KEY,
  object,
  type Repository,
  type RepositoryRequest,
  type RepositoryReply,
} from "./lib/repositories";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
const id = "00000000-0000-4000-8000-000000000001";
const repository: Repository = {
  id,
  name: "one",
  path: "/projects/one",
  availability: { state: "available" },
};
const snapshot = {
  status: { state: "ready", supervisor_pid: 41, application_version: "0.1.0" },
  cursor: { owner: "owner-41", revision: 1 },
  reset: true,
};
beforeEach(() => {
  localStorage.clear();
  vi.stubGlobal("matchMedia", () =>
    Object.assign(new EventTarget(), {
      matches: false,
      addListener() {},
      removeListener() {},
    }),
  );
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "workspace_request")
      return { kind: "workspaces", page: { workspaces: [], next: null } };
    if (command === "desktop_pid") return 42;
    if (command === "supervisor_status") return snapshot;
    return { kind: "repositories", page: { repositories: [], next: null } };
  });
});
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
  vi.unstubAllGlobals();
});

test("the native picker fills a path and only explicit registration submits it", async () => {
  const user = userEvent.setup();
  render(<App />);
  await user.click(
    await screen.findByRole("button", { name: "Add repository" }),
  );
  vi.mocked(open).mockResolvedValue("/projects/one");
  await user.click(screen.getByRole("button", { name: "Browse…" }));
  expect(
    (
      screen.getByRole("textbox", {
        name: "Repository folder",
      }) as HTMLInputElement
    ).value,
  ).toBe("/projects/one");
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([, args]) =>
        JSON.stringify(args ?? {}).includes('"operation":"register"'),
      ),
  ).toHaveLength(0);
  expect(vi.mocked(open)).toHaveBeenCalledWith({
    directory: true,
    multiple: false,
    title: "Choose an existing Git repository",
  });
});

function replies(
  handler: (request: RepositoryRequest) => Promise<RepositoryReply>,
) {
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === "workspace_request")
      return { kind: "workspaces", page: { workspaces: [], next: null } };
    if (command === "desktop_pid") return 42;
    if (command === "supervisor_status") return snapshot;
    return handler(
      (object(args) ? args.request : undefined) as RepositoryRequest,
    );
  });
}
function registrations() {
  return vi
    .mocked(invoke)
    .mock.calls.filter(
      ([, args]) =>
        object(args) &&
        object(args.request) &&
        args.request.operation === "register",
    );
}

test("picker cancellation preserves the draft and picker failure allows manual entry", async () => {
  const user = userEvent.setup();
  render(<App />);
  await user.click(
    await screen.findByRole("button", { name: "Add repository" }),
  );
  const path = screen.getByRole("textbox", { name: "Repository folder" });
  await user.type(path, "/manual");
  vi.mocked(open).mockResolvedValue(null);
  await user.click(screen.getByRole("button", { name: "Browse…" }));
  expect((path as HTMLInputElement).value).toBe("/manual");
  vi.mocked(open).mockRejectedValue(new Error("picker unavailable"));
  await user.click(screen.getByRole("button", { name: "Browse…" }));
  expect((await screen.findByRole("dialog")).textContent).toContain(
    "Enter the absolute folder path instead",
  );
  expect(path.hasAttribute("disabled")).toBe(false);
  expect(registrations()).toHaveLength(0);
});

test("restored selection remains selected while its details are collapsed", async () => {
  const user = userEvent.setup();
  localStorage.setItem(SELECTION_KEY, id);
  replies(async (request) =>
    request.operation === "get"
      ? { kind: "repository", repository }
      : {
          kind: "repositories",
          page: { repositories: [repository], next: null },
        },
  );
  render(<App />);
  await screen.findByRole("heading", { level: 1, name: "one" });
  const selected = screen.getByRole("button", { name: "one" });
  expect(selected.getAttribute("aria-current")).toBe("true");
  const toggle = screen.getByRole("button", { name: "Toggle details for one" });
  expect(toggle.getAttribute("aria-expanded")).toBe("true");
  await user.click(toggle);
  await waitFor(() =>
    expect(toggle.getAttribute("aria-expanded")).toBe("false"),
  );
  expect(
    within(
      screen.getByRole("region", { name: "Selected repository" }),
    ).getByText("/projects/one"),
  ).toBeDefined();
  expect(selected.getAttribute("aria-current")).toBe("true");
  expect(localStorage.getItem(SELECTION_KEY)).toBe(id);
  expect(registrations()).toHaveLength(0);
});

test("registration selects the durable result and never submits twice", async () => {
  const user = userEvent.setup();
  let registered = false;
  replies(async (request) => {
    if (request.operation === "register") {
      registered = true;
      return {
        kind: "registration",
        registration: {
          ...request.request,
          state: "completed",
          repository_id: id,
        },
      };
    }
    if (request.operation === "get") return { kind: "repository", repository };
    return {
      kind: "repositories",
      page: { repositories: registered ? [repository] : [], next: null },
    };
  });
  render(<App />);
  await user.click(
    await screen.findByRole("button", { name: "Add repository" }),
  );
  await user.type(
    screen.getByRole("textbox", { name: "Repository folder" }),
    "/projects/one",
  );
  await user.click(screen.getByRole("button", { name: /^Register$/ }));
  await screen.findByRole("heading", { level: 1, name: "one" });
  expect(registrations()).toHaveLength(1);
  expect(localStorage.getItem(SELECTION_KEY)).toBe(id);
  expect(localStorage.getItem(PENDING_KEY)).toBeNull();
});

test("reopening with a retained action retrieves its result without resubmitting", async () => {
  const action_id = "00000000-0000-4000-8000-000000000010";
  localStorage.setItem(
    PENDING_KEY,
    JSON.stringify({
      request: { action_id, path: "/projects/one" },
      selection: null,
    }),
  );
  replies(async (request) => {
    if (request.operation === "action")
      return {
        kind: "action",
        registration: {
          action_id,
          path: "/projects/one",
          state: "completed",
          repository_id: id,
        },
      };
    if (request.operation === "get") return { kind: "repository", repository };
    return {
      kind: "repositories",
      page: { repositories: [repository], next: null },
    };
  });
  render(<App />);
  await screen.findByRole("heading", { level: 1, name: "one" });
  expect(registrations()).toHaveLength(0);
  expect(localStorage.getItem(PENDING_KEY)).toBeNull();
});

test("unavailable repositories stay selected and storage failures retain the last observation", async () => {
  const user = userEvent.setup();
  const missing: Repository = {
    ...repository,
    availability: {
      state: "unavailable",
      error: {
        code: "invalid_path",
        message: "Restore /projects/one and refresh",
      },
    },
  };
  let fail = false;
  replies(async (request) => {
    if (fail)
      throw {
        code: "storage",
        message: "Disk unavailable; restore private storage",
      };
    if (request.operation === "get")
      return { kind: "repository", repository: missing };
    return {
      kind: "repositories",
      page: { repositories: [missing], next: null },
    };
  });
  render(<App />);
  await screen.findByRole("heading", { level: 1, name: "one" });
  expect(
    screen.getByRole("button", { name: "one" }).hasAttribute("disabled"),
  ).toBe(false);
  fail = true;
  await user.click(screen.getByRole("button", { name: "Refresh status" }));
  await screen.findByText("Disk unavailable; restore private storage");
  expect(screen.getByRole("heading", { level: 1, name: "one" })).toBeDefined();
  expect(screen.getByText(/may be stale/)).toBeDefined();
});

test("a late folder-picker response cannot overwrite a reopened dialog", async () => {
  const user = userEvent.setup();
  let resolve!: (value: string) => void;
  vi.mocked(open).mockImplementation(
    () =>
      new Promise((r) => {
        resolve = r;
      }),
  );
  render(<App />);
  await user.click(
    await screen.findByRole("button", { name: "Add repository" }),
  );
  await user.type(
    screen.getByRole("textbox", { name: "Repository folder" }),
    "/keep",
  );
  await user.click(screen.getByRole("button", { name: "Browse…" }));
  await user.keyboard("{Escape}");
  await user.click(screen.getByRole("button", { name: "Add repository" }));
  await act(async () => resolve("/late"));
  expect(
    (
      screen.getByRole("textbox", {
        name: "Repository folder",
      }) as HTMLInputElement
    ).value,
  ).toBe("/keep");
});

test("invalid registration permits correction and preserves an actionable error", async () => {
  const user = userEvent.setup();
  replies(async (request) => {
    if (request.operation === "register")
      throw {
        code: "invalid_path",
        message: "Choose an absolute existing folder",
      };
    return { kind: "repositories", page: { repositories: [], next: null } };
  });
  render(<App />);
  await user.click(
    await screen.findByRole("button", { name: "Add repository" }),
  );
  const path = screen.getByRole("textbox", { name: "Repository folder" });
  await user.type(path, "relative");
  await user.click(screen.getByRole("button", { name: /^Register$/ }));
  await waitFor(() =>
    expect(screen.getByRole("dialog").textContent).toContain(
      "Choose an absolute existing folder",
    ),
  );
  expect(path.hasAttribute("disabled")).toBe(false);
  expect(localStorage.getItem(PENDING_KEY)).toBeNull();
  expect(registrations()).toHaveLength(1);
});

test("lost registration reply reconciles the retained action without another submit", async () => {
  const user = userEvent.setup();
  let saved: { action_id: string; path: string } | undefined;
  replies(async (request) => {
    if (request.operation === "register") {
      saved = request.request;
      throw new Error("reply lost");
    }
    if (request.operation === "action")
      return {
        kind: "action",
        registration: saved
          ? { ...saved, state: "completed", repository_id: id }
          : null,
      };
    if (request.operation === "get") return { kind: "repository", repository };
    return {
      kind: "repositories",
      page: { repositories: saved ? [repository] : [], next: null },
    };
  });
  render(<App />);
  await user.click(
    await screen.findByRole("button", { name: "Add repository" }),
  );
  await user.type(
    screen.getByRole("textbox", { name: "Repository folder" }),
    "/projects/one",
  );
  await user.click(screen.getByRole("button", { name: /^Register$/ }));
  await user.keyboard("{Escape}");
  await screen.findByRole("heading", { level: 1, name: "one" });
  expect(registrations()).toHaveLength(1);
  expect(localStorage.getItem(PENDING_KEY)).toBeNull();
});

test.each([false, true])(
  "capacity rejection allows correction of an oversized registration (restored: %s)",
  async (restored) => {
    const user = userEvent.setup();
    const oversized = `/${"a".repeat(64 * 1024)}`;
    const action_id = "00000000-0000-4000-8000-000000000010";
    if (restored) {
      localStorage.setItem(
        PENDING_KEY,
        JSON.stringify({
          request: { action_id, path: oversized },
          selection: null,
        }),
      );
    }
    const submitted: { action_id: string; path: string }[] = [];
    replies(async (request) => {
      if (request.operation === "register") {
        submitted.push(request.request);
        if (request.request.path === oversized) {
          throw {
            code: "capacity",
            message: "Choose a path within 4096 UTF-8 bytes",
          };
        }
        return {
          kind: "registration",
          registration: {
            ...request.request,
            state: "completed",
            repository_id: id,
          },
        };
      }
      if (request.operation === "action")
        return { kind: "action", registration: null };
      if (request.operation === "get")
        return { kind: "repository", repository };
      return { kind: "repositories", page: { repositories: [], next: null } };
    });
    render(<App />);
    if (restored) {
      await screen.findByText(/no recorded registration/);
      expect(registrations()).toHaveLength(0);
    }
    await user.click(
      await screen.findByRole("button", { name: "Add repository" }),
    );
    const path = screen.getByRole("textbox", { name: "Repository folder" });
    if (restored) {
      expect((path as HTMLInputElement).value).toBe(oversized);
      expect(path.hasAttribute("disabled")).toBe(true);
    } else {
      fireEvent.change(path, { target: { value: oversized } });
    }
    await user.click(
      screen.getByRole("button", {
        name: restored ? "Retry registration" : /^Register$/,
      }),
    );
    await waitFor(() => expect(localStorage.getItem(PENDING_KEY)).toBeNull());
    expect(path.hasAttribute("disabled")).toBe(false);
    expect(
      within(screen.getByRole("dialog")).getByText(/within 4096 UTF-8 bytes/),
    ).toBeDefined();
    expect(submitted).toHaveLength(1);
    expect(submitted[0].path).toBe(oversized);
    if (restored) expect(submitted[0].action_id).toBe(action_id);
    fireEvent.change(path, { target: { value: repository.path } });
    await user.click(screen.getByRole("button", { name: /^Register$/ }));
    await screen.findByRole("heading", { level: 1, name: "one" });
    expect(submitted).toHaveLength(2);
    expect(submitted[1].path).toBe(repository.path);
    expect(submitted[1].action_id).not.toBe(submitted[0].action_id);
    expect(localStorage.getItem(PENDING_KEY)).toBeNull();
  },
);

test("selection during registration is not retargeted by its late completion", async () => {
  const user = userEvent.setup();
  const two: Repository = {
    ...repository,
    id: "00000000-0000-4000-8000-000000000002",
    name: "two",
    path: "/projects/two",
  };
  let resolve!: (reply: RepositoryReply) => void;
  let submitted!: { action_id: string; path: string };
  replies(async (request) => {
    if (request.operation === "register") {
      submitted = request.request;
      return new Promise((r) => {
        resolve = r;
      });
    }
    if (request.operation === "action")
      return {
        kind: "action",
        registration: { ...submitted, state: "pending" },
      };
    if (request.operation === "get")
      return {
        kind: "repository",
        repository: request.id === id ? repository : two,
      };
    return {
      kind: "repositories",
      page: { repositories: [repository, two], next: null },
    };
  });
  render(<App />);
  await screen.findByRole("heading", { level: 1, name: "one" });
  await user.click(screen.getByRole("button", { name: "Add repository" }));
  await user.type(
    screen.getByRole("textbox", { name: "Repository folder" }),
    "/projects/new",
  );
  await user.click(screen.getByRole("button", { name: /^Register$/ }));
  await user.keyboard("{Escape}");
  await user.click(screen.getByRole("button", { name: "two" }));
  await act(async () =>
    resolve({
      kind: "registration",
      registration: { ...submitted, state: "completed", repository_id: id },
    }),
  );
  await waitFor(() => expect(localStorage.getItem(PENDING_KEY)).toBeNull());
  expect(screen.getByRole("heading", { level: 1, name: "two" })).toBeDefined();
  expect(registrations()).toHaveLength(1);
});

test("cursor paging retains selection and previous returns to the original page", async () => {
  const user = userEvent.setup();
  const two: Repository = {
    ...repository,
    id: "00000000-0000-4000-8000-000000000002",
    name: "two",
  };
  replies(async (request) => {
    if (request.operation === "get") return { kind: "repository", repository };
    return {
      kind: "repositories",
      page: {
        repositories:
          request.operation === "list" && request.after ? [two] : [repository],
        next: request.operation === "list" && request.after ? null : id,
      },
    };
  });
  render(<App />);
  await screen.findByRole("heading", { level: 1, name: "one" });
  await user.click(screen.getByRole("button", { name: "Next" }));
  await screen.findByRole("button", { name: "two" });
  expect(screen.getByRole("heading", { level: 1, name: "one" })).toBeDefined();
  await user.click(screen.getByRole("button", { name: "Previous" }));
  await screen.findByRole("button", { name: "one" });
  expect(
    screen.getByRole("button", { name: "Previous" }).hasAttribute("disabled"),
  ).toBe(true);
});

test("malformed repository payloads stay visible rather than becoming empty navigation", async () => {
  vi.mocked(invoke).mockImplementation(async (command) =>
    command === "desktop_pid"
      ? 42
      : command === "supervisor_status"
        ? snapshot
        : {
            kind: "repositories",
            page: { repositories: [{ id: "bad" }], next: null },
          },
  );
  render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
  await screen.findByText(/invalid repository observation/);
  expect(screen.getByText("Repository list unavailable")).toBeDefined();
  expect(screen.queryByText("No repositories yet")).toBeNull();
  expect(registrations()).toHaveLength(0);
});

test("a missing action lookup preserves the storage failure that blocked intent", async () => {
  const user = userEvent.setup();
  replies(async (request) => {
    if (request.operation === "register")
      throw {
        code: "storage",
        message: "Cannot write private storage; restore its permissions",
      };
    if (request.operation === "action")
      return { kind: "action", registration: null };
    return { kind: "repositories", page: { repositories: [], next: null } };
  });
  render(<App />);
  await user.click(
    await screen.findByRole("button", { name: "Add repository" }),
  );
  await user.type(
    screen.getByRole("textbox", { name: "Repository folder" }),
    "/projects/one",
  );
  await user.click(screen.getByRole("button", { name: /^Register$/ }));
  await waitFor(() =>
    expect(
      within(screen.getByRole("dialog")).getByText(
        "Cannot write private storage; restore its permissions",
      ),
    ).toBeDefined(),
  );
  expect(registrations()).toHaveLength(1);
  expect(localStorage.getItem(PENDING_KEY)).not.toBeNull();
});

test.each(["success", "failure"])(
  "reconnect ignores the previous owner's late observation without repeating actions (%s)",
  async (outcome) => {
    const initialProps: { owner: string | undefined } = { owner: "owner-a" };
    const fresh: Repository = { ...repository, name: "fresh observation" };
    let stage: "initial" | "old" | "new" = "initial";
    let rejectOld!: (error: unknown) => void;
    let resolveOld!: (reply: RepositoryReply) => void;
    const requests: RepositoryRequest[] = [];
    replies(async (request) => {
      requests.push(request);
      if (request.operation === "get")
        return {
          kind: "repository",
          repository: stage === "new" ? fresh : repository,
        };
      if (stage === "old")
        return new Promise((resolve, reject) => {
          resolveOld = resolve;
          rejectOld = reject;
        });
      return {
        kind: "repositories",
        page: {
          repositories: [stage === "new" ? fresh : repository],
          next: null,
        },
      };
    });
    const { result, rerender } = renderHook(
      ({ owner }: { owner: string | undefined }) => useRepositories(owner),
      { initialProps },
    );
    await waitFor(() => expect(result.current.selected?.name).toBe("one"));
    rerender({ owner: "owner-a" });
    expect(requests.filter((r) => r.operation === "list")).toHaveLength(1);

    rerender({ owner: undefined });
    stage = "old";
    rerender({ owner: "owner-a" });
    expect(requests.filter((r) => r.operation === "list")).toHaveLength(2);
    stage = "new";
    rerender({ owner: "owner-b" });
    await waitFor(() => expect(result.current.selected?.name).toBe(fresh.name));
    await act(async () => {
      if (outcome === "success") {
        resolveOld({
          kind: "repositories",
          page: { repositories: [repository], next: null },
        });
      } else rejectOld(new Error("old owner disconnected"));
    });
    expect(result.current.selected?.name).toBe(fresh.name);
    expect(result.current.error).toBeUndefined();
    expect(result.current.loading).toBe(false);
    expect(requests.filter((r) => r.operation === "list")).toHaveLength(3);
    expect(registrations()).toHaveLength(0);
  },
);

test("reconnect retrieves a retained registration result without resubmitting", async () => {
  const initialProps: { owner: string | undefined } = { owner: "owner-a" };
  const action_id = "00000000-0000-4000-8000-000000000010";
  localStorage.setItem(
    PENDING_KEY,
    JSON.stringify({
      request: { action_id, path: repository.path },
      selection: null,
    }),
  );
  let connected = false;
  const actions: string[] = [];
  replies(async (request) => {
    if (request.operation === "action") {
      actions.push(request.action_id);
      if (!connected)
        throw { code: "connection", message: "Owner unavailable" };
      return {
        kind: "action",
        registration: {
          action_id,
          path: repository.path,
          state: "completed",
          repository_id: id,
        },
      };
    }
    if (request.operation === "get") return { kind: "repository", repository };
    return {
      kind: "repositories",
      page: { repositories: [repository], next: null },
    };
  });
  const { result, rerender } = renderHook(
    ({ owner }: { owner: string | undefined }) => useRepositories(owner),
    { initialProps },
  );
  await waitFor(() => expect(result.current.loading).toBe(false));
  expect(result.current.actionError?.code).toBe("connection");
  expect(result.current.pending?.request.action_id).toBe(action_id);
  rerender({ owner: undefined });
  connected = true;
  rerender({ owner: "owner-b" });
  await waitFor(() => expect(result.current.pending).toBeNull());
  expect(result.current.selected?.id).toBe(id);
  expect(result.current.actionError).toBeUndefined();
  expect(localStorage.getItem(PENDING_KEY)).toBeNull();
  expect(actions).toEqual([action_id, action_id]);
  expect(registrations()).toHaveLength(0);
});

test("explicit repository refresh reports loading while retaining the last observation", async () => {
  let release!: (reply: RepositoryReply) => void;
  let defer = false;
  replies(async (request) => {
    if (request.operation === "get") return { kind: "repository", repository };
    if (defer)
      return new Promise((resolve) => {
        release = resolve;
      });
    return {
      kind: "repositories",
      page: { repositories: [repository], next: null },
    };
  });
  const { result } = renderHook(() => useRepositories("owner-a"));
  await waitFor(() => expect(result.current.loading).toBe(false));
  defer = true;
  let refreshing!: Promise<void>;
  act(() => {
    refreshing = result.current.refresh();
  });
  expect(result.current.loading).toBe(true);
  expect(result.current.selected?.id).toBe(id);
  await act(async () => {
    release({
      kind: "repositories",
      page: { repositories: [repository], next: null },
    });
    await refreshing;
  });
  expect(result.current.loading).toBe(false);
  expect(result.current.selected?.id).toBe(id);
  expect(registrations()).toHaveLength(0);
});
