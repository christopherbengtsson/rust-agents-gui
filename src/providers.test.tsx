import { afterEach, expect, test, vi } from "vitest";
import { StrictMode, type ReactNode } from "react";
import {
  act,
  cleanup,
  render,
  renderHook,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { decodeProviderReply } from "./lib/providers";
import { AppObservations } from "./lib/app-observations";
import type { ProviderTarget, SetupPhase } from "./generated/contracts/types";
import { ProviderSetup } from "./components/blocks/provider-setup";
import { useProviderSetup } from "./hooks/use-provider-setup";
import { incarnationBound } from "./test-support/incarnation-bound-invoke";
import { validateProviderRequest } from "./generated/contracts/validators";
import { object } from "./lib/observation-contracts";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const a = "00000000-0000-4000-8000-000000000001";
const b = "00000000-0000-4000-8000-000000000002";
const target: ProviderTarget = {
  id: a,
  provider: "codex",
  executable: "/fixture/codex",
  selected_executable: null,
  discovery_source: "fixture",
  codex_home: "/fixture/home",
  working_directory: "/fixture/neutral",
  minimum_supported_version: "0.169.0",
};
const observation = (id = a) => ({
  kind: "observation",
  observation: {
    target: { ...target, id },
    selection: null,
    revision: 2,
    phase: "observed",
    evidence: null,
    problem: "missing_executable",
    diagnostic: "synthetic missing executable",
    login: null,
  },
});
const flush = async () => {
  for (let i = 0; i < 35; i++) await Promise.resolve();
};
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
  localStorage.clear();
});

type ReadinessOutcome = {
  phase?: SetupPhase;
  fail?: boolean;
  wait?: Promise<void>;
};
function healthyBridge(outcome: ReadinessOutcome = {}) {
  vi.mocked(invoke).mockImplementation(
    incarnationBound(async (_command, args) => {
      if (!object(args) || !validateProviderRequest(args.request))
        throw new Error("Unexpected provider request");
      if (args.request.operation === "observe") await outcome.wait;
      if (args.request.operation === "observe" && outcome.fail)
        throw new Error("Controlled provider observation failure");
      return args.request.operation === "target"
        ? { kind: "target", target }
        : {
            kind: "observation",
            observation: {
              target,
              selection: null,
              revision: 2,
              phase: outcome.phase ?? "observed",
              problem: null,
              diagnostic: null,
              login: null,
              evidence: {
                version: "0.169.0",
                authentication: "chatgpt",
                billing: "chatgpt",
                account: "fixture@example.test",
                plan: "pro",
                model_provider: "openai",
                model: null,
              },
            },
          };
    }),
  );
}
function verificationRequests() {
  return vi
    .mocked(invoke)
    .mock.calls.filter(
      ([, args]) =>
        object(args) &&
        object(args.request) &&
        args.request.operation === "observe" &&
        args.request.refresh === true,
    );
}

test("one committed Strict Mode activation verifies once; renders, hints and reconnect only observe", async () => {
  healthyBridge();
  const app = new AppObservations();
  app.connect("incarnation-a");
  app.freshness.accept("incarnation-a");
  const initialProps: { incarnation: string | undefined } = {
    incarnation: "incarnation-a",
  };
  const hook = renderHook(
    ({ incarnation }) => useProviderSetup(app, incarnation),
    {
      initialProps,
      wrapper: ({ children }: { children: ReactNode }) => (
        <StrictMode>{children}</StrictMode>
      ),
    },
  );
  expect(hook.result.current.pending).toBe(true);
  await waitFor(() => expect(hook.result.current.pending).toBe(false));
  expect(hook.result.current.stale).toBe(false);
  expect(verificationRequests()).toHaveLength(1);
  expect(vi.mocked(invoke).mock.calls).toHaveLength(2);
  hook.rerender(initialProps);
  await act(flush);
  expect(verificationRequests()).toHaveLength(1);
  await act(async () => {
    app.providers.invalidate({ kind: "provider", target_id: a });
    app.freshness.accept("incarnation-a");
    await flush();
  });
  expect(verificationRequests()).toHaveLength(1);
  await act(async () => app.connect(undefined));
  hook.rerender({ incarnation: undefined });
  expect(hook.result.current.stale).toBe(true);
  await act(async () => {
    app.connect("incarnation-a");
    app.freshness.accept("incarnation-a");
  });
  hook.rerender(initialProps);
  await act(flush);
  expect(verificationRequests()).toHaveLength(1);
  hook.unmount();
  app.client.clear();
});

test("preload only gets the descriptor; reentry verifies retained healthy evidence anew", async () => {
  healthyBridge();
  const app = new AppObservations();
  app.connect("incarnation-a");
  app.freshness.accept("incarnation-a");
  await app.providers.prepare(
    "incarnation-a",
    { operation: "target" },
    new AbortController().signal,
  );
  expect(verificationRequests()).toHaveLength(0);
  expect(vi.mocked(invoke).mock.calls).toHaveLength(1);
  const first = renderHook(() => useProviderSetup(app, "incarnation-a"));
  await waitFor(() => expect(first.result.current.pending).toBe(false));
  first.unmount();
  const second = renderHook(() => useProviderSetup(app, "incarnation-a"));
  expect(second.result.current.observation?.evidence?.account).toBe(
    "fixture@example.test",
  );
  expect(second.result.current.pending).toBe(true);
  expect(second.result.current.stale).toBe(true);
  await waitFor(() => expect(second.result.current.pending).toBe(false));
  expect(verificationRequests()).toHaveLength(2);
  second.unmount();
  app.client.clear();
});

