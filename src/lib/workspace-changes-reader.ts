import { skipToken, useQuery } from "@tanstack/react-query";
import { useSelector, shallowEqual } from "@xstate/react";
import { useStore } from "zustand";
import {
  assign,
  createActor,
  fromPromise,
  setup,
  type ActorOptions,
} from "xstate";
import type { Target } from "@/generated/contracts/types";
import { FeatureFreshness } from "./feature-freshness";
import { createObservationQueryClient } from "./feature-query-cache";
import {
  ObservationCoordinator,
  ObsoleteRead,
  type ObservationScope,
} from "./observation-coordinator";
import { workspaceRequest } from "./workspaces";
import {
  pendingInspection,
  type ChangedFile,
  type ChangesObservation,
  type PreviewObservation,
  type InspectionRequest,
  type InspectionReply,
} from "./changes";
import { repositoryError } from "./repositories";
import {
  createPresentationStore,
  type InspectionSelection,
} from "./presentation-store";

export type ChangesView = {
  observation?: ChangesObservation;
  preview?: PreviewObservation;
  selected?: InspectionSelection;
  error?: string;
  loading: boolean;
};
type Context = {
  owner: string | undefined;
  workspaceId: string;
  active: boolean;
  visible: boolean;
};
type Page = { comparison_id: string | null; after: string | null };
type Key = readonly [
  "inspection",
  string,
  (
    | Omit<Extract<InspectionRequest, { operation: "changes" }>, "refresh">
    | Omit<Extract<InspectionRequest, { operation: "change_preview" }>, "load">
  ),
];
type Cached = { reply: InspectionReply; generation: number };
class InspectionFailure extends Error {
  constructor(
    readonly generation: number,
    cause: unknown,
  ) {
    super(repositoryError(cause).message, { cause });
  }
}
type CachedState = { data?: Cached; error?: Error | null };
function visibleError(error?: Error | null) {
  return error instanceof InspectionFailure &&
    error.cause instanceof ObsoleteRead
    ? undefined
    : error;
}
function projectView(
  summary: CachedState | undefined,
  preview: CachedState | undefined,
  previous: CachedState | undefined,
  selected: ChangesView["selected"],
  loading: boolean,
  restoring = false,
): ChangesView {
  const observation =
    summary?.data?.reply.kind === "changes"
      ? summary.data.reply.observation
      : undefined;
  const retained =
    previous?.data?.reply.kind === "change_preview"
      ? previous.data.reply.observation
      : undefined;
  const latest =
    preview?.data?.reply.kind === "change_preview"
      ? preview.data.reply.observation
      : undefined;
  return {
    observation,
    selected:
      selected &&
      (!restoring || selected.comparison === observation?.page?.comparison_id)
        ? selected.comparison === observation?.page?.comparison_id
          ? selected
          : { ...selected, position: undefined }
        : undefined,
    loading,
    preview: latest
      ? { ...latest, preview: latest.preview ?? retained?.preview ?? null }
      : retained,
    error:
      visibleError(summary?.error)?.message ??
      visibleError(preview?.error)?.message,
  };
}
type Flight = {
  token: object;
  scope: ObservationScope;
  promise: Promise<void>;
};
type Coordination = {
  restoring?: boolean;
  desired?: Context;
  page: Page;
  queued?: "observe" | "refresh";
  flight?: Flight;
  summary?: Key;
  preview?: Key;
  previousPreview?: Key;
};
type Event =
  | { type: "configure"; context: Context }
  | { type: "release" }
  | { type: "select" }
  | { type: "request"; refresh: boolean }
  | { type: "page"; page: Page }
  | { type: "hint" }
  | { type: "summary" | "preview"; token: object; key: Key };
const firstPage = (): Page => ({ comparison_id: null, after: null });
const eligible = (context: Coordination, freshness: FeatureFreshness) =>
  !!context.desired?.visible &&
  !!context.desired.owner &&
  freshness.ready(context.desired.owner);
