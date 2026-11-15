import { afterEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { decodeProviderReply } from "./lib/providers";
import { AppObservations } from "./lib/app-observations";
import type { ProviderTarget } from "./generated/contracts/types";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const a = "00000000-0000-4000-8000-000000000001";
const b = "00000000-0000-4000-8000-000000000002";
const target: ProviderTarget = {
  id: a,
  provider: "codex",
  executable: "/fixture/codex",
  codex_home: "/fixture/home",
  working_directory: "/fixture/neutral",
  supported_version: "0.169.0",
};
const observation = (id = a) => ({
  kind: "observation",
  observation: {
    target: { ...target, id },
    revision: 2,
    phase: "observed",
    evidence: null,
    problem: "missing_executable",
  },
});
const flush = async () => {
  for (let i = 0; i < 35; i++) await Promise.resolve();
};
afterEach(() => {
  vi.resetAllMocks();
  localStorage.clear();
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

test("provider freshness is independent per target and late old-owner results stay isolated", async () => {
  const app = new AppObservations();
  app.connect("owner-a");
  app.freshness.accept("owner-a");
  const completions: (() => void)[] = [];
  vi.mocked(invoke).mockImplementation(
    (_command, args) =>
      new Promise((resolve) => {
        const expected =
          args && "expectedOwner" in args ? args.expectedOwner : undefined;
        const request = args && "request" in args ? args.request : undefined;
        if (
          !request ||
          typeof request !== "object" ||
          !("target_id" in request)
        )
          throw new Error("wrong request");
        completions.push(() =>
          resolve({
            owner: expected,
            result: { Ok: observation(String(request.target_id)) },
          }),
        );
      }),
  );
  const releaseA = app.providers.observe("owner-a", {
    operation: "observe",
    target_id: a,
  });
  const releaseB = app.providers.observe("owner-a", {
    operation: "observe",
    target_id: b,
  });
  await flush();
  expect(invoke).toHaveBeenCalledTimes(2);
  completions[0]();
  completions[1]();
  await flush();
  app.providers.invalidate({ kind: "provider", target_id: a });
  app.freshness.accept("owner-a");
  await flush();
  expect(invoke).toHaveBeenCalledTimes(3);
  app.connect("owner-b");
  app.freshness.accept("owner-b");
  completions[2]();
  await flush();
  expect(
    app.providers.peek("owner-b", { operation: "observe", target_id: a })?.data,
  ).toBeUndefined();
  expect(
    app.providers.peek("owner-a", { operation: "observe", target_id: b })?.data
      ?.value,
  ).toEqual(observation(b));
  releaseA();
  releaseB();
  app.client.clear();
});
