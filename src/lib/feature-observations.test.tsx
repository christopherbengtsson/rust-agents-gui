import { expect, test, vi } from "vitest";
import { RepositoryObservations } from "./repository-observations";
import { createObservationQueryClient } from "./feature-query-cache";
import { FeatureFreshness } from "./feature-freshness";
import { ObservationCoordinator } from "./observation-coordinator";
import { object } from "./observation-contracts";
import { invoke } from "@tauri-apps/api/core";
import { ownerBound } from "../test-support/owner-bound-invoke";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const flush = async () => {
  for (let i = 0; i < 25; i++) await Promise.resolve();
};
test("baseline precedes repository reads, and hints during initial and follow-up reads stay pending", async () => {
  const freshness = new FeatureFreshness();
  const feature = new RepositoryObservations(
    createObservationQueryClient(),
    new ObservationCoordinator(),
    freshness,
  );
  freshness.connect("owner");
  const completions: (() => void)[] = [];
  vi.mocked(invoke).mockImplementation(
    ownerBound(
      () =>
        new Promise((resolve) => {
          completions.push(() =>
            resolve({
              kind: "repositories",
              page: { repositories: [], next: null },
            }),
          );
        }),
    ),
  );
  const unlisten = feature.subscribe(() => {});
  const release = feature.observe("owner", { operation: "list", after: null });
  await flush();
  expect(invoke).not.toHaveBeenCalled();
  feature.invalidate({ kind: "repositories" });
  freshness.accept("owner");
  await flush();
  expect(invoke).toHaveBeenCalledTimes(1);
  feature.invalidate({ kind: "repositories" });
  freshness.accept("owner");
  completions[0]();
  await flush();
  expect(invoke).toHaveBeenCalledTimes(2);
  feature.invalidate({ kind: "repositories" });
  freshness.accept("owner");
  completions[1]();
  await flush();
  expect(invoke).toHaveBeenCalledTimes(3);
  completions[2]();
  await flush();
  expect(invoke).toHaveBeenCalledTimes(3);
  release();
  unlisten();
});

import { afterEach } from "vitest";
import { AppObservations } from "./app-observations";
import { ObservationFeed, type FeedTransport } from "./observation-feed";
import type { Repository, RepositoryRequest } from "./repositories";
const repository: Repository = {
  id: "00000000-0000-4000-8000-000000000001",
  name: "one",
  path: "/one",
  availability: { state: "available" },
};
const listRequest = { operation: "list", after: null } as const;
function fixture() {
  const app = new AppObservations();
  app.connect("owner");
  let sequence = 0;
  const send = (targets?: import("./observation-feed").DirtyTarget[]) => {
    sequence++;
    app.accept({
      subscription_id: "fixture",
      batch_id: sequence,
      cursor: { owner: "owner", sequence },
      sample: targets
        ? { kind: "hints", watermark: sequence, targets }
        : { kind: "resync_required", watermark: sequence },
    });
  };
  return { app, feature: app.repositories, send };
}
afterEach(() => {
  vi.resetAllMocks();
  vi.useRealTimers();
  localStorage.clear();
});

test("Query cancellation and remount retain physical identity and the original captured generation", async () => {
  const { app, feature, send } = fixture();
  const completions: (() => void)[] = [];
  vi.mocked(invoke).mockImplementation(
    ownerBound(
      () =>
        new Promise((resolve) => {
          completions.push(() =>
            resolve({
              kind: "repositories",
              page: { repositories: [repository], next: null },
            }),
          );
        }),
    ),
  );
  send();
  const release = feature.observe("owner", listRequest);
  await flush();
  expect(invoke).toHaveBeenCalledTimes(1);
  await app.client.cancelQueries();
  release();
  send([{ kind: "repositories" }]);
  const remount = feature.observe("owner", listRequest);
  await flush();
  expect(invoke).toHaveBeenCalledTimes(1);
  completions[0]();
  await flush();
  expect(invoke).toHaveBeenCalledTimes(2);
  completions[1]();
  await flush();
  expect(feature.peek("owner", listRequest)?.data?.value).toMatchObject({
    kind: "repositories",
    page: { repositories: [repository] },
  });
  remount();
  app.client.clear();
});

