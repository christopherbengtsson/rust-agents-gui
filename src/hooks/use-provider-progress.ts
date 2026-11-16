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
  incarnation: string | undefined,
  recover = false,
) {
  const feature = app.providers;
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
  const action = useSyncExternalStore(feature.subscribeActions, () =>
    shownIncarnation &&
    targetId &&
    feature.action(shownIncarnation, targetId)?.kind === "login"
      ? feature.action(shownIncarnation, targetId)
      : undefined,
  );
  const state = useSyncExternalStore(feature.subscribe, () =>
    shownIncarnation && targetId
      ? feature.snapshot(shownIncarnation, {
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
      shownIncarnation && retain
        ? feature.observe(shownIncarnation, { operation: "target" })
        : undefined,
    [feature, shownIncarnation, retain],
  );
  useEffect(
    () =>
      retain && shownIncarnation && targetId
        ? feature.observe(shownIncarnation, {
            operation: "observe",
            target_id: targetId,
          })
        : undefined,
    [feature, shownIncarnation, targetId, retain],
  );
  return {
    observation,
    stale: !incarnation || !!state?.error || action?.state === "uncertain",
    active:
      loginActive(observation) ||
      action?.state === "pending" ||
      action?.state === "uncertain",
  };
}

export function useProviderLogin(
  app: AppObservations,
  incarnation: string | undefined,
  observation: ProviderObservation | undefined,
) {
  const target = observation?.target.id;
  const action = useSyncExternalStore(app.providers.subscribeActions, () =>
    incarnation &&
    target &&
    app.providers.action(incarnation, target)?.kind === "login"
      ? app.providers.action(incarnation, target)
      : undefined,
  );
  const attempt = observation?.login?.id;
  return {
    pending: action?.state === "pending",
    uncertain: action?.state === "uncertain",
    message: action?.message,
    start: () => {
      if (incarnation && observation)
        void app.providers.startBrowser(incarnation, observation);
    },
    cancel: () => {
      if (incarnation && target && attempt)
        void app.providers.loginAction(
          incarnation,
          target,
          attempt,
          "cancel_login",
        );
    },
    reopen: () => {
      if (incarnation && target && attempt)
        void app.providers.loginAction(
          incarnation,
          target,
          attempt,
          "reopen_browser",
        );
    },
    reconcile: () => {
      if (incarnation && target && (action?.attempt ?? attempt))
        void app.providers.loginAction(
          incarnation,
          target,
          action?.attempt ?? attempt ?? "",
          "reconcile_login",
        );
    },
  };
}
