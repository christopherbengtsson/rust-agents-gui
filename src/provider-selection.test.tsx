import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useProviderSelection } from "./hooks/use-provider-selection";
import { AppObservations } from "./lib/app-observations";
import { incarnationBound } from "./test-support/incarnation-bound-invoke";
import { object } from "./lib/observation-contracts";
import { validateProviderRequest } from "./generated/contracts/validators";
import type {
  ProviderObservation,
  ProviderRequest,
} from "./generated/contracts/types";
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});
const observation: ProviderObservation = {
  target: {
    id: "00000000-0000-4000-8000-000000000001",
    provider: "codex",
    executable: null,
    selected_executable: null,
    discovery_source: null,
    codex_home: "/home",
    working_directory: "/neutral",
    minimum_supported_version: "0.169.0",
  },
  revision: 2,
  phase: "observed",
  evidence: null,
  problem: "missing_executable",
  diagnostic: null,
  login: null,
  selection: null,
};
function app() {
  const app = new AppObservations();
  app.connect("incarnation");
  app.freshness.accept("incarnation");
  return app;
}

test("chooser cancellation leaves the runtime and saved selection untouched", async () => {
  vi.mocked(open).mockResolvedValue(null);
  const { result } = renderHook(() =>
    useProviderSelection(app(), "incarnation", observation),
  );
  await act(() => result.current.locate());
  expect(invoke).not.toHaveBeenCalled();
  expect(open).toHaveBeenCalledExactlyOnceWith({
    directory: false,
    multiple: false,
    title: "Locate Codex executable",
  });
});

test("a chooser result arriving after navigation has no selection effect", async () => {
  let resolve: (path: string) => void = () => {};
  vi.mocked(open).mockReturnValue(
    new Promise<string>((done) => {
      resolve = done;
    }),
  );
  const feature = app();
  const { result, unmount } = renderHook(() =>
    useProviderSelection(feature, "incarnation", observation),
  );
  let pending: Promise<void> | undefined;
  act(() => {
    pending = result.current.locate();
  });
  unmount();
  await act(async () => {
    resolve("/chosen/codex");
    await pending;
  });
  expect(invoke).not.toHaveBeenCalled();
});

test("lost selection reply reconciles the same action and never repeats validation intent", async () => {
  const feature = app();
  const requests: ProviderRequest[] = [];
  let current = observation;
  vi.mocked(invoke).mockImplementation(
    incarnationBound(async (_command, args) => {
      if (!object(args) || !validateProviderRequest(args.request))
        throw new Error("Unexpected request");
      const request = args.request;
      requests.push(request);
      if (request.operation === "select_executable") {
        current = {
          ...current,
          revision: 3,
          phase: "checking",
          selection: {
            id: request.action_id,
            path: request.path,
            phase: "checking",
          },
        };
        throw new Error("Controlled lost reply");
      }
      return { kind: "observation", observation: current };
    }),
  );
  await feature.providers.selectExecutable(
    "incarnation",
    observation,
    "/chosen/codex",
  );
  await waitFor(() =>
    expect(
      feature.providers.action("incarnation", observation.target.id),
    ).toBeUndefined(),
  );
  expect(
    requests.filter((request) => request.operation === "select_executable"),
  ).toHaveLength(1);
  const selection = requests.find(
    (request) => request.operation === "select_executable",
  );
  expect(selection).toMatchObject({
    target_id: observation.target.id,
    expected_revision: 2,
    path: "/chosen/codex",
  });
  expect(
    requests.find((request) => request.operation === "reconcile_selection"),
  ).toMatchObject({ action_id: current.selection?.id });
  expect(
    requests.some((request) => request.operation === "start_browser_login"),
  ).toBe(false);
});

test.each(["replacement", undefined])(
  "incarnation detachment/replacement during a chooser does not fence later locating: %s",
  async (replacement) => {
    let resolve: (path: string) => void = () => {};
    vi.mocked(open).mockReturnValue(
      new Promise<string>((done) => {
        resolve = done;
      }),
    );
    const feature = app();
    const initialProps: { incarnation: string | undefined } = {
      incarnation: "incarnation",
    };
    const { result, rerender } = renderHook(
      ({ incarnation }: { incarnation: string | undefined }) =>
        useProviderSelection(feature, incarnation, observation),
      { initialProps },
    );
    let pending: Promise<void> | undefined;
    act(() => {
      pending = result.current.locate();
    });
    expect(result.current.picking).toBe(true);
    rerender({ incarnation: replacement });
    if (!replacement) rerender({ incarnation: "incarnation" });
    await act(async () => {
      resolve("/chosen/codex");
      await pending;
    });
    expect(invoke).not.toHaveBeenCalled();
    expect(result.current.picking).toBe(false);
  },
);