test("an entry verification awaiting baseline survives disconnect until its first dispatch", async () => {
  healthyBridge();
  const app = new AppObservations();
  app.connect("incarnation-a");
  app.freshness.accept("incarnation-a");
  const first = renderHook(() => useProviderSetup(app, "incarnation-a"));
  await waitFor(() => expect(first.result.current.pending).toBe(false));
  first.unmount();

  // Keep the descriptor/account cached, but hold the new connection baseline.
  await act(async () => app.connect("incarnation-a"));
  const initialProps: { incarnation: string | undefined } = {
    incarnation: "incarnation-a",
  };
  const next = renderHook(
    ({ incarnation }) => useProviderSetup(app, incarnation),
    {
      initialProps,
    },
  );
  await act(flush);
  expect(verificationRequests()).toHaveLength(1);
  expect(next.result.current.stale).toBe(true);
  await act(async () => app.connect(undefined));
  next.rerender({ incarnation: undefined });
  await act(flush);
  expect(verificationRequests()).toHaveLength(1);
  await act(async () => {
    app.connect("incarnation-a");
    app.freshness.accept("incarnation-a");
  });
  next.rerender(initialProps);
  await act(flush);
  expect(verificationRequests()).toHaveLength(2);
  expect(next.result.current.pending).toBe(false);
  expect(next.result.current.stale).toBe(false);
  next.unmount();
  app.client.clear();
});

test("leaving before baseline releases an unsent entry verification", async () => {
  healthyBridge();
  const app = new AppObservations();
  app.connect("incarnation-a");
  app.freshness.accept("incarnation-a");
  const first = renderHook(() => useProviderSetup(app, "incarnation-a"));
  await waitFor(() => expect(first.result.current.pending).toBe(false));
  first.unmount();
  await act(async () => app.connect("incarnation-a"));
  const next = renderHook(() => useProviderSetup(app, "incarnation-a"));
  await act(flush);
  next.unmount();
  await act(async () => {
    app.freshness.accept("incarnation-a");
    await flush();
  });
  expect(verificationRequests()).toHaveLength(1);
  app.client.clear();
});

test("a replacement incarnation gets its own entry verification over retained evidence", async () => {
  healthyBridge();
  const app = new AppObservations();
  for (const incarnation of ["incarnation-b", "incarnation-a"]) {
    app.connect(incarnation);
    app.freshness.accept(incarnation);
    const prior = renderHook(() => useProviderSetup(app, incarnation));
    await waitFor(() => expect(prior.result.current.pending).toBe(false));
    prior.unmount();
  }
  await act(async () => app.connect("incarnation-a"));
  const initialProps: { incarnation: string | undefined } = {
    incarnation: "incarnation-a",
  };
  const hook = renderHook(
    ({ incarnation }) => useProviderSetup(app, incarnation),
    {
      initialProps,
    },
  );
  await act(flush);
  await act(async () => app.connect(undefined));
  hook.rerender({ incarnation: undefined });
  await act(async () => {
    app.connect("incarnation-b");
    app.freshness.accept("incarnation-b");
  });
  hook.rerender({ incarnation: "incarnation-b" });
  expect(hook.result.current.pending).toBe(true);
  expect(hook.result.current.stale).toBe(true);
  await act(flush);
  expect(
    verificationRequests().filter(
      ([, args]) =>
        object(args) && args.expectedIncarnation === "incarnation-a",
    ),
  ).toHaveLength(1);
  expect(
    verificationRequests().filter(
      ([, args]) =>
        object(args) && args.expectedIncarnation === "incarnation-b",
    ),
  ).toHaveLength(2);
  hook.unmount();
  app.client.clear();
});

