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
  run(): Promise<unknown>;
  consumers: Set<Consumer>;
};

export class ObservationCoordinator {
  private active = 0;
  private queue: Read[] = [];
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
    // Complete feature keys include owner and all request flags; object order is irrelevant.
    const key = JSON.stringify(target, (_, value) =>
      value !== null && typeof value === "object" && !Array.isArray(value)
        ? Object.fromEntries(
            Object.entries(value).sort(([a], [b]) => a.localeCompare(b)),
          )
        : value,
    );
    let entry = this.pending.get(key);
    if (!entry) {
      if (this.active >= 2 && this.queue.length >= 64)
        return Promise.reject({
          code: "busy",
          delivery: "not_sent",
          source: "local",
          message:
            "Observation queue is full. Wait for pending reads to finish, then refresh.",
        });
      entry = { key, state: "queued", run, consumers: new Set() };
      this.pending.set(key, entry);
      this.queue.push(entry);
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
          this.queue = this.queue.filter((entry) => entry !== read);
          this.settle(read, { ok: false, error: new ObsoleteRead() });
        }
        // Active consumers settle only when physical work does. In particular,
        // the inspection reader's sequence must not release its lane on disposal.
        // Keep active keys discoverable so a new interest cannot duplicate IPC.
      };
      read.consumers.add(consumer);
      signal.addEventListener("abort", dispose, { once: true });
    });
    this.drain();
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

  private drain() {
    while (this.active < 2 && this.queue.length) {
      const read = this.queue.shift()!;
      read.state = "active";
      this.active++;
      let physical: Promise<unknown>;
      try {
        physical = read.run();
      } catch (error) {
        physical = Promise.reject(error);
      }
      void physical
        .then(
          (value) => this.settle(read, { ok: true, value }),
          (error: unknown) => this.settle(read, { ok: false, error }),
        )
        .finally(() => {
          this.active--;
          this.drain();
        });
    }
  }
}
