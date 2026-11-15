import { useEffect, useSyncExternalStore } from "react";
import type { AppObservations } from "@/lib/app-observations";
import type { ProviderObservation } from "@/generated/contracts/types";

export function loginActive(observation: ProviderObservation | undefined) {
  return (
    !!observation?.login &&
    !["completed", "failed", "cancelled"].includes(observation.login.phase)
  );
}

/** App-lifetime status interest. Reads never admit a runtime probe or login. */
export function useProviderProgress(
  app: AppObservations,
  owner: string | undefined,
  recover = false,
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
  const targetId = target?.id;
  const action = useSyncExternalStore(feature.subscribeActions, () =>
    shownOwner && targetId ? feature.action(shownOwner, targetId) : undefined,
  );
  const state = useSyncExternalStore(feature.subscribe, () =>
    shownOwner && targetId
      ? feature.snapshot(shownOwner, {
          operation: "observe",
          target_id: targetId,
        })
      : undefined,
  );
  const observation =
    state?.data?.value.kind === "observation"
      ? state.data.value.observation
      : undefined;
  const retain =
    recover ||
    loginActive(observation) ||
    action?.state === "pending" ||
    action?.state === "uncertain";
  useEffect(
    () =>
      shownOwner && retain
        ? feature.observe(shownOwner, { operation: "target" })
        : undefined,
    [feature, shownOwner, retain],
  );
  useEffect(
    () =>
      retain && shownOwner && targetId
        ? feature.observe(shownOwner, {
            operation: "observe",
            target_id: targetId,
          })
        : undefined,
    [feature, shownOwner, targetId, retain],
  );
  return {
    observation,
    stale: !owner || !!state?.error || action?.state === "uncertain",
    active:
      loginActive(observation) ||
      action?.state === "pending" ||
      action?.state === "uncertain",
  };
}

export function useProviderLogin(
  app: AppObservations,
  owner: string | undefined,
  observation: ProviderObservation | undefined,
) {
  const target = observation?.target.id;
  const action = useSyncExternalStore(app.providers.subscribeActions, () =>
    owner && target ? app.providers.action(owner, target) : undefined,
  );
  const attempt = observation?.login?.id;
  return {
    pending: action?.state === "pending",
    uncertain: action?.state === "uncertain",
    message: action?.message,
    start: () => {
      if (owner && observation)
        void app.providers.startBrowser(owner, observation);
    },
    cancel: () => {
      if (owner && target && attempt)
        void app.providers.loginAction(owner, target, attempt, "cancel_login");
    },
    reopen: () => {
      if (owner && target && attempt)
        void app.providers.loginAction(
          owner,
          target,
          attempt,
          "reopen_browser",
        );
    },
    reconcile: () => {
      if (owner && target && (action?.attempt ?? attempt))
        void app.providers.loginAction(
          owner,
          target,
          action?.attempt ?? attempt ?? "",
          "reconcile_login",
        );
    },
  };
}