test("hidden details do not fan out; overflow and resync remain owed separately by hidden targets", async () => {
  const { app, feature, send } = fixture();
  const other = { ...repository, id: "00000000-0000-4000-8000-000000000002" };
  const requests: RepositoryRequest[] = [];
  vi.mocked(invoke).mockImplementation(
    ownerBound((_, args) => {
      const request = object(args) ? args.request : undefined;
      if (!request || typeof request !== "object" || !("operation" in request))
        throw new Error("fixture request");
      if (request.operation === "get" && "id" in request) {
        requests.push({
          operation: "get",
          id: String(request.id),
          refresh: false,
        });
        return {
          kind: "repository",
          repository: request.id === other.id ? other : repository,
        };
      }
      return { kind: "repositories", page: { repositories: [], next: null } };
    }),
  );
  send();
  const a = { operation: "get", id: repository.id } as const;
  const b = { operation: "get", id: other.id } as const;
  const hideA = feature.observe("owner", a);
  const hideB = feature.observe("owner", b);
  await flush();
  hideA();
  hideB();
  expect(requests).toHaveLength(2);
  for (let index = 0; index < 257; index++)
    send([
      {
        kind: "inspection",
        workspace_id: `00000000-0000-4000-8000-${index.toString(16).padStart(12, "0")}`,
      },
    ]);
  expect(app.freshness["identities"].size).toBeLessThanOrEqual(256);
  await flush();
  expect(requests).toHaveLength(2);
  const showA = feature.observe("owner", a);
  await flush();
  expect(requests).toHaveLength(3);
  // Satisfying A cannot clear B's sampled global obligation.
  const showB = feature.observe("owner", b);
  await flush();
  expect(requests).toHaveLength(4);
  send();
  await flush();
  expect(requests).toHaveLength(6);
  showA();
  showB();
  app.client.clear();
});

test("errors retain last-good data without retries and a new hint permits recovery", async () => {
  const { app, feature, send } = fixture();
  let fail = false;
  vi.mocked(invoke).mockImplementation(
    ownerBound(() => {
      if (fail) throw new Error("controlled lost reply");
      return {
        kind: "repositories",
        page: { repositories: [repository], next: null },
      };
    }),
  );
  send();
  const release = feature.observe("owner", listRequest);
  await flush();
  const good = feature.peek("owner", listRequest)?.data;
  fail = true;
  send([{ kind: "repositories" }]);
  await flush();
  expect(feature.peek("owner", listRequest)?.status).toBe("error");
  expect(feature.peek("owner", listRequest)?.data).toBe(good);
  await flush();
  expect(invoke).toHaveBeenCalledTimes(2);
  fail = false;
  send([{ kind: "repositories" }]);
  await flush();
  expect(invoke).toHaveBeenCalledTimes(3);
  expect(feature.peek("owner", listRequest)?.status).toBe("success");
  release();
  app.client.clear();
});

test("feed records feature dirtiness before ack and starts reads only after synchronous acceptance", async () => {
  const { app, feature } = fixture();
  vi.mocked(invoke).mockImplementation(
    ownerBound(() => ({
      kind: "repositories",
      page: { repositories: [], next: null },
    })),
  );
  let deliver: ((value: unknown) => void) | undefined;
  let subscription = "";
  const transport: FeedTransport = {
    subscribe: vi.fn(async (_, id, __, receive) => {
      subscription = id;
      deliver = receive;
    }),
    acknowledge: vi.fn(async () => {
      expect(app.freshness.ready("owner")).toBe(true);
      expect(app.freshness.version(["repositories"])).toBeGreaterThan(0);
      expect(invoke).not.toHaveBeenCalled();
    }),
    stop: vi.fn(async () => {}),
  };
  const feed = new ObservationFeed(transport, app);
  const release = feature.observe("owner", listRequest);
  feed.connect("owner");
  await flush();
  deliver?.({
    kind: "batch",
    batch: {
      subscription_id: subscription,
      batch_id: 1,
      cursor: { owner: "owner", sequence: 2 },
      sample: {
        kind: "hints",
        watermark: 2,
        targets: [{ kind: "repositories" }],
      },
    },
  });
  expect(transport.acknowledge).toHaveBeenCalledOnce();
  await flush();
  expect(invoke).toHaveBeenCalledOnce();
  release();
  feed.connect(undefined);
  await flush();
  app.client.clear();
});

