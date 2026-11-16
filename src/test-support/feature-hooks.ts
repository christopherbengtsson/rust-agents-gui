import { createMemoryHistory } from "@tanstack/react-router";
import { Navigation, mainDestination } from "../lib/navigation";
import { readSelection } from "../lib/repositories";
import { readWorkspaceSelection } from "../lib/workspaces";
import { useEffect, useState, useSyncExternalStore } from "react";
import { AppObservations } from "../lib/app-observations";
import type { ObservationCoordinator } from "../lib/observation-coordinator";
import { useRepositories as useRepositoryFeature } from "../hooks/use-repositories";
import { useWorkspaces as useWorkspaceFeature } from "../hooks/use-workspaces";

// Isolated feature scenarios supply an accepted baseline explicitly. App scenarios
// exercise the real feed sink; physical scheduling scenarios share their real gate.
function useFixtureApp(
  incarnation: string | undefined,
  reads?: ObservationCoordinator | AppObservations,
) {
  const [app] = useState(() =>
    reads instanceof AppObservations ? reads : new AppObservations(reads),
  );
  useEffect(() => {
    app.connect(incarnation);
    if (incarnation)
      app.accept({
        subscription_id: "fixture",
        batch_id: 1,
        cursor: { incarnation, sequence: 1 },
        sample: { kind: "resync_required", watermark: 1 },
      });
  }, [app, incarnation]);
  return app;
}
// A controlled history supplies explicit targets to these isolated hook units.
function useFixtureNavigation(repository: string | null, workspace?: string) {
  const [history] = useState(() =>
    createMemoryHistory({
      initialEntries: [
        workspace
          ? `/repository/${repository}/workspace/${workspace}?view=workspace`
          : repository
            ? `/repository/${repository}`
            : "/",
      ],
    }),
  );
  const [navigation] = useState(
    () =>
      new Navigation(history, (target) => {
        history.push(
          target.kind === "home"
            ? "/"
            : target.kind === "repository"
              ? `/repository/${target.repositoryId}`
              : `/repository/${target.repositoryId}/workspace/${target.workspaceId}?view=${target.view}`,
        );
      }),
  );
  useEffect(() => navigation.start(), [navigation]);
  const href = useSyncExternalStore(
    (listener) => history.subscribe(listener),
    () => history.location.href,
  );
  return { navigation, target: mainDestination(href) };
}
export function useRepositories(
  incarnation: string | undefined,
  reads?: ObservationCoordinator | AppObservations,
  repositoryTarget: string | null = readSelection() ??
    "00000000-0000-4000-8000-000000000001",
) {
  const app = useFixtureApp(incarnation, reads);
  const { navigation, target } = useFixtureNavigation(repositoryTarget);
  return useRepositoryFeature(
    incarnation,
    app,
    target && target.kind !== "home" ? target.repositoryId : null,
    navigation,
  );
}
export function useWorkspaces(
  incarnation: string | undefined,
  repositoryId: string | undefined,
  repositoryIds: string,
  initial: { repository_id: string; workspace_id: string } | null,
  reads?: ObservationCoordinator | AppObservations,
) {
  const app = useFixtureApp(incarnation, reads);
  const saved = readWorkspaceSelection();
  const { navigation, target } = useFixtureNavigation(
    repositoryId ?? "fixture",
    saved?.workspace_id ?? initial?.workspace_id,
  );
  return useWorkspaceFeature(
    incarnation,
    repositoryId,
    repositoryIds,
    target?.kind === "workspace" ? target.workspaceId : undefined,
    app,
    navigation,
  );
}
