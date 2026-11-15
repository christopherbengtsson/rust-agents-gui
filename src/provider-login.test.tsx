import { afterEach, expect, test, vi } from "vitest";
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
import { AppObservations } from "./lib/app-observations";
import { decodeProviderReply } from "./lib/providers";
import { object } from "./lib/observation-contracts";
import { validateProviderRequest } from "./generated/contracts/validators";
import type {
  ProviderObservation,
  ProviderRequest,
} from "./generated/contracts/types";
import { ownerBound } from "./test-support/owner-bound-invoke";
import { useProviderProgress } from "./hooks/use-provider-progress";
import { ProviderSignIn } from "./components/blocks/provider-sign-in";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});
const id = "00000000-0000-4000-8000-000000000001";
const attempt = "00000000-0000-4000-8000-000000000002";
const initial: ProviderObservation = {
  target: {
    id,
    provider: "codex",
    executable: "/fixture/codex",
    codex_home: "/fixture/home",
    working_directory: "/fixture/neutral",
    supported_version: "0.169.0",
  },
  revision: 2,
  phase: "observed",
  evidence: null,
  problem: "authentication_unavailable",
  diagnostic: null,
  login: null,
};
const pending: ProviderObservation = {
  ...initial,
  revision: 3,
  login: {
    id: attempt,
    phase: "pending",
    outcome: null,
    browser_open_failed: false,
    diagnostic: null,
  },
};

test("lost start replies reconcile once and app progress retains interest after the settings view releases it", async () => {
  const app = new AppObservations();
  app.connect("owner");
  app.freshness.accept("owner");
  let current = initial;
  const requests: ProviderRequest[] = [];
  vi.mocked(invoke).mockImplementation(
    ownerBound(async (_command, args) => {
      if (!object(args) || !validateProviderRequest(args.request))
        throw new Error("Unexpected request");
      const request = args.request;
      requests.push(request);
      if (request.operation === "target")
        return { kind: "target", target: initial.target };
      if (request.operation === "start_browser_login") {
        current = {
          ...pending,
          login: {
            ...pending.login,
            id: request.attempt_id,
            phase: "pending",
            outcome: null,
            browser_open_failed: false,
            diagnostic: null,
          },
        };
        throw new Error("Controlled lost reply");
      }
      return { kind: "observation", observation: current };
    }),
  );
  const releaseTarget = app.providers.observe("owner", { operation: "target" });
  const releaseView = app.providers.observe("owner", {
    operation: "observe",
    target_id: id,
  });
  const root = renderHook(() => useProviderProgress(app, "owner"));
  await waitFor(() =>
    expect(root.result.current.observation?.revision).toBe(2),
  );
  await act(async () => {
    await Promise.all([
      app.providers.startBrowser("owner", initial),
      app.providers.startBrowser("owner", initial),
    ]);
  });
  await waitFor(() => expect(root.result.current.active).toBe(true));
  expect(
    requests.filter((request) => request.operation === "start_browser_login"),
  ).toHaveLength(1);
  expect(
    requests.filter((request) => request.operation === "reconcile_login"),
  ).toHaveLength(1);
  releaseView();
  releaseTarget();
  current = {
    ...current,
    revision: 4,
    login: {
      id: current.login?.id ?? attempt,
      phase: "completed",
      outcome: "completed",
      browser_open_failed: false,
      diagnostic: null,
    },
  };
  await act(async () => {
    app.providers.invalidate({ kind: "provider", target_id: id });
    app.freshness.accept("owner");
  });
  await waitFor(() =>
    expect(root.result.current.observation?.login?.phase).toBe("completed"),
  );
  expect(root.result.current.active).toBe(false);
  root.unmount();
});

test("unknown start delivery stays fenced until explicit reconciliation and reconnect never repeats start", async () => {
  const app = new AppObservations();
  app.connect("owner");
  app.freshness.accept("owner");
  let canRead = false;
  const requests: ProviderRequest[] = [];
  vi.mocked(invoke).mockImplementation(
    ownerBound(async (_command, args) => {
      if (!object(args) || !validateProviderRequest(args.request))
        throw new Error("Unexpected request");
      requests.push(args.request);
      if (!canRead || args.request.operation === "start_browser_login")
        throw new Error("Controlled outage");
      return { kind: "observation", observation: pending };
    }),
  );
  await app.providers.startBrowser("owner", initial);
  expect(app.providers.action("owner", id)?.state).toBe("uncertain");
  await app.providers.startBrowser("owner", initial);
  expect(
    requests.filter((request) => request.operation === "start_browser_login"),
  ).toHaveLength(1);
  app.connect(undefined);
  canRead = true;
  app.connect("owner");
  app.freshness.accept("owner");
  await waitFor(() =>
    expect(app.providers.action("owner", id)).toBeUndefined(),
  );
  expect(
    requests.filter((request) => request.operation === "start_browser_login"),
  ).toHaveLength(1);
});

test("login replies reject another attempt or target and keep provider URLs out of the generated observation", () => {
  const request: ProviderRequest = {
    operation: "cancel_login",
    target_id: id,
    attempt_id: attempt,
  };
  expect(() =>
    decodeProviderReply(request, {
      kind: "observation",
      observation: { ...pending, login: { ...pending.login, id } },
    }),
  ).toThrow();
  expect(() =>
    decodeProviderReply(request, {
      kind: "observation",
      observation: { ...pending, target: { ...initial.target, id: attempt } },
    }),
  ).toThrow();
  expect(() =>
    decodeProviderReply(request, { kind: "observation", observation: pending }),
  ).not.toThrow();
});

test("mount and explained ChatGPT presentation have no login effect; reopen and cancel target the existing attempt", async () => {
  const user = userEvent.setup();
  const callbacks = {
    pending: false,
    uncertain: false,
    message: undefined,
    start: vi.fn(),
    cancel: vi.fn(),
    reopen: vi.fn(),
    reconcile: vi.fn(),
  };
  const view = render(
    <ProviderSignIn observation={initial} available login={callbacks} />,
  );
  expect(screen.getByText("Sign in through your browser.")).toBeTruthy();
  expect(callbacks.start).not.toHaveBeenCalled();
  await user.click(
    screen.getByRole("button", { name: "Sign in through browser" }),
  );
  expect(callbacks.start).toHaveBeenCalledTimes(1);
  view.rerender(
    <ProviderSignIn observation={pending} available login={callbacks} />,
  );
  await user.click(screen.getByRole("button", { name: "Reopen browser" }));
  await user.click(screen.getByRole("button", { name: "Cancel sign-in" }));
  expect(callbacks.reopen).toHaveBeenCalledTimes(1);
  expect(callbacks.cancel).toHaveBeenCalledTimes(1);
  expect(callbacks.start).toHaveBeenCalledTimes(1);
  view.rerender(
    <ProviderSignIn
      observation={{
        ...pending,
        login: {
          id: attempt,
          phase: "cancelled",
          outcome: "cancelled",
          browser_open_failed: false,
          diagnostic: null,
        },
      }}
      available
      login={callbacks}
    />,
  );
  expect(document.activeElement).toBe(
    screen.getByText("Browser sign-in cancelled"),
  );
});