test("a failed observation after checking exposes recovery through the real setup view", async () => {
  const outcome: ReadinessOutcome = { phase: "checking" };
  healthyBridge(outcome);
  const app = new AppObservations();
  app.connect("incarnation-a");
  app.freshness.accept("incarnation-a");
  const hook = renderHook(() => useProviderSetup(app, "incarnation-a"));
  await waitFor(() =>
    expect(hook.result.current.observation?.phase).toBe("checking"),
  );
  outcome.fail = true;
  await act(async () => {
    app.providers.invalidate({ kind: "provider", target_id: a });
    app.freshness.accept("incarnation-a");
    await flush();
  });
  expect(hook.result.current.error).toBeTruthy();
  const block = render(
    <ProviderSetup setup={hook.result.current} onOpenHelp={() => {}} />,
  );
  expect(screen.getByRole("status").textContent).toBe("Couldn’t verify Codex");
  expect(
    screen.getByText(/Current authentication could not be verified/),
  ).toBeTruthy();
  outcome.fail = false;
  outcome.phase = "observed";
  let releaseRetry = () => {};
  outcome.wait = new Promise<void>((resolve) => {
    releaseRetry = resolve;
  });
  await userEvent
    .setup()
    .click(screen.getByRole("button", { name: "Check again" }));
  expect(hook.result.current.pending).toBe(true);
  block.rerender(
    <ProviderSetup setup={hook.result.current} onOpenHelp={() => {}} />,
  );
  expect(screen.getByRole("status").textContent).toBe("Checking Codex…");
  expect(screen.queryByRole("alert")).toBeNull();
  expect(screen.queryByRole("button", { name: "Check again" })).toBeNull();
  await act(async () => {
    releaseRetry();
    await flush();
  });
  await waitFor(() => expect(hook.result.current.pending).toBe(false));
  block.rerender(
    <ProviderSetup setup={hook.result.current} onOpenHelp={() => {}} />,
  );
  expect(screen.getByRole("status").textContent).toBe("Connected");
  expect(screen.queryByRole("button", { name: "Check again" })).toBeNull();
  hook.unmount();
  block.unmount();
  app.client.clear();
});

test("late target discovery after leaving cannot admit an entry check", async () => {
  const app = new AppObservations();
  app.connect("incarnation-a");
  app.freshness.accept("incarnation-a");
  let complete = () => {};
  vi.mocked(invoke).mockImplementation(
    incarnationBound(
      () =>
        new Promise((resolve) => {
          complete = () => resolve({ kind: "target", target });
        }),
    ),
  );
  const hook = renderHook(() => useProviderSetup(app, "incarnation-a"));
  await act(flush);
  hook.unmount();
  await act(async () => {
    complete();
    await flush();
  });
  expect(verificationRequests()).toHaveLength(0);
  expect(vi.mocked(invoke).mock.calls).toHaveLength(1);
  app.client.clear();
});

test("stopped delivery makes setup stale without replaying its entry verification", async () => {
  healthyBridge();
  const app = new AppObservations();
  app.connect("incarnation-a");
  app.freshness.accept("incarnation-a");
  const hook = renderHook(
    ({ updatesAvailable }) =>
      useProviderSetup(app, "incarnation-a", updatesAvailable),
    {
      initialProps: { updatesAvailable: true },
    },
  );
  await waitFor(() => expect(hook.result.current.pending).toBe(false));
  hook.rerender({ updatesAvailable: false });
  expect(hook.result.current.stale).toBe(true);
  expect(hook.result.current.pending).toBe(false);
  expect(hook.result.current.updatesAvailable).toBe(false);
  hook.rerender({ updatesAvailable: true });
  await act(flush);
  expect(verificationRequests()).toHaveLength(1);
  hook.unmount();
  app.client.clear();
});

test("provider observations reject cross-configuration replies and unsafe semantic values", () => {
  const request = {
    operation: "observe",
    target_id: a,
    refresh: false,
  } as const;
  expect(() => decodeProviderReply(request, observation(b))).toThrow();
  expect(() =>
    decodeProviderReply(request, {
      ...observation(),
      observation: {
        ...observation().observation,
        revision: Number.MAX_SAFE_INTEGER + 1,
      },
    }),
  ).toThrow();
  expect(decodeProviderReply(request, observation())).toEqual(observation());
});

test("provider freshness is independent per target and late old-incarnation results stay isolated", async () => {
  const app = new AppObservations();
  app.connect("incarnation-a");
  app.freshness.accept("incarnation-a");
  const completions: (() => void)[] = [];
  vi.mocked(invoke).mockImplementation(
    (_command, args) =>
      new Promise((resolve) => {
        const expected =
          args && "expectedIncarnation" in args
            ? args.expectedIncarnation
            : undefined;
        const request = args && "request" in args ? args.request : undefined;
        if (
          !request ||
          typeof request !== "object" ||
          !("target_id" in request)
        )
          throw new Error("wrong request");
        completions.push(() =>
          resolve({
            incarnation: expected,
            result: { Ok: observation(String(request.target_id)) },
          }),
        );
      }),
  );
  const releaseA = app.providers.observe("incarnation-a", {
    operation: "observe",
    target_id: a,
  });
  const releaseB = app.providers.observe("incarnation-a", {
    operation: "observe",
    target_id: b,
  });
  await flush();
  expect(invoke).toHaveBeenCalledTimes(2);
  completions[0]();
  completions[1]();
  await flush();
  app.providers.invalidate({ kind: "provider", target_id: a });
  app.freshness.accept("incarnation-a");
  await flush();
  expect(invoke).toHaveBeenCalledTimes(3);
  app.connect("incarnation-b");
  app.freshness.accept("incarnation-b");
  completions[2]();
  await flush();
  expect(
    app.providers.peek("incarnation-b", { operation: "observe", target_id: a })
      ?.data,
  ).toBeUndefined();
  expect(
    app.providers.peek("incarnation-a", { operation: "observe", target_id: b })
      ?.data?.value,
  ).toEqual(observation(b));
  releaseA();
  releaseB();
  app.client.clear();
});