const keyFor = (owner: string, request: InspectionRequest): Key =>
  request.operation === "changes"
    ? [
        "inspection",
        owner,
        {
          operation: "changes",
          workspace_id: request.workspace_id,
          comparison_id: request.comparison_id,
          after: request.after,
        },
      ]
    : [
        "inspection",
        owner,
        { operation: "change_preview", target: request.target },
      ];

function inspectionMachine(
  reader: WorkspaceChangesReader,
  freshness: FeatureFreshness,
) {
  return setup({
    types: { context: {} as Coordination, events: {} as Event },
    actors: {
      settle: fromPromise(async ({ input }: { input: Flight | undefined }) => {
        await input?.promise;
      }),
    },
    guards: {
      dirty: ({ context }) => reader.dirty(context),
      unavailable: ({ context }) => !eligible(context, freshness),
      requested: ({ context }) =>
        eligible(context, freshness) && !!context.queued,
      cadence: ({ context }) =>
        eligible(context, freshness) && !reader.paused(context),
      current: ({ context, event }) =>
        (event.type === "summary" || event.type === "preview") &&
        context.flight?.token === event.token,
    },
    actions: {
      configure: assign(({ context, event }) => {
        if (event.type !== "configure") return {};
        const previous = context.desired;
        const changed =
          previous?.workspaceId !== event.context.workspaceId ||
          (!!event.context.owner && previous.owner !== event.context.owner);
        return {
          desired: event.context,
          restoring:
            changed || !event.context.visible ? true : context.restoring,
          queued: event.context.visible
            ? changed
              ? "observe"
              : (context.queued ??
                (reader.failed(context) &&
                !reader.dirty({ ...context, desired: event.context })
                  ? undefined
                  : "observe"))
            : undefined,
          ...(changed
            ? {
                page: firstPage(),
                summary: undefined,
                preview: undefined,
                previousPreview: undefined,
              }
            : {}),
        };
      }),
      release: assign(({ context }) => ({
        desired: context.desired
          ? { ...context.desired, visible: false }
          : undefined,
        queued: undefined,
        restoring: true,
      })),
      request: assign(({ context, event }) => ({
        queued:
          event.type === "request" && event.refresh
            ? "refresh"
            : (context.queued ?? "observe"),
      })),
      page: assign(({ context, event }) =>
        event.type === "page"
          ? { page: event.page, queued: context.queued ?? "observe" }
          : {},
      ),
      select: assign(({ context }) => ({
        restoring: false,
        preview: undefined,
        previousPreview: undefined,
        queued: context.queued ?? "observe",
      })),
      invalidate: ({ context }) => context.flight?.scope.dispose(),
      hint: assign(({ context }) =>
        reader.dirty(context) ? { queued: context.queued ?? "observe" } : {},
      ),
      refresh: assign({ queued: "refresh" }),
      begin: assign(({ context, self }) => {
        const refresh = context.queued === "refresh";
        const page = refresh ? firstPage() : context.page;
        const scope = reader.reads.scope();
        const token = {};
        const current = () =>
          self.getSnapshot().matches("reading") &&
          self.getSnapshot().context.flight?.token === token;
        const promise = Promise.resolve().then(() =>
          reader.sequence(
            { ...context, page },
            scope,
            refresh,
            current,
            (type, key) => self.send({ type, token, key }),
          ),
        );
        return { page, queued: undefined, flight: { token, scope, promise } };
      }),
      settled: assign(({ context }) => {
        context.flight?.scope.dispose();
        return {
          flight: undefined,
          queued:
            context.queued ?? (reader.dirty(context) ? "observe" : undefined),
        };
      }),
      summary: assign(({ event }) =>
        event.type === "summary" ? { summary: event.key } : {},
      ),
      preview: assign(({ context, event }) =>
        event.type === "preview"
          ? {
              restoring: false,
              preview: event.key,
              previousPreview: context.preview ?? context.previousPreview,
            }
          : {},
      ),
    },
  }).createMachine({
    initial: "idle",
    context: { page: firstPage() },
    states: {
      idle: {
        always: [
          { guard: "requested", target: "reading" },
          { guard: "cadence", target: "waiting" },
        ],
        on: {
          configure: { target: "idle", reenter: true, actions: "configure" },
          request: { target: "idle", reenter: true, actions: "request" },
          select: { target: "idle", reenter: true, actions: "select" },
          page: { target: "idle", reenter: true, actions: "page" },
          hint: { target: "idle", reenter: true, actions: "hint" },
          release: { actions: "release" },
        },
      },
      waiting: {
        after: {
          2000: [
            { guard: "unavailable", target: "idle" },
            { target: "reading", actions: "refresh" },
          ],
        },
        on: {
          configure: { target: "idle", actions: "configure" },
          request: { target: "idle", actions: "request" },
          select: { target: "idle", actions: "select" },
          page: { target: "idle", actions: "page" },
          hint: [
            { guard: "unavailable", target: "idle" },
            { guard: "dirty", target: "idle", actions: "hint" },
          ],
          release: { target: "idle", actions: "release" },
        },
      },
      reading: {
        entry: "begin",
        invoke: {
          src: "settle",
          input: ({ context }) => context.flight,
          onDone: { target: "idle", actions: "settled" },
        },
        on: {
          configure: {
            target: "draining",
            actions: ["invalidate", "configure"],
          },
          request: { target: "draining", actions: ["invalidate", "request"] },
          select: { target: "draining", actions: ["invalidate", "select"] },
          page: { target: "draining", actions: ["invalidate", "page"] },
          release: { target: "draining", actions: ["invalidate", "release"] },
          summary: { guard: "current", actions: "summary" },
          preview: { guard: "current", actions: "preview" },
        },
      },
      draining: {
        invoke: {
          src: "settle",
          input: ({ context }) => context.flight,
          onDone: { target: "idle", actions: "settled" },
        },
        on: {
          configure: { actions: "configure" },
          request: { actions: "request" },
          select: { actions: "select" },
          page: { actions: "page" },
          release: { actions: "release" },
        },
      },
    },
  });
}
export const inspectionTransitions = {
  idle: ["reading", "waiting", "idle"],
  waiting: ["reading", "idle"],
  reading: ["draining", "idle"],
  draining: ["idle"],
} satisfies Record<
  ReturnType<ReturnType<typeof inspectionMachine>["resolveState"]>["value"] &
    string,
  readonly (ReturnType<
    ReturnType<typeof inspectionMachine>["resolveState"]
  >["value"] &
    string)[]
