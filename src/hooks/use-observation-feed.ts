import { useEffect, useState, useSyncExternalStore } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import { ObservationFeed, type FeedAcceptance } from "@/lib/observation-feed";

export function useObservationFeed(
  owner: string | undefined,
  acceptance: FeedAcceptance,
) {
  const [feed] = useState(
    () =>
      new ObservationFeed(
        {
          subscribe: async (expectedOwner, subscriptionId, after, receive) => {
            const onEvent = new Channel<unknown>(receive);
            try {
              await invoke("observation_subscribe", {
                expectedOwner,
                subscriptionId,
                after: after ?? null,
                onEvent,
              });
            } catch (error) {
              // No Rust channel owner exists if registration failed.
              onEvent.onmessage = () => {};
              throw error;
            }
          },
          acknowledge: (acknowledgement) =>
            invoke("observation_acknowledge", { acknowledgement }),
          stop: (subscriptionId) =>
            invoke("observation_stop", { subscriptionId }),
        },
        acceptance,
      ),
  );
  const state = useSyncExternalStore(feed.subscribe, feed.getSnapshot);
  useEffect(() => {
    feed.connect(owner);
    return () => feed.connect(undefined);
  }, [feed, owner]);
  return state;
}
