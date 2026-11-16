import { useSelector } from "@xstate/react";
import { useEffect, useState } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import { ObservationFeed, type FeedAcceptance } from "@/lib/observation-feed";

export function useObservationFeed(
  incarnation: string | undefined,
  acceptance: FeedAcceptance,
) {
  const [feed] = useState(
    () =>
      new ObservationFeed(
        {
          subscribe: async (
            expectedIncarnation,
            subscriptionId,
            after,
            receive,
          ) => {
            const onEvent = new Channel<unknown>(receive);
            try {
              await invoke("observation_subscribe", {
                expectedIncarnation,
                subscriptionId,
                after: after ?? null,
                onEvent,
              });
            } catch (error) {
              // No Rust channel incarnation exists if registration failed.
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
  const state = useSelector(
    feed.actor,
    feed.getSnapshot,
    (a, b) =>
      a.state === b.state &&
      a.message === b.message &&
      a.accepted?.incarnation === b.accepted?.incarnation &&
      a.accepted?.sequence === b.accepted?.sequence,
  );
  useEffect(() => {
    feed.connect(incarnation);
    return () => feed.connect(undefined);
  }, [feed, incarnation]);
  return state;
}
