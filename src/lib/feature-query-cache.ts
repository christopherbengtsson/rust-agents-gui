import { QueryClient, QueryObserver, hashKey } from "@tanstack/react-query";
import { ObservationCoordinator } from "./observation-coordinator";
import { FeatureFreshness } from "./feature-freshness";

export function createObservationQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        networkMode: "always",
        retry: false,
        refetchOnWindowFocus: false,
        refetchOnReconnect: false,
        refetchOnMount: false,
        staleTime: Infinity,
        gcTime: 5 * 60 * 1000,
      },
    },
  });
}
class ObservationReadFailure extends Error {
  constructor(
    readonly generation: number,
    cause: unknown,
  ) {
    super("Observation read failed", { cause });
  }
}
type Observed<T> = { value: T; generation: number; owner: string };
type Interest<Q, R> = {
  owner: string;
  request: Q;
  users: number;
  observer: QueryObserver<Observed<R>, Error>;
  unsubscribe(): void;
  force: boolean;
  reprobe: boolean;
};

// Internal Query adapter, never configured by views. It retains request metadata,
// not observation bodies. Feature modules supply their own concrete targets.
export class FeatureQueryCache<Q, R> {
  private interests = new Map<string, Interest<Q, R>>();
  private listeners = new Set<() => void>();
  private revision = 0;
  private lastOwner?: string;
  // One bounded concrete target set, supplied by the feature for an explicit
  // refresh that crosses temporary connection-interest detachment.
  private requestedRefresh?: { owner: string; targets: Map<string, boolean> };
  constructor(
    private feature: string,
    private readonly client: QueryClient,
    private reads: ObservationCoordinator,
    private freshness: FeatureFreshness,
    private tags: (request: Q) => string[],
    private request: (
      owner: string,
      request: Q,
      reprobe: boolean,
    ) => Promise<R>,
    private canReprobe: (request: Q) => boolean = () => false,
  ) {
    this.freshness.subscribe(() => {
      if (
        this.requestedRefresh &&
        this.freshness.currentOwner() &&
        !this.freshness.matches(this.requestedRefresh.owner)
      )
        this.requestedRefresh = undefined;
      this.expireRequestedRefresh();
      this.publish();
      this.interests.forEach((interest) => this.schedule(interest));
    });
  }
  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };
  getSnapshot = () => this.revision;
  private publish() {
    this.revision++;
    this.listeners.forEach((listener) => listener());
  }
  private key(owner: string, request: Q) {
    return [this.feature, owner, request] as const;
  }
  retainedOwner() {
    return this.lastOwner;
  }
  observe(owner: string, request: Q) {
    this.lastOwner = owner;
    const key = this.key(owner, request);
    const identity = hashKey(key);
    let interest = this.interests.get(identity);
    if (!interest) {
      const observer = new QueryObserver<Observed<R>, Error>(this.client, {
        queryKey: key,
        enabled: false,
      });
      interest = {
        owner,
        request,
        users: 0,
        observer,
        unsubscribe() {},
        force: false,
        reprobe: false,
      };
      this.interests.set(identity, interest);
      const current = interest;
      current.unsubscribe = observer.subscribe(() => {
        this.publish();
        this.schedule(current);
      });
    }
    const current = interest;
    current.users++;
    const requested = this.requestedRefresh;
    if (requested?.owner === owner && requested.targets.has(identity)) {
      current.force = true;
      current.reprobe ||= requested.targets.get(identity) === true;
      requested.targets.delete(identity);
      if (!requested.targets.size) this.requestedRefresh = undefined;
    }
    queueMicrotask(() => {
      if (current.users) this.publish();
    });
    this.schedule(current);
    let released = false;
    return () => {
      if (released) return;
      released = true;
      if (--current.users) return;
      this.interests.delete(identity);
      current.unsubscribe();
      current.observer.destroy();
      void this.client.cancelQueries({ queryKey: key, exact: true });
    };
  }
  peek(owner: string, request: Q) {
    const state = this.snapshot(owner, request);
    return state
      ? {
          ...state,
          error:
            state.error instanceof ObservationReadFailure
              ? state.error.cause
              : state.error,
        }
      : undefined;
  }
  // Query supplies immutable, reference-stable states for React subscriptions.
  // A discarded revision followed by separate mutable reads is not a reactive
  // dependency under React Compiler. Keep Query as the sole body owner.
  snapshot(owner: string, request: Q) {
    return this.client.getQueryState<Observed<R>>(this.key(owner, request));
  }
  isCurrent(owner: string, request: Q) {
    if (!this.freshness.ready(owner)) return false;
    const state = this.client.getQueryState<Observed<R>>(
      this.key(owner, request),
    );
    const version = this.freshness.version(this.tags(request));
    return state?.error instanceof ObservationReadFailure
      ? state.error.generation >= version
      : !!state?.data && state.data.generation >= version;
  }
  prepare(owner: string, request: Q, signal: AbortSignal): Promise<void> {
    if (signal.aborted || !this.freshness.matches(owner))
      return Promise.resolve();
    const release = this.observe(owner, request);
    return new Promise((resolve) => {
      const finish = () => {
        unsubscribe();
        signal.removeEventListener("abort", finish);
        release();
        resolve();
      };
      const check = () => {
        if (
          signal.aborted ||
          !this.freshness.matches(owner) ||
          this.isCurrent(owner, request)
        )
          finish();
      };
      const unsubscribe = this.subscribe(check);
      signal.addEventListener("abort", finish, { once: true });
      check();
    });
  }
  protected record(owner: string, request: Q, value: R) {
    this.client.setQueryData<Observed<R>>(this.key(owner, request), {
      value,
      owner,
      generation: -1,
    });
  }
  async refresh(owner: string, reprobe = false, targets?: readonly Q[]) {
    const interested = [...this.interests.values()].filter(
      (interest) => interest.owner === owner,
    );
    const requested = new Map(
      (targets ?? interested.map((interest) => interest.request)).map(
        (request) => [hashKey(this.key(owner, request)), reprobe],
      ),
    );
    this.requestedRefresh = { owner, targets: requested };
    await Promise.all(
      interested.map(async (interest) => {
        const identity = hashKey(this.key(owner, interest.request));
        if (!requested.has(identity)) return;
        interest.force = true;
        interest.reprobe ||= reprobe;
        requested.delete(identity);
        await this.pump(interest);
      }),
    );
    this.expireRequestedRefresh();
  }
  private expireRequestedRefresh() {
    const requested = this.requestedRefresh;
    if (requested && this.freshness.ready(requested.owner))
      queueMicrotask(() => {
        // The baseline's attachment wave has finished; hidden/obsolete targets
        // must not resurrect an explicit reprobe on some later navigation.
        if (this.requestedRefresh === requested)
          this.requestedRefresh = undefined;
      });
  }
  private schedule(interest: Interest<Q, R>) {
    // Acceptance only records generations synchronously; reads begin after ack.
    queueMicrotask(() => {
      void this.pump(interest);
    });
  }
  private async pump(interest: Interest<Q, R>): Promise<void> {
    const { owner, request } = interest;
    if (!interest.users || !this.freshness.ready(owner)) return;
    const key = this.key(owner, request);
    const state = this.client.getQueryState<Observed<R>>(key);
    if (state?.fetchStatus === "fetching") return;
    const version = this.freshness.version(this.tags(request));
    if (!interest.force) {
      if (
        state?.status === "error" &&
        state.error instanceof ObservationReadFailure &&
        state.error.generation >= version
      )
        return;
      if (
        state?.status !== "error" &&
        state?.data &&
        state.data.generation >= version
      )
        return;
    }
    const reprobe = interest.reprobe && this.canReprobe(request);
    interest.force = false;
    interest.reprobe = false;
    try {
      await this.client.query({
        queryKey: key,
        staleTime: 0,
        queryFn: async ({ signal }) => {
          const scope = this.reads.scope();
          const abort = () => scope.dispose();
          signal.addEventListener("abort", abort, { once: true });
          if (signal.aborted) abort();
          const rejectedGeneration = this.freshness.version(this.tags(request));
          try {
            return await scope.read([...key, { reprobe }], async () => {
              // Capture at physical start, not logical remount/coalescing.
              const generation = this.freshness.version(this.tags(request));
              try {
                return {
                  value: await this.request(owner, request, reprobe),
                  generation,
                  owner,
                };
              } catch (cause) {
                throw new ObservationReadFailure(generation, cause);
              }
            });
          } catch (cause) {
            throw cause instanceof ObservationReadFailure
              ? cause
              : new ObservationReadFailure(rejectedGeneration, cause);
          } finally {
            signal.removeEventListener("abort", abort);
            scope.dispose();
          }
        },
      });
    } catch {
      // Query retains the error and last-good data. New hints or explicit refresh
      // can try again; a failed read cannot create its own retry loop.
    }
    this.schedule(interest);
  }
}