>;

// One feature owner: actor coordinates; Query owns bodies; Zustand owns selection.
export class WorkspaceChangesReader {
  readonly actor;
  private readonly unsubscribeFreshness;
  constructor(
    readonly reads = new ObservationCoordinator(),
    readonly client = createObservationQueryClient(),
    private freshness = new FeatureFreshness(),
    readonly presentation = createPresentationStore(),
    options?: Pick<ActorOptions<ReturnType<typeof inspectionMachine>>, "clock">,
  ) {
    this.actor = createActor(
      inspectionMachine(this, freshness),
      options,
    ).start();
    this.unsubscribeFreshness = freshness.subscribe(() => {
      // Invalidations are recorded synchronously; scheduling follows feed ack.
      queueMicrotask(() => this.actor.send({ type: "hint" }));
    });
  }
  private cached(key?: Key) {
    return key ? this.client.getQueryState<Cached>(key) : undefined;
  }
  private selected(context: Coordination) {
    return context.desired
      ? this.presentation.getState().inspection[context.desired.workspaceId]
      : undefined;
  }
  getView = (): ChangesView => {
    const snapshot = this.actor.getSnapshot();
    const context = snapshot.context;
    const summary = this.cached(context.summary);
    const preview = this.cached(context.preview);
    const previous = this.cached(context.previousPreview);
    return projectView(
      summary,
      preview,
      previous,
      this.selected(context),
      snapshot.matches("reading") || snapshot.matches("draining"),
      context.restoring,
    );
  };

