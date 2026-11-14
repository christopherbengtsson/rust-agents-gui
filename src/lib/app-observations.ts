import { createStore } from "zustand/vanilla";
import type { FeedBatch } from "./observation-feed";
import { ObservationCoordinator } from "./observation-coordinator";
import { FeatureFreshness } from "./feature-freshness";
import { createObservationQueryClient } from "./feature-query-cache";
import { RepositoryObservations } from "./repository-observations";
import { WorkspaceObservations } from "./workspace-observations";
import { WorkspaceChangesReader } from "./workspace-changes-reader";
import { readSelection } from "./repositories";
import { readWorkspaceSelection } from "./workspaces";

export class AppObservations {
  readonly client = createObservationQueryClient();
  readonly freshness = new FeatureFreshness();
  readonly repositories: RepositoryObservations;
  readonly workspaces: WorkspaceObservations;
  readonly inspection: WorkspaceChangesReader;
  // Presentation identities only. Query owns records; retained action authority
  // stays in its existing versioned records. F05 adds reading-context persistence.
  readonly presentation = createStore<{
    repository: string | null;
    workspace: ReturnType<typeof readWorkspaceSelection>;
  }>(() => ({
    repository: readSelection(),
    workspace: readWorkspaceSelection(),
  }));
  constructor(readonly reads = new ObservationCoordinator()) {
    this.repositories = new RepositoryObservations(
      this.client,
      reads,
      this.freshness,
    );
    this.workspaces = new WorkspaceObservations(
      this.client,
      reads,
      this.freshness,
    );
    this.inspection = new WorkspaceChangesReader(reads);
  }
  connect = (owner: string | undefined) => {
    this.freshness.connect(owner);
  };
  accept = (batch: FeedBatch) => {
    if (!this.freshness.matches(batch.cursor.owner)) return;
    if (batch.sample.kind === "resync_required") this.freshness.resync();
    if (batch.sample.kind === "hints")
      for (const target of batch.sample.targets) {
        this.repositories.invalidate(target);
        this.workspaces.invalidate(target);
        this.inspection.invalidate(target, this.freshness);
      }
    this.freshness.accept(batch.cursor.owner);
  };
}
