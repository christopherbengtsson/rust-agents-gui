import { expect, test } from "vitest";
import { decodeFeedBatch } from "./lib/observation-feed";

import { afterEach, vi } from "vitest";
import { ObservationFeed, type FeedTransport } from "./lib/observation-feed";
const flush = async () => {
  for (let i = 0; i < 12; i++) await Promise.resolve();
};
afterEach(() => vi.useRealTimers());
function scenario() {
  const deliveries: {
    owner: string;
    id: string;
    after: unknown;
    receive: (value: unknown) => void;
  }[] = [];
  const transport: FeedTransport = {
    subscribe: vi.fn(async (owner, id, after, receive) => {
      deliveries.push({ owner, id, after, receive });
    }),
    acknowledge: vi.fn(async () => {}),
    stop: vi.fn(async () => {}),
  };
  const acceptance = { connect: vi.fn(), accept: vi.fn() };
  const feed = new ObservationFeed(transport, acceptance);
  const send = (
    index: number,
    batch: number,
    seq: number,
    kind = "resync_required",
  ) => {
    const session = deliveries[index];
    session.receive({
      kind: "batch",
      batch: {
        subscription_id: session.id,
        batch_id: batch,
        cursor: { owner: session.owner, sequence: seq },
        sample: {
          kind,
          watermark: seq,
          ...(kind === "hints" ? { targets: [{ kind: "repositories" }] } : {}),
        },
      },
    });
  };
  return { feed, transport, deliveries, send, acceptance };
}
test("acceptance precedes ack, and reconnect uses the feed cursor rather than status", async () => {
  const s = scenario();
  s.feed.connect("owner-a");
  await flush();
  vi.mocked(s.transport.acknowledge).mockImplementation(async (ack) => {
    expect(s.feed.getSnapshot().accepted).toEqual(ack.cursor);
    expect(s.acceptance.accept).toHaveBeenCalledWith(
      expect.objectContaining({ cursor: ack.cursor }),
    );
  });
  s.send(0, 1, 10);
  await flush();
  s.feed.connect(undefined);
  s.feed.connect("owner-a");
  await flush();
  expect(s.deliveries[1].after).toEqual({ owner: "owner-a", sequence: 10 });
  s.send(0, 2, 999);
  expect(s.feed.getSnapshot().accepted?.sequence).toBe(10);
  s.feed.connect("owner-b");
  await flush();
  expect(s.deliveries[2].after).toBeUndefined();
  s.feed.connect(undefined);
  await flush();
});
test("a next batch may arrive before the preceding IPC acknowledgement promise settles", async () => {
  const s = scenario();
  s.feed.connect("owner-a");
  await flush();
  let finish!: () => void;
  vi.mocked(s.transport.acknowledge).mockImplementationOnce(
    () =>
      new Promise<void>((resolve) => {
        finish = resolve;
      }),
  );
  s.send(0, 1, 1);
  s.send(0, 2, 2, "hints");
  expect(s.feed.getSnapshot().state).toBe("live");
  expect(s.feed.getSnapshot().accepted?.sequence).toBe(2);
  finish();
  s.feed.connect(undefined);
  await flush();
});
test("stalls and malformed deliveries stop visibly without advancing the accepted cursor", async () => {
  vi.useFakeTimers();
  const s = scenario();
  s.feed.connect("owner-a");
  await flush();
  s.send(0, 1, 7);
  await flush();
  await vi.advanceTimersByTimeAsync(15_000);
  expect(s.feed.getSnapshot().state).toBe("stopped");
  expect(s.feed.getSnapshot().message).toMatch(/stalled/);
  expect(s.transport.stop).toHaveBeenCalledWith(s.deliveries[0].id);
  s.send(0, 2, 8);
  expect(s.feed.getSnapshot().accepted?.sequence).toBe(7);
  s.feed.connect("owner-a");
  await flush();
  s.deliveries[1].receive({ kind: "batch", batch: {} });
  expect(s.feed.getSnapshot().state).toBe("stopping");
  await flush();
  expect(s.feed.getSnapshot().state).toBe("stopped");
  expect(s.feed.getSnapshot().accepted?.sequence).toBe(7);
  s.feed.connect(undefined);
  await flush();
});
test("late setup is stopped and joined before the replacement is subscribed", async () => {
  const s = scenario();
  let finish!: () => void;
  vi.mocked(s.transport.subscribe).mockImplementationOnce(
    async (owner, id, after, receive) => {
      s.deliveries.push({ owner, id, after, receive });
      await new Promise<void>((resolve) => {
        finish = resolve;
      });
    },
  );
  s.feed.connect("owner-a");
  await flush();
  s.feed.connect("owner-b");
  await flush();
  expect(s.transport.stop).toHaveBeenCalledWith(s.deliveries[0].id);
  expect(s.deliveries).toHaveLength(1);
  s.send(0, 1, 9);
  expect(s.feed.getSnapshot().accepted).toBeUndefined();
  finish();
  await flush();
  expect(s.deliveries).toHaveLength(2);
  expect(s.deliveries[1].owner).toBe("owner-b");
  s.feed.connect(undefined);
  await flush();
});
import corpus from "../contracts/observations-v13.json";
test("the shared protocol corpus passes production feed decoding", () => {
  let previous = 0;
  for (const value of corpus.feed) {
    const batch = decodeFeedBatch(value, "owner-a", "feed-1", previous);
    expect(batch).toEqual(value);
    previous = batch.batch_id;
  }
});
test.each([
  { subscription_id: "other" },
  { batch_id: 2 },
  { cursor: { owner: "replacement", sequence: 2 } },
  { cursor: { owner: "owner-a", sequence: Number.MAX_SAFE_INTEGER + 1 } },
  {
    sample: {
      kind: "hints",
      watermark: 2,
      targets: [{ kind: "workspace", workspace_id: "path" }],
    },
  },
  {
    sample: {
      kind: "hints",
      watermark: 2,
      targets: Array.from({ length: 33 }, () => ({ kind: "repositories" })),
    },
  },
  { sample: { kind: "heartbeat", watermark: 2 } },
  { extra: "x".repeat(4096) },
])("malformed batch cannot advance acceptance: %j", (override) => {
  expect(() =>
    decodeFeedBatch(
      {
        subscription_id: "feed-1",
        batch_id: 1,
        cursor: { owner: "owner-a", sequence: 2 },
        sample: {
          kind: "hints",
          watermark: 2,
          targets: [{ kind: "repositories" }],
        },
        ...override,
      },
      "owner-a",
      "feed-1",
      0,
    ),
  ).toThrow();
});

