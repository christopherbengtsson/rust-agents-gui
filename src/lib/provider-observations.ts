import type { QueryClient } from "@tanstack/react-query";
import type { ProviderReply, Target } from "@/generated/contracts/types";
import { FeatureQueryCache } from "./feature-query-cache";
import { FeatureFreshness } from "./feature-freshness";
import { ObservationCoordinator } from "./observation-coordinator";
import { providerRequest } from "./providers";

type Read =
  | { operation: "target" }
  | { operation: "observe"; target_id: string };
export class ProviderObservations extends FeatureQueryCache<
  Read,
  ProviderReply
> {
  constructor(
    client: QueryClient,
    reads: ObservationCoordinator,
    private generations: FeatureFreshness,
  ) {
    super(
      "providers",
      client,
      reads,
      generations,
      (request) =>
        request.operation === "target"
          ? ["provider-target"]
          : [`provider:${request.target_id}`],
      (owner, request, refresh) =>
        providerRequest(
          owner,
          request.operation === "target" ? request : { ...request, refresh },
        ),
      (request) => request.operation === "observe",
    );
  }
  invalidate(target: Target) {
    if (target.kind === "provider")
      this.generations.mark(`provider:${target.target_id}`);
  }
}
