import { useStore } from "zustand";
import { Alert, AlertTitle, AlertDescription } from "@/components/ui/alert";
import { createContext, useContext, useEffect } from "react";
import {
  Outlet,
  useLocation,
  useRouteContext,
  useRouterState,
} from "@tanstack/react-router";
import { useObservationFeed } from "@/hooks/use-observation-feed";
import { useSupervisorConnection } from "@/hooks/use-supervisor-connection";
import { useRepositories } from "@/hooks/use-repositories";
import { useWorkspaces } from "@/hooks/use-workspaces";
import { RepositorySidebar } from "@/components/blocks/repository-sidebar";
import { SidebarProvider } from "@/components/ui/sidebar";
import {
  isSettingsPath,
  mainDestination,
  type MainDestination,
} from "@/lib/navigation";
import type { AppObservations } from "@/lib/app-observations";

export function targetStatus(
  app: AppObservations,
  owner: string | undefined,
  target: MainDestination | undefined,
) {
  if (!target || target.kind === "home") return "valid";
  if (!owner) return "unavailable";
  const repository = { operation: "get", id: target.repositoryId } as const;
  const repo = app.repositories.peek(owner, repository);
  if (!app.repositories.isCurrent(owner, repository)) return "pending";
  if (repo?.error) return "unavailable";
  if (repo?.data?.value.kind !== "repository" || !repo.data.value.repository)
    return "missing";
  if (target.kind === "repository") return "valid";
  const workspace = { operation: "get", id: target.workspaceId } as const;
  const state = app.workspaces.peek(owner, workspace);
  if (!app.workspaces.isCurrent(owner, workspace)) return "pending";
  if (state?.error) return "unavailable";
  return state?.data?.value.kind === "workspace" &&
    state.data.value.workspace?.repository_id === target.repositoryId
    ? "valid"
    : "missing";
}
function useRuntime() {
  const { app, navigation } = useRouteContext({ from: "__root__" });
  const location = useLocation();
  const mainVisible = !isSettingsPath(location.pathname);
  const target = mainDestination(location.href);
  const { connection, refreshStatus } = useSupervisorConnection();
  const owner =
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined;
  const feed = useObservationFeed(owner, app);
  const observedConnection = { ...connection, feed };
  const repositoryId =
    target && target.kind !== "home" ? target.repositoryId : undefined;
  const repositories = useRepositories(
    owner,
    app,
    repositoryId ?? null,
    navigation,
    mainVisible,
  );
  const workspaces = useWorkspaces(
    owner,
    repositoryId,
    repositories.page?.repositories.map((r) => r.id).join(",") ?? "",
    target?.kind === "workspace" ? target.workspaceId : undefined,
    app,
    navigation,
    mainVisible,
  );
  const status = targetStatus(app, owner, target);
  useEffect(() => {
    navigation.restoreOnce(app.presentation.getState().rememberedMain);
    if (navigation.history.location.href !== location.href) return;
    if (navigation.finishRestoration(status)) return;
    if (target && status === "valid") {
      const presentation = app.presentation.getState();
      presentation.rememberMain(target);
      if (target.kind === "workspace")
        presentation.rememberView(target.workspaceId, target.view);
    }
  }, [app, navigation, location.href, status, target]);
  return {
    app,
    navigation,
    target,
    status,
    repositories,
    workspaces,
    connection: observedConnection,
    refresh: async () => {
      await refreshStatus();
      await repositories.refresh();
      await workspaces.refresh();
    },
  };
}
const RuntimeContext = createContext<ReturnType<typeof useRuntime> | null>(
  null,
);
export function useAppRuntime() {
  const runtime = useContext(RuntimeContext);
  if (!runtime) throw new Error("App runtime is missing");
  return runtime;
}
export function AppRuntime() {
  const runtime = useRuntime();
  const location = useLocation();
  const settings = isSettingsPath(location.pathname);
  const pending = useRouterState({
    select: (state) => state.status === "pending",
  });
  useEffect(() => {
    if (settings || pending) return;
    const focus = runtime.navigation.takeReturnFocus();
    if (focus) document.getElementById(focus)?.focus({ preventScroll: true });
  }, [runtime.navigation, location.href, settings, pending]);
  const storageError = useStore(
    runtime.app.presentation,
    (state) => state.storageError,
  );
  return (
    <RuntimeContext value={runtime}>
      <SidebarProvider className="h-dvh min-h-0">
        <a
          className="skip-link"
          href={settings ? "#settings-main" : "#workspace"}
        >
          {settings ? "Skip to settings" : "Skip to workspace"}
        </a>
        {!settings && (
          <RepositorySidebar
            connection={runtime.connection}
            repositories={runtime.repositories}
            workspaces={runtime.workspaces}
            onSettingsOpen={runtime.navigation.rememberSettingsTrigger}
          />
        )}
        <div className="flex min-h-0 min-w-0 flex-1 flex-col">
          {storageError && (
            <Alert>
              <AlertTitle>
                Desktop presentation preferences unavailable
              </AlertTitle>
              <AlertDescription>{storageError}</AlertDescription>
            </Alert>
          )}
          <Outlet />
        </div>
      </SidebarProvider>
    </RuntimeContext>
  );
}
