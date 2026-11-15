import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { loginActive } from "./use-provider-progress";
import type { AppObservations } from "@/lib/app-observations";

type SetupScope = { owner: string; targetId: string };

export function useProviderSetup(
  app: AppObservations,
  owner: string | undefined,
  updatesAvailable = true,
) {
  const feature = app.providers;
  // One entry intent per exact scope, distinct from transport reattachment.
  // These markers are presentation metadata, never provider/process evidence.
  const entryRequested = useRef<SetupScope | undefined>(undefined);
  const [attached, setAttached] = useState<SetupScope>();
  const shownOwner = owner ?? feature.retainedOwner();
  const targetState = useSyncExternalStore(feature.subscribe, () =>
    shownOwner
      ? feature.snapshot(shownOwner, { operation: "target" })
      : undefined,
  );
  const target =
    targetState?.data?.value.kind === "target"
      ? targetState.data.value.target
      : undefined;
  const targetId = target?.id;
  const entryAttached =
    !!attached &&
    attached.owner === shownOwner &&
    attached.targetId === targetId;
  const state = useSyncExternalStore(feature.subscribe, () =>
    shownOwner && target
      ? feature.snapshot(shownOwner, {
          operation: "observe",
          target_id: target.id,
        })
      : undefined,
  );
  const current = useSyncExternalStore(
    feature.subscribe,
    () =>
      !!shownOwner &&
      !!target &&
      feature.isCurrent(shownOwner, {
        operation: "observe",
        target_id: target.id,
      }),
  );
  const observation =
    state?.data?.value.kind === "observation"
      ? state.data.value.observation
      : undefined;
  useEffect(
    () => (owner ? feature.observe(owner, { operation: "target" }) : undefined),
    [feature, owner],
  );
  useEffect(() => {
    if (!shownOwner || !targetId) return;
    let disposed = false;
    let release: (() => void) | undefined;
    // A disposed Strict Mode setup must neither consume the activation nor
    // cancel its replacement's check before it has attached its interest.
    queueMicrotask(() => {
      if (disposed) return;
      const scope = { owner: shownOwner, targetId };
      const request = { operation: "observe", target_id: targetId } as const;
      const requested = entryRequested.current;
      if (requested?.owner !== shownOwner || requested.targetId !== targetId) {
        entryRequested.current = scope;
        // Mark verification before attaching: the first read uses refresh:true
        // instead of an initial probe followed by a competing refresh request.
        void feature.refresh(shownOwner, true, [request]);
      }
      // Transport detachment is not view release. Keeping this owner-bound
      // interest preserves an undispatched force flag while the feed baseline
      // is unavailable. Queue admission waits for that exact owner's baseline;
      // admitted work retains physical identity and is never replayed.
      release = feature.observe(shownOwner, request);
      setAttached(scope);
    });
    return () => {
      disposed = true;
      release?.();
    };
  }, [feature, shownOwner, targetId]);
  const error = targetState?.error ?? state?.error;
  const stale =
    loginActive(observation) ||
    !owner ||
    !updatesAvailable ||
    !entryAttached ||
    !!error ||
    (!!observation &&
      (observation.phase !== "observed" ||
        (!!observation.problem &&
          observation.problem !== "authentication_unavailable"))) ||
    (!!target && !current);
  return {
    target: observation?.target ?? target,
    observation,
    error,
    stale,
    connected: !!owner,
    updatesAvailable,
    pending:
      !!owner &&
      updatesAvailable &&
      (targetState?.fetchStatus === "fetching" ||
        state?.fetchStatus === "fetching" ||
        (!error &&
          (!target || !entryAttached || observation?.phase === "checking"))),
    refresh: () =>
      owner
        ? feature.refresh(
            owner,
            true,
            target
              ? [{ operation: "observe", target_id: target.id }]
              : [{ operation: "target" }],
          )
        : Promise.resolve(),
  };
}