test("a failed stop attempt does not permanently block later explicit reconnect", async () => {
  const s = scenario();
  s.feed.connect("owner-a");
  await flush();
  vi.mocked(s.transport.stop).mockRejectedValueOnce(
    new Error("Controlled lost stop reply"),
  );
  s.feed.connect("owner-b");
  await flush();
  s.feed.connect("owner-b");
  await flush();
  expect(s.deliveries.at(-1)?.owner).toBe("owner-b");
  s.feed.connect(undefined);
  await flush();
});

test("failed registration requests cleanup of its possibly attached native subscription", async () => {
  const s = scenario();
  vi.mocked(s.transport.subscribe).mockRejectedValueOnce(
    new Error("Lost registration reply"),
  );
  s.feed.connect("owner-a");
  await flush();
  expect(s.feed.getSnapshot().state).toBe("stopped");
  expect(s.transport.stop).toHaveBeenCalledTimes(2);
  expect(s.transport.stop).toHaveBeenLastCalledWith(expect.any(String));
  s.feed.connect(undefined);
  await flush();
});

test("cleanup failure stays visible and fenced until explicit native reconciliation", async () => {
  const s = scenario();
  s.feed.connect("owner-a");
  await flush();
  vi.mocked(s.transport.stop).mockRejectedValue(
    new Error("Native cleanup unconfirmed"),
  );
  s.feed.connect("owner-b");
  await flush();
  expect(s.feed.getSnapshot().state).toBe("stopped");
  expect(s.feed.getSnapshot().message).toMatch(/cleanup unconfirmed/);
  expect(s.deliveries).toHaveLength(1);
  vi.mocked(s.transport.stop).mockResolvedValue(undefined);
  s.feed.connect("owner-c");
  await flush();
  expect(s.deliveries.map((d) => d.owner)).toEqual(["owner-a", "owner-c"]);
  s.feed.connect(undefined);
  await flush();
});

test("configuration changes during stopping retain only the latest desired owner", async () => {
  const s = scenario();
  s.feed.connect("owner-a");
  await flush();
  let join: () => void = () => {
    throw new Error("Cleanup not started");
  };
  vi.mocked(s.transport.stop).mockImplementationOnce(
    () =>
      new Promise<void>((resolve) => {
        join = resolve;
      }),
  );
  s.feed.connect("owner-b");
  await flush();
  expect(s.feed.getSnapshot().state).toBe("stopping");
  s.feed.connect("owner-c");
  s.feed.connect("owner-d");
  await flush();
  expect(s.deliveries).toHaveLength(1);
  join();
  await flush();
  expect(s.deliveries.map((d) => d.owner)).toEqual(["owner-a", "owner-d"]);
  s.feed.connect(undefined);
  await flush();
});

test("owner changes preserve the delivery failure while exact native cleanup is pending", async () => {
  const s = scenario();
  s.feed.connect("owner-a");
  await flush();
  let join: () => void = () => {
    throw new Error("Cleanup not started");
  };
  vi.mocked(s.transport.stop).mockImplementationOnce(
    () =>
      new Promise<void>((resolve) => {
        join = resolve;
      }),
  );
  s.deliveries[0].receive({
    kind: "stopped",
    subscription_id: s.deliveries[0].id,
    message: "Delivery failed; native cleanup pending",
  });
  s.feed.connect("owner-b");
  expect(s.feed.getSnapshot().state).toBe("stopping");
  expect(s.feed.getSnapshot().message).toContain("Delivery failed");
  expect(s.deliveries).toHaveLength(1);
  join();
  await flush();
  expect(s.deliveries.at(-1)?.owner).toBe("owner-b");
  expect(s.feed.getSnapshot().message).toBeUndefined();
  s.feed.connect(undefined);
  await flush();
});
