import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { loginActive } from "./use-provider-progress";
import type { AppObservations } from "@/lib/app-observations";

type SetupScope = { incarnation: string; targetId: string };

export function useProviderSetup(
  app: AppObservations,
  incarnation: string | undefined,
  updatesAvailable = true,
) {
  const feature = app.providers;
  // One entry intent per exact scope, distinct from transport reattachment.
  // These markers are presentation metadata, never provider/process evidence.
  const entryRequested = useRef<SetupScope | undefined>(undefined);
  const [attached, setAttached] = useState<SetupScope>();
  const shownIncarnation = incarnation ?? feature.retainedIncarnation();
  const targetState = useSyncExternalStore(feature.subscribe, () =>
    shownIncarnation
      ? feature.snapshot(shownIncarnation, { operation: "target" })
      : undefined,
  );
  const target =
    targetState?.data?.value.kind === "target"
      ? targetState.data.value.target
      : undefined;
  const targetId = target?.id;
  const entryAttached =
    !!attached &&
    attached.incarnation === shownIncarnation &&
    attached.targetId === targetId;
  const state = useSyncExternalStore(feature.subscribe, () =>
    shownIncarnation && target
      ? feature.snapshot(shownIncarnation, {
          operation: "observe",
          target_id: target.id,
        })
      : undefined,
  );
  const current = useSyncExternalStore(
    feature.subscribe,
    () =>
      !!shownIncarnation &&
      !!target &&
      feature.isCurrent(shownIncarnation, {
        operation: "observe",
        target_id: target.id,
      }),
  );
  const observation =
    state?.data?.value.kind === "observation"
      ? state.data.value.observation
      : undefined;
  useEffect(
    () =>
      incarnation
        ? feature.observe(incarnation, { operation: "target" })
        : undefined,
    [feature, incarnation],
  );
  useEffect(() => {
    if (!shownIncarnation || !targetId) return;
    let disposed = false;
    let release: (() => void) | undefined;
    // A disposed Strict Mode setup must neither consume the activation nor
    // cancel its replacement's check before it has attached its interest.
    queueMicrotask(() => {
      if (disposed) return;
      const scope = { incarnation: shownIncarnation, targetId };
      const request = { operation: "observe", target_id: targetId } as const;
      const requested = entryRequested.current;
      if (
        requested?.incarnation !== shownIncarnation ||
        requested.targetId !== targetId
      ) {
        entryRequested.current = scope;
        // Mark verification before attaching: the first read uses refresh:true
        // instead of an initial probe followed by a competing refresh request.
        void feature.refresh(shownIncarnation, true, [request]);
      }
      // Transport detachment is not view release. Keeping this incarnation-bound
      // interest preserves an undispatched force flag while the feed baseline
      // is unavailable. Queue admission waits for that exact incarnation's baseline;
      // admitted work retains physical identity and is never replayed.
      release = feature.observe(shownIncarnation, request);
      setAttached(scope);
    });
    return () => {
      disposed = true;
      release?.();
    };
  }, [feature, shownIncarnation, targetId]);
  const error = targetState?.error ?? state?.error;
  const stale =
    loginActive(observation) ||
    !incarnation ||
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
    connected: !!incarnation,
    updatesAvailable,
    pending:
      !!incarnation &&
      updatesAvailable &&
      (targetState?.fetchStatus === "fetching" ||
        state?.fetchStatus === "fetching" ||
        (!error &&
          (!target || !entryAttached || observation?.phase === "checking"))),
    refresh: () =>
      incarnation
        ? feature.refresh(
            incarnation,
            true,
            target
              ? [{ operation: "observe", target_id: target.id }]
              : [{ operation: "target" }],
          )
        : Promise.resolve(),
  };
}
