import { useEffect, useSyncExternalStore } from "react";
import type { AppObservations } from "@/lib/app-observations";

export function useProviderSetup(
  app: AppObservations,
  owner: string | undefined,
) {
  const feature = app.providers;
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
  useEffect(
    () =>
      owner && target
        ? feature.observe(owner, { operation: "observe", target_id: target.id })
        : undefined,
    [feature, owner, target],
  );
  const error = targetState?.error ?? state?.error;
  const stale =
    !owner ||
    !!error ||
    (!!observation &&
      (observation.phase !== "observed" ||
        (!!observation.problem &&
          observation.problem !== "authentication_unavailable"))) ||
    (!!target && !current);
  return {
    target,
    observation,
    error,
    stale,
    connected: !!owner,
    pending:
      state?.fetchStatus === "fetching" || observation?.phase === "checking",
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
