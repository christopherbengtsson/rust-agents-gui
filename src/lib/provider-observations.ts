import type { QueryClient } from "@tanstack/react-query";
import type {
  ProviderReply,
  ProviderObservation,
  ProviderRequest,
  Target,
} from "@/generated/contracts/types";
import { FeatureQueryCache } from "./feature-query-cache";
import { FeatureFreshness } from "./feature-freshness";
import { ObservationCoordinator } from "./observation-coordinator";
import { providerRequest } from "./providers";

type PendingAction = {
  owner: string;
  target: string;
  attempt: string;
  state: "pending" | "uncertain" | "rejected";
  message?: string;
  release(): void;
};

type Read =
  | { operation: "target" }
  | { operation: "observe"; target_id: string };
export class ProviderObservations extends FeatureQueryCache<
  Read,
  ProviderReply
> {
  // Identities and delivery feedback only; Query owns all provider observations.
  private actions = new Map<string, PendingAction>();
  private actionListeners = new Set<() => void>();
  subscribeActions = (listener: () => void) => {
    this.actionListeners.add(listener);
    return () => {
      this.actionListeners.delete(listener);
    };
  };
  private publishActions() {
    this.actionListeners.forEach((listener) => listener());
  }
  action(owner: string, target: string) {
    return this.actions.get(`${owner}:${target}`);
  }
  async startBrowser(owner: string, observation: ProviderObservation) {
    const previous = this.action(owner, observation.target.id);
    if (previous && previous.state !== "rejected") return;
    await this.sendAction(owner, {
      operation: "start_browser_login",
      target_id: observation.target.id,
      attempt_id: crypto.randomUUID(),
      expected_revision: observation.revision,
    });
  }
  async loginAction(
    owner: string,
    target: string,
    attempt: string,
    operation: "cancel_login" | "reopen_browser" | "reconcile_login",
  ) {
    if (this.action(owner, target)?.state === "pending") return;
    await this.sendAction(owner, {
      operation,
      target_id: target,
      attempt_id: attempt,
    });
  }
  private async sendAction(
    owner: string,
    request: Extract<ProviderRequest, { attempt_id: string }>,
  ) {
    const key = `${owner}:${request.target_id}`;
    const previous = this.actions.get(key);
    let action: PendingAction = {
      owner,
      target: request.target_id,
      attempt: request.attempt_id,
      state: "pending",
      release:
        previous && previous.state !== "rejected"
          ? previous.release
          : this.observe(owner, {
              operation: "observe",
              target_id: request.target_id,
            }),
    };
    this.actions.set(key, action);
    this.publishActions();
    try {
      const reply = await providerRequest(owner, request);
      this.acceptAction(owner, request.target_id, reply);
      if (this.actions.get(key) === action) {
        this.actions.delete(key);
        action.release();
      }
    } catch (error) {
      const message =
        error &&
        typeof error === "object" &&
        "message" in error &&
        typeof error.message === "string"
          ? error.message.slice(0, 512)
          : "Sign-in action could not be confirmed.";
      // Keep interest and identity even if the start/cancel reply is lost.
      action = { ...action, state: "uncertain", message };
      this.actions.set(key, action);
      try {
        const reply = await providerRequest(owner, {
          operation: "reconcile_login",
          target_id: request.target_id,
          attempt_id: request.attempt_id,
        });
        this.acceptAction(owner, request.target_id, reply);
        if (this.actions.get(key) === action) {
          action.release();
          if (
            reply.kind === "observation" &&
            reply.observation.login?.id === request.attempt_id &&
            request.operation !== "reopen_browser"
          )
            this.actions.delete(key);
          else this.actions.set(key, { ...action, state: "rejected", message });
        }
      } catch {
        /* A failed status read grants no retry authority. */
      }
    } finally {
      this.publishActions();
      void this.refresh(owner, false, [
        { operation: "observe", target_id: request.target_id },
      ]);
    }
  }
  private acceptAction(owner: string, target: string, reply: ProviderReply) {
    const request = { operation: "observe", target_id: target } as const;
    const current = this.snapshot(owner, request)?.data?.value;
    if (
      reply.kind === "observation" &&
      (current?.kind !== "observation" ||
        reply.observation.revision >= current.observation.revision)
    )
      this.record(owner, request, reply);
  }
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
    let readyOwner: string | undefined;
    generations.subscribe(() => {
      const owner = generations.currentOwner();
      const ready = owner && generations.ready(owner) ? owner : undefined;
      if (ready && ready !== readyOwner) {
        for (const action of this.actions.values())
          if (action.owner === ready && action.state === "uncertain")
            void this.loginAction(
              ready,
              action.target,
              action.attempt,
              "reconcile_login",
            );
      }
      readyOwner = ready;
    });
  }
  invalidate(target: Target) {
    if (target.kind === "provider")
      this.generations.mark(`provider:${target.target_id}`);
  }
}