  configure(context: Context) {
    this.actor.send({ type: "configure", context });
  }
  invalidate(target: Target) {
    if (target.kind === "inspection")
      this.freshness.mark(`inspection:${target.workspace_id}`);
  }
  release() {
    this.actor.send({ type: "release" });
  }
  async dispose() {
    this.release();
    await this.actor.getSnapshot().context.flight?.promise;
    this.unsubscribeFreshness();
    this.actor.stop();
  }
  select = (file: ChangedFile) => {
    const context = this.actor.getSnapshot().context;
    const workspace = context.desired?.workspaceId;
    if (!workspace) return;
    const summary = this.cached(context.summary)?.data?.reply;
    const comparison =
      summary?.kind === "changes"
        ? summary.observation.page?.comparison_id
        : undefined;
    this.presentation
      .getState()
      .rememberFile(workspace, { id: file.id, path: file.path, comparison });
    this.actor.send({ type: "select" });
  };
  refresh = () => {
    this.actor.send({ type: "request", refresh: true });
    return Promise.resolve();
  };
  first = () => {
    this.actor.send({ type: "page", page: firstPage() });
    return Promise.resolve();
  };
  next = () => {
    const page = this.getView().observation?.page;
    if (page?.next)
      this.actor.send({
        type: "page",
        page: { comparison_id: page.comparison_id, after: page.next },
      });
    return Promise.resolve();
  };
  private version(context: Coordination) {
    return this.freshness.version([
      `inspection:${context.desired?.workspaceId}`,
    ]);
  }
  dirty(context: Coordination) {
    if (!eligible(context, this.freshness)) return false;
    const version = this.version(context);
    const dirtyKey = (key?: Key) => {
      const state = this.cached(key);
      const captured =
        state?.error instanceof InspectionFailure
          ? state.error.generation
          : state?.data?.generation;
      return captured === undefined || captured < version;
    };
    const summaryState = this.cached(context.summary);
    const summary = summaryState?.data?.reply;
    // A pending/failed summary cannot supply a current preview target. Its
    // retained preview may still be dirty; wait for summary progress hints
    // rather than immediately repeating the same pending summary read.
    const previewEligible =
      !summaryState?.error &&
      summary?.kind === "changes" &&
      summary.observation.phase === "ready";
    return (
      dirtyKey(context.summary) ||
      (previewEligible &&
        !!context.desired?.active &&
        !!this.selected(context) &&
        !!context.preview &&
        dirtyKey(context.preview))
    );
  }
  failed(context: Coordination) {
    const summary = this.cached(context.summary);
    const preview = this.cached(context.preview);
    const observation =
      summary?.data?.reply.kind === "changes"
        ? summary.data.reply.observation
        : undefined;
    const selectedPreview =
      preview?.data?.reply.kind === "change_preview"
        ? preview.data.reply.observation
        : undefined;
    return (
      !!visibleError(summary?.error) ||
      observation?.phase === "failed" ||
      (!!context.desired?.active &&
        (selectedPreview?.phase === "failed" || !!visibleError(preview?.error)))
    );
  }
  paused(context: Coordination) {
    const summary = this.cached(context.summary)?.data?.reply;
    const preview = this.cached(context.preview)?.data?.reply;
    return (
      this.failed(context) ||
      (summary?.kind === "changes" &&
        pendingInspection(summary.observation.phase)) ||
      (!!context.desired?.active &&
        preview?.kind === "change_preview" &&
        pendingInspection(preview.observation.phase))
    );
  }
  private async read(
    owner: string,
    request: InspectionRequest,
    scope: ObservationScope,
    context: Coordination,
  ) {
    const key = keyFor(owner, request);
    const rejectedGeneration = this.version(context);
    return this.client.query<Cached>({
      queryKey: key,
      staleTime: 0,
      queryFn: async () => {
        try {
          return await scope.read(["workspace", owner, request], async () => {
            const generation = this.version(context);
            try {
              const reply = await workspaceRequest(owner, request);
              if (reply.kind !== "changes" && reply.kind !== "change_preview")
                throw new Error("Unexpected inspection reply");
              return { reply, generation };
            } catch (error) {
              throw new InspectionFailure(generation, error);
            }
          });
        } catch (error) {
          throw error instanceof InspectionFailure
            ? error
            : new InspectionFailure(rejectedGeneration, error);
        }
      },
    });
  }
  async sequence(
    context: Coordination,
    scope: ObservationScope,
    refresh: boolean,
    current: () => boolean,
    publish: (type: "summary" | "preview", key: Key) => void,
  ) {
    const desired = context.desired;
    if (!desired?.owner || !current() || !this.freshness.ready(desired.owner))
      return;
    const { owner, workspaceId, active } = desired;
    const selected = this.selected(context);
    const request: InspectionRequest = {
      operation: "changes",
      workspace_id: workspaceId,
      refresh,
      ...context.page,
    };
    const summaryKey = keyFor(owner, request);
    publish("summary", summaryKey);
    try {
      const result = await this.read(owner, request, scope, context);
      if (!current() || result.reply.kind !== "changes") return;
      const summary = result.reply.observation;
      if (summary.phase !== "ready" || !summary.page) return;
      if (
        context.restoring &&
        selected &&
        selected.comparison !== summary.page.comparison_id
      ) {
        // File IDs are comparison-scoped. Reusing one under a fresh comparison
        // cannot validate restoration, even when a display path still matches.
        if (
          this.selected(context)?.id === selected.id &&
          this.selected(context)?.comparison === selected.comparison
        )
          this.presentation.getState().rememberFile(workspaceId, undefined);
        return;
      }
      if (!active || !selected) return;
      // Native validation resolves the exact file even outside the current page.
      const target = {
        workspace_id: workspaceId,
        comparison_id: summary.page.comparison_id,
        file_id: selected.id,
      };
      const previewRequest: InspectionRequest = {
        operation: "change_preview",
        target,
        load:
          refresh ||
          !this.cached(
            keyFor(owner, { operation: "change_preview", target, load: false }),
          )?.data,
      };
      const previewKey = keyFor(owner, previewRequest);
      publish("preview", previewKey);
      await this.read(owner, previewRequest, scope, context);
      if (
        current() &&
        this.selected(context)?.id === selected.id &&
        this.selected(context)?.comparison === selected.comparison &&
        selected.comparison !== target.comparison_id
      )
        this.presentation.getState().rememberFile(workspaceId, {
          id: selected.id,
          path: selected.path,
          comparison: target.comparison_id,
        });
    } catch {
      // Query retains exact-target failures and last-good data; the actor pauses.
    }
  }
}

