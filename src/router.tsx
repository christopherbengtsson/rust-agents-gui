import {
  createBrowserHistory,
  createRouter,
  type RouterHistory,
} from "@tanstack/react-router";
import { routeTree } from "./routeTree.gen";
import { Navigation } from "./lib/navigation";
import type { AppObservations } from "./lib/app-observations";

export type RouteContext = { app: AppObservations; navigation: Navigation };
export function createAppRouter(
  app: AppObservations,
  history: RouterHistory = createBrowserHistory(),
) {
  const navigation = new Navigation(history, (target, replace) => {
    if (target.kind === "home") void router.navigate({ to: "/", replace });
    else if (target.kind === "repository")
      void router.navigate({
        to: "/repository/$repositoryId",
        params: { repositoryId: target.repositoryId },
        replace,
      });
    else
      void router.navigate({
        to: "/repository/$repositoryId/workspace/$workspaceId",
        params: {
          repositoryId: target.repositoryId,
          workspaceId: target.workspaceId,
        },
        search: { view: target.view },
        replace,
      });
  });
  const router = createRouter({
    routeTree,
    history,
    context: { app, navigation },
    defaultPreloadStaleTime: 0,
    defaultStaleTime: 0,
    defaultGcTime: 0,
    defaultPendingMs: 0,
    defaultPendingMinMs: 0,
  });
  return router;
}
declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}
