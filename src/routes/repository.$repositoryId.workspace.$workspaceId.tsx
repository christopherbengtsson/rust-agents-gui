import { identity } from "@/lib/observation-contracts";
import { createFileRoute } from "@tanstack/react-router";
import { workspaceSearch } from "@/lib/navigation";
import { MainScreen } from "@/screens/main";
import { InvalidView } from "@/screens/invalid-view";
export const Route = createFileRoute(
  "/repository/$repositoryId/workspace/$workspaceId",
)({
  params: {
    parse: (params) => {
      if (!identity(params.workspaceId))
        throw new Error("Invalid workspace identity in this URL.");
      return params;
    },
  },
  validateSearch: workspaceSearch,
  loader: ({ context: { app }, params, abortController }) => {
    const owner = app.freshness.currentOwner();
    if (owner)
      void app.workspaces.prepare(
        owner,
        { operation: "get", id: params.workspaceId },
        abortController.signal,
      );
  },
  component: MainScreen,
  errorComponent: InvalidView,
});
