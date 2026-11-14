import { createRootRouteWithContext, Link } from "@tanstack/react-router";
import type { RouteContext } from "@/router";
import { AppRuntime } from "@/app-runtime";
export const Route = createRootRouteWithContext<RouteContext>()({
  component: AppRuntime,
  notFoundComponent: () => (
    <main id="workspace" className="workspace-region flex-1">
      <h1>Destination unavailable</h1>
      <Link to="/">Return home</Link>
    </main>
  ),
});
