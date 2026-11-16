// Explicit qualification build only; normal frontend builds remove the caller.
import type { QueryClient } from "@tanstack/react-query";
export function installNativeRetention(client: QueryClient) {
  const measure = () => {
    for (let index = 0; index < 160; index++)
      client.setQueryData(["native-retention-fixture", index], { index });
    queueMicrotask(() => {
      const queries = client.getQueryCache().getAll();
      window.dispatchEvent(
        new CustomEvent("native-retention-result", {
          detail: {
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
