import { bounded, owner as validOwner, text } from "./observation-contracts";

import type {
  FeedCursor,
  FeedBatch,
  FeedAck,
  Target,
} from "@/generated/contracts/types";
import {
  validateFeedBatch,
  validateFeedEvent,
} from "@/generated/contracts/validators.js";
export type {
  FeedCursor,
  FeedBatch,
  FeedAck,
} from "@/generated/contracts/types";
export type DirtyTarget = Target;
function sequence(value: number): boolean {
  return Number.isSafeInteger(value) && value > 0;
}
export function decodeFeedBatch(
  value: unknown,
  owner: string,
  id: string,
  previous: number,
  cursor?: FeedCursor,
): FeedBatch {
  if (
    !validateFeedBatch(value) ||
    !bounded(value, 4096) ||
    value.subscription_id !== id ||
    !sequence(value.batch_id) ||
    value.batch_id !== previous + 1 ||
    !validOwner(value.cursor.owner) ||
    value.cursor.owner !== owner ||
    !sequence(value.cursor.sequence) ||
    value.sample.watermark !== value.cursor.sequence
  )
    throw new Error("Invalid observation delivery identity or bounds");
  const sample = value.sample;
  if (
    sample.kind === "hints" &&
    sample.targets.length > 0 &&
    sample.targets.length <= 32 &&
    (!cursor || value.cursor.sequence > cursor.sequence)
  )
    return {
      subscription_id: id,
      batch_id: value.batch_id,
      cursor: { owner, sequence: value.cursor.sequence },
      sample: {
        kind: "hints",
        watermark: value.cursor.sequence,
        targets: sample.targets,
      },
    };
  if (
    sample.kind === "resync_required" ||
    (sample.kind === "heartbeat" && cursor?.sequence === value.cursor.sequence)
  )
    return {
      subscription_id: id,
      batch_id: value.batch_id,
      cursor: { owner, sequence: value.cursor.sequence },
      sample: { kind: sample.kind, watermark: value.cursor.sequence },
    };
  throw new Error("Invalid observation delivery sample");
}

export type FeedAcceptance = {
  connect(owner: string | undefined): void;
  accept(batch: FeedBatch): void;
};
export type FeedState = {
  state: "idle" | "starting" | "live" | "stopped";
  message?: string;
  accepted?: FeedCursor;
};
export type FeedTransport = {
  subscribe: (
    owner: string,
    id: string,
    after: FeedCursor | undefined,
    receive: (value: unknown) => void,
  ) => Promise<void>;
  acknowledge: (acknowledgement: FeedAck) => Promise<unknown>;
  stop: (id: string) => Promise<unknown>;
};
export class ObservationFeed {
  private snapshot: FeedState = {
    state: "idle",
  };
  private listeners = new Set<() => void>();
  private accepted?: FeedCursor;
  private generation = 0;
  private serial = Promise.resolve();
  private current?: {
    id: string;
    timer?: ReturnType<typeof setTimeout>;
    stopping?: Promise<unknown>;
  };
  constructor(
    private transport: FeedTransport,
    private acceptance?: FeedAcceptance,
  ) {}
  getSnapshot = () => this.snapshot;
  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };
  private publish(state: FeedState["state"], message?: string) {
    this.snapshot = {
      state,
      message,
      accepted: this.accepted,
    };
    this.listeners.forEach((listener) => listener());
  }
  connect(owner: string | undefined) {
    this.acceptance?.connect(owner);
    const generation = ++this.generation;
    if (this.current?.timer) clearTimeout(this.current.timer);
    if (this.current && !this.current.stopping) {
      this.current.stopping = this.transport.stop(this.current.id);
      void this.current.stopping.catch(() => {});
    }
    this.serial = this.serial
      .then(async () => {
        if (this.current) {
          const old = this.current;
          await old.stopping?.catch(() => {
            /* A lost cleanup reply is reconciled by the idempotent stop below. */
          });
          await this.transport.stop(old.id);
          this.current = undefined;
        }
        if (generation !== this.generation) return;
        if (!owner) {
          this.publish("idle");
          return;
        }
        if (this.accepted?.owner !== owner) {
          this.accepted = undefined;
        }
        const session: NonNullable<ObservationFeed["current"]> = {
          id: crypto.randomUUID(),
        };
        this.current = session;
        let previous = 0;
        const live = () =>
          generation === this.generation && this.current === session;
        const fail = (error: unknown) => {
          if (!live()) return;
          if (session.timer) clearTimeout(session.timer);
          this.publish(
            "stopped",
            error instanceof Error
              ? error.message
              : "Observation delivery stopped. Refresh connection explicitly.",
          );
          // Invalidate callbacks immediately; physical stop still drains through serial cleanup.
          this.generation += 1;
          session.stopping ??= this.transport.stop(session.id);
          void session.stopping.catch(() => {
            /* The visible stopped state retains uncertainty. */
          });
        };
        const arm = () => {
          if (session.timer) clearTimeout(session.timer);
          session.timer = setTimeout(
            () =>
              fail(
                new Error(
                  "Observation delivery stalled. Refresh connection explicitly.",
                ),
              ),
            15_000,
          );
        };
        this.publish("starting");
        arm();
        await this.transport.subscribe(
          owner,
          session.id,
          this.accepted,
          (value) => {
            if (!live()) return;
            try {
              if (!validateFeedEvent(value))
                throw new Error("Invalid observation delivery");
              if (
                value.kind === "stopped" &&
                value.subscription_id === session.id &&
                text(value.message, 4096)
              )
                throw new Error(value.message);
              if (value.kind !== "batch")
                throw new Error(
                  "Unexpected or overlapping observation delivery",
                );
              const batch = decodeFeedBatch(
                value.batch,
                owner,
                session.id,
                previous,
                this.accepted,
              );
              this.acceptance?.accept(batch);
              this.accepted = batch.cursor;
              previous = batch.batch_id;
              this.publish("live");
              arm();
              void this.transport
                .acknowledge({
                  subscription_id: session.id,
                  batch_id: batch.batch_id,
                  cursor: batch.cursor,
                })
                .catch(fail);
            } catch (error) {
              fail(error);
            }
          },
        );
        // Setup may resolve after cleanup. The serialized successor owns stop/join.
        if (!live()) await this.transport.stop(session.id);
      })
      .catch((error: unknown) => {
        if (generation !== this.generation) return;
        if (this.current?.timer) clearTimeout(this.current.timer);
        if (this.current) {
          this.current.stopping ??= this.transport.stop(this.current.id);
          void this.current.stopping.catch(() => {});
        }
        this.publish(
          "stopped",
          error instanceof Error
            ? error.message
            : "Observation subscription failed. Refresh connection explicitly.",
        );
      });
  }
}
