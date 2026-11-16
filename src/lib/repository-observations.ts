import type { QueryClient } from "@tanstack/react-query";
import type { Target } from "@/generated/contracts/types";
import { FeatureQueryCache } from "./feature-query-cache";
import { FeatureFreshness } from "./feature-freshness";
import { ObservationCoordinator } from "./observation-coordinator";
import { repositoryRequest, type RepositoryReply } from "./repositories";

type Read =
  | { operation: "list"; after: string | null }
  | { operation: "get"; id: string }
  | { operation: "action"; action_id: string };
export class RepositoryObservations extends FeatureQueryCache<
  Read,
  RepositoryReply
> {
  constructor(
    client: QueryClient,
    reads: ObservationCoordinator,
    private generations: FeatureFreshness,
  ) {
    super(
      "repositories",
      client,
      reads,
      generations,
      (request) => {
        switch (request.operation) {
          case "list":
            return ["repositories"];
          case "get":
            return [`repository:${request.id}`];
          case "action":
            return [`registration:${request.action_id}`];
        }
      },
      (incarnation, request, refresh) =>
        repositoryRequest(
          incarnation,
          request.operation === "action" ? request : { ...request, refresh },
        ),
      (request) => request.operation !== "action",
    );
  }
  refreshSelection(
    incarnation: string,
    after: string | null,
    selectedId: string | null,
    actionId: string | undefined,
    reprobe: boolean,
  ) {
    const targets: Read[] = [{ operation: "list", after }];
    if (selectedId) targets.push({ operation: "get", id: selectedId });
    if (actionId) targets.push({ operation: "action", action_id: actionId });
    return this.refresh(incarnation, reprobe, targets);
  }
  invalidate(target: Target) {
    switch (target.kind) {
      case "repositories":
        this.generations.mark("repositories");
        break;
      case "repository":
        this.generations.mark(`repository:${target.repository_id}`);
        break;
      case "registration":
        this.generations.mark(`registration:${target.action_id}`);
        break;
    }
  }
}
