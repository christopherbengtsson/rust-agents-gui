import { identity } from "@/lib/observation-contracts";
import { TargetError } from "@/screens/target-error";
import { createFileRoute, Outlet } from "@tanstack/react-router";
export const Route = createFileRoute("/repository/$repositoryId")({
  params: {
    parse: (params) => {
      if (!identity(params.repositoryId))
        throw new Error("Invalid repository identity in this URL.");
      return params;
    },
  },
  errorComponent: TargetError,
  loader: ({ context: { app }, params, abortController }) => {
    const incarnation = app.freshness.currentIncarnation();
    if (incarnation)
      void app.repositories.prepare(
        incarnation,
        { operation: "get", id: params.repositoryId },
        abortController.signal,
      );
  },
  component: RepositoryRoute,
});
function RepositoryRoute() {
  return <Outlet />;
}