test("hint-driven reads observe availability; explicit reprobe has a distinct physical flag and no Checking loop", async () => {
  vi.useFakeTimers();
  const { app, feature, send } = fixture();
  let fault = false;
  const refreshes: boolean[] = [];
  vi.mocked(invoke).mockImplementation(
    ownerBound((_, args) => {
      const request = object(args) ? args.request : undefined;
      const refresh =
        typeof request === "object" &&
        request !== null &&
        "refresh" in request &&
        request.refresh === true;
      refreshes.push(refresh);
      return {
        kind: "repositories",
        page: {
          repositories: [
            {
              ...repository,
              availability: refresh
                ? { state: "checking" }
                : fault
                  ? {
                      state: "unavailable",
                      error: { code: "git", message: "availability fault" },
                    }
                  : { state: "available" },
            },
          ],
          next: null,
        },
      };
    }),
  );
  send();
  const release = feature.observe("owner", listRequest);
  await flush();
  await feature.refresh("owner", true);
  await flush();
  expect(refreshes).toEqual([false, true]);
  fault = true;
  send([{ kind: "repositories" }]);
  await flush();
  expect(refreshes).toEqual([false, true, false]);
  await vi.advanceTimersByTimeAsync(1000);
  expect(refreshes).toHaveLength(3);
  expect(feature.peek("owner", listRequest)?.data?.value).toMatchObject({
    kind: "repositories",
    page: { repositories: [{ availability: { state: "unavailable" } }] },
  });
  release();
  app.client.clear();
});

import { onlineManager, focusManager } from "@tanstack/react-query";
test("local observations ignore browser offline/focus events and inactive Query data expires after five minutes", async () => {
  vi.useFakeTimers();
  const { app, feature, send } = fixture();
  app.client.mount();
  onlineManager.setOnline(false);
  vi.mocked(invoke).mockImplementation(
    ownerBound(() => ({
      kind: "repositories",
      page: { repositories: [], next: null },
    })),
  );
  send();
  const release = feature.observe("owner", listRequest);
  await flush();
  expect(invoke).toHaveBeenCalledOnce();
  focusManager.setFocused(false);
  focusManager.setFocused(true);
  onlineManager.setOnline(true);
  await flush();
  expect(invoke).toHaveBeenCalledOnce();
  release();
  await flush();
  await vi.advanceTimersByTimeAsync(299_999);
  expect(app.client.getQueryCache().getAll()).toHaveLength(1);
  await vi.advanceTimersByTimeAsync(1);
  expect(app.client.getQueryCache().getAll()).toHaveLength(0);
  app.client.unmount();
  focusManager.setFocused(undefined);
  app.client.clear();
});

test("a failed detail stays paused across hide/remount until a new hint or explicit refresh", async () => {
  const { app, feature, send } = fixture();
  const target = { operation: "get", id: repository.id } as const;
  vi.mocked(invoke).mockImplementation(
    ownerBound(() => {
      throw new Error("detail unavailable");
    }),
  );
  send();
  const hide = feature.observe("owner", target);
  await flush();
  expect(invoke).toHaveBeenCalledOnce();
  hide();
  const hideAgain = feature.observe("owner", target);
  await flush();
  expect(invoke).toHaveBeenCalledOnce();
  send([{ kind: "repository", repository_id: repository.id }]);
  await flush();
  expect(invoke).toHaveBeenCalledTimes(2);
  await feature.refresh("owner");
  await flush();
  expect(invoke).toHaveBeenCalledTimes(3);
  hideAgain();
  app.client.clear();
});

test("a hint while queued is captured by the eventual failed physical read and cannot trigger an extra retry", async () => {
  const { app, feature, send } = fixture();
  const scope = app.reads.scope();
  let releaseFirst: (() => void) | undefined;
  let releaseSecond: (() => void) | undefined;
  const first = scope.read(
    ["held-one"],
    () =>
      new Promise<void>((resolve) => {
        releaseFirst = resolve;
      }),
  );
  const second = scope.read(
    ["held-two"],
    () =>
      new Promise<void>((resolve) => {
        releaseSecond = resolve;
      }),
  );
  vi.mocked(invoke).mockImplementation(
    ownerBound(() => {
      throw new Error("controlled detail fault");
    }),
  );
  send();
  const hide = feature.observe("owner", {
    operation: "get",
    id: repository.id,
  });
  await flush();
  expect(invoke).not.toHaveBeenCalled();
  send([{ kind: "repository", repository_id: repository.id }]);
  releaseFirst?.();
  await first;
  await flush();
  expect(invoke).toHaveBeenCalledOnce();
  await flush();
  expect(invoke).toHaveBeenCalledOnce();
  releaseSecond?.();
  await second;
  hide();
  scope.dispose();
  app.client.clear();
});
