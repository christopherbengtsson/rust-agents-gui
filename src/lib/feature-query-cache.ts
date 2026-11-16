import { QueryClient, QueryObserver, hashKey } from "@tanstack/react-query";
import { ObservationCoordinator } from "./observation-coordinator";
import { FeatureFreshness } from "./feature-freshness";

export function createObservationQueryClient() {
  const client = new QueryClient({
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
  // Query owns bodies; this policy adds no result index or serialized copy.
  // Disabled feature observers still express visible/pending interests.
  let queued = false;
  client.getQueryCache().subscribe(() => {
    if (queued) return;
    queued = true;
    queueMicrotask(() => {
      queued = false;
      const inactive = client
        .getQueryCache()
        .getAll()
        .filter(
          (query) =>
            query.getObserversCount() === 0 &&
            query.state.fetchStatus === "idle",
        )
        .sort((a, b) => a.state.dataUpdatedAt - b.state.dataUpdatedAt);
      for (const query of inactive.slice(0, Math.max(0, inactive.length - 128)))
        client.getQueryCache().remove(query);
    });
  });
  return client;
}
class ObservationReadFailure extends Error {
  constructor(
    readonly generation: number,
    cause: unknown,
  ) {
    super("Observation read failed", { cause });
  }
}
type Observed<T> = { value: T; generation: number; incarnation: string };
type Interest<Q, R> = {
  incarnation: string;
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
  private lastIncarnation?: string;
  // One bounded concrete target set, supplied by the feature for an explicit
  // refresh that crosses temporary connection-interest detachment.
  private requestedRefresh?: {
    incarnation: string;
    targets: Map<string, boolean>;
  };
  constructor(
    private feature: string,
    private readonly client: QueryClient,
    private reads: ObservationCoordinator,
    private freshness: FeatureFreshness,
    private tags: (request: Q) => string[],
    private request: (
      incarnation: string,
      request: Q,
      reprobe: boolean,
    ) => Promise<R>,
    private canReprobe: (request: Q) => boolean = () => false,
  ) {
    this.freshness.subscribe(() => {
      if (
        this.requestedRefresh &&
        this.freshness.currentIncarnation() &&
        !this.freshness.matches(this.requestedRefresh.incarnation)
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
  private key(incarnation: string, request: Q) {
    return [this.feature, incarnation, request] as const;
  }
  retainedIncarnation() {
    return this.lastIncarnation;
  }
  observe(incarnation: string, request: Q) {
    this.lastIncarnation = incarnation;
    const key = this.key(incarnation, request);
    const identity = hashKey(key);
    let interest = this.interests.get(identity);
    if (!interest) {
      const observer = new QueryObserver<Observed<R>, Error>(this.client, {
        queryKey: key,
        enabled: false,
      });
      interest = {
        incarnation,
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
    if (
      requested?.incarnation === incarnation &&
      requested.targets.has(identity)
    ) {
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
  peek(incarnation: string, request: Q) {
    const state = this.snapshot(incarnation, request);
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
  // dependency under React Compiler. Keep Query as the sole body incarnation.
  snapshot(incarnation: string, request: Q) {
    return this.client.getQueryState<Observed<R>>(
      this.key(incarnation, request),
    );
  }
  isCurrent(incarnation: string, request: Q) {
    if (!this.freshness.ready(incarnation)) return false;
    const state = this.client.getQueryState<Observed<R>>(
      this.key(incarnation, request),
    );
    const version = this.freshness.version(this.tags(request));
    return state?.error instanceof ObservationReadFailure
      ? state.error.generation >= version
      : !!state?.data && state.data.generation >= version;
  }
  prepare(incarnation: string, request: Q, signal: AbortSignal): Promise<void> {
    if (signal.aborted || !this.freshness.matches(incarnation))
      return Promise.resolve();
    const release = this.observe(incarnation, request);
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
          !this.freshness.matches(incarnation) ||
          this.isCurrent(incarnation, request)
        )
          finish();
      };
      const unsubscribe = this.subscribe(check);
      signal.addEventListener("abort", finish, { once: true });
      check();
    });
  }
  protected record(incarnation: string, request: Q, value: R) {
    this.client.setQueryData<Observed<R>>(this.key(incarnation, request), {
      value,
      incarnation,
      generation: -1,
    });
  }
  async refresh(incarnation: string, reprobe = false, targets?: readonly Q[]) {
    const interested = [...this.interests.values()].filter(
      (interest) => interest.incarnation === incarnation,
    );
    const requested = new Map(
      (targets ?? interested.map((interest) => interest.request)).map(
        (request) => [hashKey(this.key(incarnation, request)), reprobe],
      ),
    );
    this.requestedRefresh = { incarnation, targets: requested };
    await Promise.all(
      interested.map(async (interest) => {
        const identity = hashKey(this.key(incarnation, interest.request));
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
    if (requested && this.freshness.ready(requested.incarnation))
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
    const { incarnation, request } = interest;
    if (!interest.users || !this.freshness.ready(incarnation)) return;
    const key = this.key(incarnation, request);
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
                  value: await this.request(incarnation, request, reprobe),
                  generation,
                  incarnation,
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
