// Explicit qualification build only; normal frontend builds remove the caller.
import type { QueryClient } from "@tanstack/react-query";
export function installNativeRetention(client: QueryClient) {
  const measure = () => {
    for (let index = 0; index < 160; index++)
      client.setQueryData(["native-retention-fixture", index], {
        index,
        body: `${index}:`.padEnd(48 * 1024, "x"),
      });
    queueMicrotask(() => {
      const queries = client.getQueryCache().getAll();
      window.dispatchEvent(
        new CustomEvent("native-retention-result", {
          detail: {
            offeredEntries: 160,
            offeredBodyBytesPerEntry: 48 * 1024,
            // Serialized retained fixture bytes are measured separately from heap.
            // This transient measurement creates no retained second body index.
            retainedFixtureJsonBytes: queries
              .filter(
                (query) => query.queryKey[0] === "native-retention-fixture",
              )
              .reduce(
                (bytes, query) =>
                  bytes +
                  new TextEncoder().encode(JSON.stringify(query.state.data))
                    .length,
                0,
              ),
            entries: queries.length,
            observed: queries.filter((query) => query.getObserversCount() > 0)
              .length,
            inactive: queries.filter(
              (query) =>
                query.getObserversCount() === 0 &&
                query.state.fetchStatus === "idle",
            ).length,
            fetching: queries.filter(
              (query) => query.state.fetchStatus !== "idle",
            ).length,
          },
        }),
      );
    });
  };
  window.addEventListener("native-retention-measure", measure);
  return () => window.removeEventListener("native-retention-measure", measure);
}
