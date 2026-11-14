import type { QueryClient } from "@tanstack/react-query";
import type { Target } from "@/generated/contracts/types";
import { FeatureQueryCache } from "./feature-query-cache";
import { FeatureFreshness } from "./feature-freshness";
import { ObservationCoordinator } from "./observation-coordinator";
import { workspaceRequest, type WorkspaceReply } from "./workspaces";

type Read =
  | { operation: "list"; repository_id: string; after: string | null }
  | { operation: "get"; id: string }
  | { operation: "action"; action_id: string };
export class WorkspaceObservations extends FeatureQueryCache<
  Read,
  WorkspaceReply
> {
  constructor(
    client: QueryClient,
    reads: ObservationCoordinator,
    private generations: FeatureFreshness,
  ) {
    super(
      "workspaces",
      client,
      reads,
      generations,
      (request) => {
        switch (request.operation) {
          case "list":
            return ["workspaces", `workspaces:${request.repository_id}`];
          case "get":
            return [`workspace:${request.id}`];
          case "action":
            return [`workspace-action:${request.action_id}`];
        }
      },
      (owner, request) => workspaceRequest(owner, request),
    );
  }
  recordAction(owner: string, reply: WorkspaceReply) {
    if (reply.kind !== "action" || !reply.action || !reply.workspace) return;
    this.recordWorkspace(owner, reply.workspace);
  }
  recordWorkspace(owner: string, workspace: import("./workspaces").Workspace) {
    const target = { operation: "get", id: workspace.id } as const;
    const existing = this.peek(owner, target);
    // Seed a newly admitted/selected identity, never replace a newer detail or
    // erase a cached failure merely because the user selects its list record.
    if (!existing?.data && existing?.status !== "error")
      this.record(owner, target, { kind: "workspace", workspace });
  }
  invalidate(target: Target) {
    switch (target.kind) {
      case "workspaces":
        this.generations.mark(
          target.repository_id === null
            ? "workspaces"
            : `workspaces:${target.repository_id}`,
        );
        break;
      case "workspace":
        this.generations.mark(`workspace:${target.workspace_id}`);
        break;
      case "workspace_action":
        this.generations.mark(`workspace-action:${target.action_id}`);
        break;
    }
  }
}
