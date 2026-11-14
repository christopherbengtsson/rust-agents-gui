import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, test, vi } from "vitest";
import { useObservationFeed } from "./hooks/use-observation-feed";
import corpus from "../contracts/observations-v8.json";

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
  const { result } = renderHook(() => useObservationFeed("owner-a"));
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
  expect(result.current.dirty.all).toBe(true);
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
