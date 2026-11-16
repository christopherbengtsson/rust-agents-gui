import { useStore } from "zustand";
import { Alert, AlertTitle, AlertDescription } from "@/components/ui/alert";
import { createContext, useContext, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  Outlet,
  Link,
  useLocation,
  useRouteContext,
  useRouterState,
} from "@tanstack/react-router";
import { useObservationFeed } from "@/hooks/use-observation-feed";
import { useCoreConnection } from "@/hooks/use-core-connection";
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
import { useProviderProgress } from "@/hooks/use-provider-progress";

export function targetStatus(
  app: AppObservations,
  incarnation: string | undefined,
  target: MainDestination | undefined,
) {
  if (!target || target.kind === "home") return "valid";
  if (!incarnation) return "unavailable";
  const repository = { operation: "get", id: target.repositoryId } as const;
  const repo = app.repositories.peek(incarnation, repository);
  if (!app.repositories.isCurrent(incarnation, repository)) return "pending";
  if (repo?.error) return "unavailable";
  if (repo?.data?.value.kind !== "repository" || !repo.data.value.repository)
    return "missing";
  if (target.kind === "repository") return "valid";
  const workspace = { operation: "get", id: target.workspaceId } as const;
  const state = app.workspaces.peek(incarnation, workspace);
  if (!app.workspaces.isCurrent(incarnation, workspace)) return "pending";
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
  const { connection, refreshStatus } = useCoreConnection();
  const incarnation =
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.incarnation
      : undefined;
  const feed = useObservationFeed(incarnation, app);
  const observedConnection = { ...connection, feed };
  const repositoryId =
    target && target.kind !== "home" ? target.repositoryId : undefined;
  const repositories = useRepositories(
    incarnation,
    app,
    repositoryId ?? null,
    navigation,
    mainVisible,
  );
  const workspaces = useWorkspaces(
    incarnation,
    repositoryId,
    repositories.page?.repositories.map((r) => r.id).join(",") ?? "",
    target?.kind === "workspace" ? target.workspaceId : undefined,
    app,
    navigation,
    mainVisible,
  );
  const status = targetStatus(app, incarnation, target);
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
  const providerProgress = useProviderProgress(
    runtime.app,
    runtime.connection.state === "connected"
      ? runtime.connection.observation.snapshot.cursor.incarnation
      : undefined,
    runtime.connection.observation?.snapshot.provider_login_pending === true,
  );
  const [shutdown, setShutdown] = useState<string>();
  useEffect(() => {
    let disposed = false;
    let release: (() => void) | undefined;
    void listen<unknown>("owned-shutdown", (event) => {
      if (
        !disposed &&
        typeof event.payload === "string" &&
        event.payload.length <= 4096
      )
        setShutdown(event.payload);
    })
      .then((stop) => {
        if (disposed) stop();
        else release = stop;
      })
      .catch(() => {
        if (!disposed)
          setShutdown(
            "Desktop shutdown updates are unavailable. Keep the app open if Quit cannot confirm owned cleanup.",
          );
      });
    return () => {
      disposed = true;
      release?.();
    };
  }, []);
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
          onClick={(event) => {
            if (
              event.button !== 0 ||
              event.metaKey ||
              event.ctrlKey ||
              event.shiftKey ||
              event.altKey
            )
              return;
            event.preventDefault();
            document
              .getElementById(settings ? "settings-main" : "workspace")
              ?.focus();
          }}
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
          {shutdown && (
            <Alert>
              <AlertTitle>Owned shutdown</AlertTitle>
              <AlertDescription>{shutdown}</AlertDescription>
            </Alert>
          )}
          {providerProgress.active && (
            <div className="flex shrink-0 items-center gap-2 border-b px-4 py-2 text-sm">
              <Link
                to="/settings/providers"
                search={{ provider: "codex" }}
                replace={settings}
              >
                Codex sign-in{" "}
                {providerProgress.observation?.login?.phase ===
                "cleanup_uncertain"
                  ? "needs attention"
                  : "in progress"}
              </Link>
              {providerProgress.stale && (
                <span className="text-muted-foreground">
                  Last known · status unavailable
                </span>
              )}
            </div>
          )}
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
