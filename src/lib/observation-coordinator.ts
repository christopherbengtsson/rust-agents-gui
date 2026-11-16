import PQueue from "p-queue";

// Presentation scheduling only: cancellation never interrupts physical IPC.
export type ObservationScope = {
  read<T>(key: readonly unknown[], run: () => Promise<T>): Promise<T>;
  dispose(): void;
};

export class ObsoleteRead extends Error {
  constructor() {
    super("Observation interest is obsolete");
  }
}

type Consumer = {
  obsolete: boolean;
  resolve(value: unknown): void;
  reject(error: unknown): void;
  detach(): void;
};
type Read = {
  key: string;
  state: "queued" | "active" | "done";
  cancel: AbortController;
  consumers: Set<Consumer>;
};

export class ObservationCoordinator {
  private queue = new PQueue({ concurrency: 2 });
  private pending = new Map<string, Read>();

  scope(): ObservationScope {
    const controller = new AbortController();
    return {
      read: <T>(key: readonly unknown[], run: () => Promise<T>) =>
        this.read(key, run, controller.signal),
      dispose: () => controller.abort(),
    };
  }

  private read<T>(
    target: readonly unknown[],
    run: () => Promise<T>,
    signal: AbortSignal,
  ): Promise<T> {
    if (signal.aborted) return Promise.reject(new ObsoleteRead());
    // Complete feature keys include incarnation and all request flags; object order is irrelevant.
    const key = JSON.stringify(target, (_, value) =>
      value !== null && typeof value === "object" && !Array.isArray(value)
        ? Object.fromEntries(
            Object.entries(value).sort(([a], [b]) => a.localeCompare(b)),
          )
        : value,
    );
    let entry = this.pending.get(key);
    const created = !entry;
    if (!entry) {
      if (this.queue.size >= 64)
        return Promise.reject({
          code: "busy",
          delivery: "not_sent",
          source: "local",
          message:
            "Observation queue is full. Wait for pending reads to finish, then refresh.",
        });
      entry = {
        key,
        state: "queued",
        cancel: new AbortController(),
        consumers: new Set(),
      };
      this.pending.set(key, entry);
    }
    const read = entry;
    const result = new Promise<T>((resolve, reject) => {
      const consumer: Consumer = {
        obsolete: false,
        // The complete feature key identifies the result type as well as its target.
        resolve: (value) => resolve(value as T),
        reject,
        detach: () => signal.removeEventListener("abort", dispose),
      };
      const dispose = () => {
        consumer.obsolete = true;
        consumer.detach();
        if (
          read.state === "queued" &&
          [...read.consumers].every((consumer) => consumer.obsolete)
        ) {
          // This controller can only abort waiting work. It is never forwarded
          // once the task starts, so p-queue retains its physical slot.
          read.cancel.abort(new ObsoleteRead());
          this.settle(read, { ok: false, error: new ObsoleteRead() });
        }
        // Active consumers settle only when physical work does. In particular,
        // the inspection reader's sequence must not release its lane on disposal.
        // Keep active keys discoverable so a new interest cannot duplicate IPC.
      };
      read.consumers.add(consumer);
      signal.addEventListener("abort", dispose, { once: true });
    });
    if (created) {
      void this.queue
        .add(
          async () => {
            read.state = "active";
            try {
              const value = await run();
              this.settle(read, { ok: true, value });
            } catch (error) {
              this.settle(read, { ok: false, error });
            }
          },
          { signal: read.cancel.signal },
        )
        .catch((error: unknown) => {
          if (read.state !== "done") this.settle(read, { ok: false, error });
        });
    }
    return result;
  }

  private forget(read: Read) {
    if (this.pending.get(read.key) === read) this.pending.delete(read.key);
  }

  private settle(
    read: Read,
    outcome: { ok: true; value: unknown } | { ok: false; error: unknown },
  ) {
    read.state = "done";
    this.forget(read);
    for (const consumer of read.consumers) {
      consumer.detach();
      if (consumer.obsolete) consumer.reject(new ObsoleteRead());
      else if (outcome.ok) consumer.resolve(outcome.value);
      else consumer.reject(outcome.error);
    }
    read.consumers.clear();
  }
}