// Feature Hook owns library bindings and Query definitions; screens receive only
// the ChangesView and targeted feature callbacks from useWorkspaceChanges.
export function useInspectionView(reader: WorkspaceChangesReader): ChangesView {
  const reading = useSelector(
    reader.actor,
    (snapshot) => ({
      workspace: snapshot.context.desired?.workspaceId,
      summary: snapshot.context.summary,
      preview: snapshot.context.preview,
      previous: snapshot.context.previousPreview,
      loading: snapshot.matches("reading") || snapshot.matches("draining"),
      restoring: snapshot.context.restoring,
    }),
    shallowEqual,
  );
  const selected = useStore(reader.presentation, (state) =>
    reading.workspace ? state.inspection[reading.workspace] : undefined,
  );
  const summary = useQuery<Cached>(
    {
      queryKey: reading.summary ?? ["inspection", "inactive", "summary"],
      enabled: false,
      queryFn: skipToken,
    },
    reader.client,
  );
  const preview = useQuery<Cached>(
    {
      queryKey: reading.preview ?? ["inspection", "inactive", "preview"],
      enabled: false,
      queryFn: skipToken,
    },
    reader.client,
  );
  const previous = useQuery<Cached>(
    {
      queryKey: reading.previous ?? ["inspection", "inactive", "previous"],
      enabled: false,
      queryFn: skipToken,
    },
    reader.client,
  );
  return projectView(
    summary,
    preview,
    previous,
    selected,
    reading.loading,
    reading.restoring,
  );
}
