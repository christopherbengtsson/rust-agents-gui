import { useEffect, useState } from "react";
import { AppObservations } from "../lib/app-observations";
import type { ObservationCoordinator } from "../lib/observation-coordinator";
import { useRepositories as useRepositoryFeature } from "../hooks/use-repositories";
import { useWorkspaces as useWorkspaceFeature } from "../hooks/use-workspaces";

// Isolated feature scenarios supply an accepted baseline explicitly. App scenarios
// exercise the real feed sink; physical scheduling scenarios share their real gate.
function useFixtureApp(
  owner: string | undefined,
  reads?: ObservationCoordinator | AppObservations,
) {
  const [app] = useState(() =>
    reads instanceof AppObservations ? reads : new AppObservations(reads),
  );
  useEffect(() => {
    app.connect(owner);
    if (owner)
      app.accept({
        subscription_id: "fixture",
        batch_id: 1,
        cursor: { owner, sequence: 1 },
        sample: { kind: "resync_required", watermark: 1 },
      });
  }, [app, owner]);
  return app;
}
export function useRepositories(
  owner: string | undefined,
  reads?: ObservationCoordinator | AppObservations,
) {
  return useRepositoryFeature(owner, useFixtureApp(owner, reads));
}
export function useWorkspaces(
  owner: string | undefined,
  repositoryId: string | undefined,
  repositoryIds: string,
  initial: { repository_id: string; workspace_id: string } | null,
  reads?: ObservationCoordinator | AppObservations,
) {
  return useWorkspaceFeature(
    owner,
    repositoryId,
    repositoryIds,
    initial,
    useFixtureApp(owner, reads),
  );
}
