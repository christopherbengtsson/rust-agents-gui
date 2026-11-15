import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, test, vi } from "vitest";
import { useObservationFeed } from "./hooks/use-observation-feed";
import corpus from "../contracts/observations-v11.json";
import { object } from "./lib/observation-contracts";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  Channel: class {
    onmessage: (value: unknown) => void;
    constructor(receive: (value: unknown) => void) {
      this.onmessage = receive;
    }
  },
}));
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

test("repository and workspace progress traverses the mounted Tauri channel with exact acknowledgements", async () => {
  let channel!: { onmessage: (value: unknown) => void };
  let id!: string;
  const acknowledged: number[] = [];
  const acceptance = { connect: vi.fn(), accept: vi.fn() };
  const { result } = renderHook(() =>
    useObservationFeed("owner-a", acceptance),
  );
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === "observation_subscribe") {
      const subscription = args as {
        subscriptionId: string;
        onEvent: typeof channel;
      };
      channel = subscription.onEvent;
      id = subscription.subscriptionId;
    }
    if (command === "observation_acknowledge") {
      const { acknowledgement } = args as {
        acknowledgement: { cursor: { sequence: number } };
      };
      acknowledged.push(acknowledgement.cursor.sequence);
    }
  });
  await waitFor(() => expect(channel).toBeDefined());
  for (const sample of corpus.feed) {
    act(() =>
      channel.onmessage({
        kind: "batch",
        batch: { ...sample, subscription_id: id },
      }),
    );
    await waitFor(() =>
      expect(result.current.accepted?.sequence).toBe(sample.cursor.sequence),
    );
  }
  expect(acknowledged).toEqual([1, 2]);
  expect(acceptance.accept).toHaveBeenCalled();
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === "observation_acknowledge"),
  ).toHaveLength(2);
  expect(
    vi
      .mocked(invoke)
      .mock.calls.every(([command]) => command.startsWith("observation_")),
  ).toBe(true);
});

test("heartbeat acceptance does not render an unchanged feed selection", async () => {
  let deliver: ((event: unknown) => void) | undefined;
  let subscriptionId: unknown;
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command !== "observation_subscribe") return;
    if (
      !object(args) ||
      !object(args.onEvent) ||
      typeof args.onEvent.onmessage !== "function"
    )
      throw new Error("Missing fixture channel");
    const onmessage = args.onEvent.onmessage;
    deliver = (event) => onmessage(event);
    subscriptionId = args.subscriptionId;
  });
  const acceptance = { connect: vi.fn(), accept: vi.fn() };
  let renders = 0;
  renderHook(() => {
    renders++;
    return useObservationFeed("owner-a", acceptance);
  });
  await act(async () => {});
  const send = (batch_id: number, kind: "resync_required" | "heartbeat") =>
    deliver?.({
      kind: "batch",
      batch: {
        subscription_id: subscriptionId,
        batch_id,
        cursor: { owner: "owner-a", sequence: 1 },
        sample: { kind, watermark: 1 },
      },
    });
  act(() => send(1, "resync_required"));
  const before = renders;
  act(() => send(2, "heartbeat"));
  expect(acceptance.accept).toHaveBeenCalledTimes(2);
  expect(renders).toBe(before);
});
