import {
  assign,
  createActor,
  fromPromise,
  setup,
  type ActorOptions,
} from "xstate";
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
  state: "idle" | "starting" | "live" | "stopping" | "stopped";
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
type Session = { id: string; owner: string; setup: Promise<void> };
type FeedContext = {
  desired?: string;
  session?: Session;
  accepted?: FeedCursor;
  previous: number;
  message?: string;
};
type FeedEvent =
  | { type: "configure"; owner?: string }
  | { type: "delivery"; id: string; value: unknown }
  | { type: "failed"; id: string; error: unknown }
  | { type: "acknowledge"; batch: FeedBatch };
const message = (error: unknown) =>
  error instanceof Error
    ? error.message
    : "Observation delivery stopped. Refresh connection explicitly.";

function feedMachine(transport: FeedTransport, acceptance?: FeedAcceptance) {
  return setup({
    types: { context: {} as FeedContext, events: {} as FeedEvent },
    actors: {
      cleanup: fromPromise(
        async ({ input }: { input: Session | undefined }) => {
          if (!input) return;
          // Stop immediately, then await registration and repeat the exact stop/join.
          // A lost first reply is harmless only after native reconciliation succeeds.
          const early = transport.stop(input.id);
          await Promise.allSettled([early, input.setup]);
          await transport.stop(input.id);
        },
      ),
    },
    guards: {
      desired: ({ context }) => !!context.desired,
      retained: ({ context }) => !!context.session,
      failed: ({ context }) => !!context.message,
      currentFailure: ({ context, event }) =>
        event.type === "failed" && event.id === context.session?.id,
      currentDelivery: ({ context, event }) =>
        event.type === "delivery" && event.id === context.session?.id,
    },
    actions: {
      configure: assign(({ event }) =>
        event.type === "configure" ? { desired: event.owner } : {},
      ),
      begin: assign(({ context, self }) => {
        const owner = context.desired;
        if (!owner) return {};
        const id = crypto.randomUUID();
        const accepted =
          context.accepted?.owner === owner ? context.accepted : undefined;
        // Defer setup until the session handle is installed in actor context.
        const registration = Promise.resolve().then(() =>
          transport.subscribe(owner, id, accepted, (value) =>
            self.send({ type: "delivery", id, value }),
          ),
        );
        void registration.catch((error: unknown) =>
          self.send({ type: "failed", id, error }),
        );
        return {
          session: { id, owner, setup: registration },
          previous: 0,
          accepted,
          message: undefined,
        };
      }),
      accept: assign(({ context, event, self }) => {
        if (event.type !== "delivery" || !context.session) return {};
        const { id, owner } = context.session;
        try {
          const value = event.value;
          if (!validateFeedEvent(value))
            throw new Error("Invalid observation delivery");
          if (
            value.kind === "stopped" &&
            value.subscription_id === id &&
            text(value.message, 4096)
          )
            throw new Error(value.message);
          if (value.kind !== "batch")
            throw new Error("Unexpected observation delivery");
          const batch = decodeFeedBatch(
            value.batch,
            owner,
            id,
            context.previous,
            context.accepted,
          );
          acceptance?.accept(batch);
          // Ack is sent as another actor event after this acceptance transition commits.
          self.send({ type: "acknowledge", batch });
          return { accepted: batch.cursor, previous: batch.batch_id };
        } catch (error) {
          self.send({ type: "failed", id, error });
          return {};
        }
      }),
      fail: assign(({ event }) =>
        event.type === "failed"
          ? { desired: undefined, message: message(event.error) }
          : {},
      ),
      stalled: assign({
        desired: undefined,
        message: "Observation delivery stalled. Refresh connection explicitly.",
      }),
      clearSession: assign({ session: undefined }),
      acknowledge: ({ event, self }) => {
        if (event.type !== "acknowledge") return;
        const { subscription_id, batch_id, cursor } = event.batch;
        const acknowledgement = { subscription_id, batch_id, cursor };
        void transport.acknowledge(acknowledgement).catch((error: unknown) =>
          self.send({
            type: "failed",
            id: acknowledgement.subscription_id,
            error,
          }),
        );
      },
    },
  }).createMachine({
    id: "feed",
    initial: "idle",
    context: { previous: 0 },
    states: {
      idle: { on: { configure: { target: "decide", actions: "configure" } } },
      decide: {
        always: [
          { guard: "retained", target: "stopping" },
          { guard: "desired", target: "starting" },
          { guard: "failed", target: "stopped" },
          { target: "idle" },
        ],
      },
      starting: {
        entry: "begin",
        after: { 15000: { target: "stopping", actions: "stalled" } },
        on: {
          configure: { target: "stopping", actions: "configure" },
          delivery: {
            guard: "currentDelivery",
            target: "live",
            actions: "accept",
          },
          failed: {
            guard: "currentFailure",
            target: "stopping",
            actions: "fail",
          },
        },
      },
      live: {
        after: { 15000: { target: "stopping", actions: "stalled" } },
        on: {
          configure: { target: "stopping", actions: "configure" },
          delivery: {
            guard: "currentDelivery",
            target: "live",
            reenter: true,
            actions: "accept",
          },
          acknowledge: { actions: "acknowledge" },
          failed: {
            guard: "currentFailure",
            target: "stopping",
            actions: "fail",
          },
        },
      },
      stopping: {
        invoke: {
          src: "cleanup",
          input: ({ context }) => context.session,
          onDone: { target: "decide", actions: "clearSession" },
          onError: {
            target: "stopped",
            actions: assign(({ event }) => ({ message: message(event.error) })),
          },
        },
        on: { configure: { actions: "configure" } },
      },
      stopped: {
        on: { configure: { target: "decide", actions: "configure" } },
      },
    },
  });
}

// Typed lifecycle map: runtime transitions above are the single lifecycle authority.
export const feedTransitions = {
  idle: ["decide"],
  decide: ["stopping", "starting", "stopped", "idle"],
  starting: ["live", "stopping"],
  live: ["live", "stopping"],
  stopping: ["decide", "stopped"],
  stopped: ["decide"],
} satisfies Record<
  ReturnType<ReturnType<typeof feedMachine>["resolveState"]>["value"] & string,
  readonly (ReturnType<
    ReturnType<typeof feedMachine>["resolveState"]
  >["value"] &
    string)[]
>;

export class ObservationFeed {
  readonly actor;
  constructor(
    transport: FeedTransport,
    private acceptance?: FeedAcceptance,
    options?: Pick<ActorOptions<ReturnType<typeof feedMachine>>, "clock">,
  ) {
    this.actor = createActor(
      feedMachine(transport, acceptance),
      options,
    ).start();
  }
  getSnapshot = (): FeedState => {
    const snapshot = this.actor.getSnapshot();
    return {
      state: snapshot.value === "decide" ? "stopping" : snapshot.value,
      message: snapshot.context.message,
      accepted: snapshot.context.accepted,
    };
  };
  connect(owner: string | undefined) {
    this.acceptance?.connect(owner);
    this.actor.send({ type: "configure", owner });
  }
}
