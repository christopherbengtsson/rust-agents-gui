import {
  useCallback,
  useEffect,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import type { Navigation } from "@/lib/navigation";
import type { AppObservations } from "@/lib/app-observations";
import { pendingFailure, uncertainAfter } from "@/lib/observation-contracts";
import { completeOperation } from "@/lib/complete-operation";
import {
  PENDING_KEY,
  readPending,
  repositoryError,
  repositoryRequest,
  type PendingRegistration,
  type Registration,
  type Repository,
  type RepositoryError,
  type RepositoryPage,
} from "@/lib/repositories";

export function useRepositories(
  incarnation: string | undefined,
  app: AppObservations,
  selectedId: string | null,
  navigation: Navigation,
  visible = true,
) {
  const feature = app.repositories;
  useSyncExternalStore(feature.subscribe, feature.getSnapshot);
  const [after, setAfter] = useState<string | null>(null);
  const [previous, setPrevious] = useState<(string | null)[]>([]);
  const [pending, setPending] = useState(readPending);
  const currentPending = useRef(pending);
  const [actionError, setActionError] = useState<RepositoryError>();
  const [submitting, setSubmitting] = useState(false);
  const [preferenceError, setPreferenceError] = useState<string>();
  const mounted = useRef(false);
  const actionEpoch = useRef(0);
  const pendingNavigation = useRef<number | undefined>(undefined);
  const sending = useRef(false);
  const shownIncarnation = incarnation ?? feature.retainedIncarnation();
  const list = shownIncarnation
    ? feature.peek(shownIncarnation, { operation: "list", after })
    : undefined;
  const detail =
    shownIncarnation && selectedId
      ? feature.peek(shownIncarnation, { operation: "get", id: selectedId })
      : undefined;
  const page =
    list?.data?.value.kind === "repositories"
      ? list.data.value.page
      : undefined;
  const selected =
    detail?.data?.value.kind === "repository"
      ? (detail.data.value.repository ?? undefined)
      : page?.repositories.find((item) => item.id === selectedId);
  const observedIncarnation =
    (selectedId
      ? (detail?.data?.incarnation ?? list?.data?.incarnation)
      : list?.data?.incarnation) ?? incarnation;
  const actionId = pending?.request.action_id;
  const error = list?.error ?? detail?.error;
  const loading =
    !list ||
    list.fetchStatus === "fetching" ||
    (list.status === "pending" && !list.data);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      actionEpoch.current += 1;
      pendingNavigation.current = undefined;
    };
  }, [incarnation]);
  useEffect(() => {
    if (!incarnation || !visible) return;
    return feature.observe(incarnation, { operation: "list", after });
  }, [feature, incarnation, after, visible]);
  useEffect(() => {
    if (!incarnation || !visible || !selectedId) return;
    return feature.observe(incarnation, { operation: "get", id: selectedId });
  }, [feature, incarnation, selectedId, visible]);
  useEffect(() => {
    if (!incarnation || !actionId) return;
    return feature.observe(incarnation, {
      operation: "action",
      action_id: actionId,
    });
  }, [feature, incarnation, actionId]);
  const finish = useCallback(
    (action: Registration) => {
      const retained = currentPending.current;
      if (
        !retained ||
        action.action_id !== retained.request.action_id ||
        action.path !== retained.request.path ||
        action.create_initial_workspace !==
          (retained.request.create_initial_workspace ?? false) ||
        action.state === "pending"
      )
        return;
      if (action.state === "completed") {
        navigation.complete(
          pendingNavigation.current,
          action.initial_workspace_id
            ? {
                kind: "workspace",
                repositoryId: action.repository_id,
                workspaceId: action.initial_workspace_id,
                view: "workspace",
              }
            : { kind: "repository", repositoryId: action.repository_id },
        );
      }
      currentPending.current = null;
      setPending(null);
      try {
        localStorage.removeItem(PENDING_KEY);
      } catch {
        setPreferenceError(
          "The completed registration could not be removed from desktop preferences. Reopening will retrieve its recorded result.",
        );
      }
      setActionError(action.state === "failed" ? action.error : undefined);
    },
    [navigation],
  );
  useEffect(
    () =>
      feature.subscribe(() => {
        if (!incarnation || !actionId || !currentPending.current) return;
        const action = feature.peek(incarnation, {
          operation: "action",
          action_id: actionId,
        });
        if (!action) return;
        if (action.error) {
          setActionError(repositoryError(action.error));
          return;
        }
        const result = action.data?.value;
        if (result?.kind !== "action") return;
        if (!result.registration) {
          if (!sending.current)
            setActionError(
              (last) =>
                last ?? {
                  code: "not_found",
                  message:
                    "The core has no recorded registration for this action. Submit the same request explicitly to try again.",
                },
            );
        } else {
          const request = currentPending.current.request;
          if (
            result.registration.path !== request.path ||
            result.registration.create_initial_workspace !==
              (request.create_initial_workspace ?? false)
          )
            setActionError({
              code: "conflict",
              message:
                "This action ID records a different path. Preserve its ID and inspect the recorded registration before submitting another request.",
            });
          else finish(result.registration);
        }
      }),
    [feature, incarnation, actionId, finish],
  );
  function refresh(recheck = true) {
    return incarnation
      ? feature.refreshSelection(
          incarnation,
          after,
          selectedId,
          actionId,
          recheck,
        )
      : Promise.resolve();
  }
  async function register(path: string): Promise<boolean> {
    if (!incarnation || !observedIncarnation || sending.current) return false;
    const old = currentPending.current;
    if (old && old.request.path !== path) {
      setActionError({
        code: "conflict",
        message:
          "Retrieve the pending registration result before changing its path.",
      });
      return false;
    }
    const target = navigation.current();
    const intent: PendingRegistration = old ?? {
      request: {
        action_id: crypto.randomUUID(),
        path,
        create_initial_workspace: true,
      },
      selection: target && target.kind !== "home" ? target.repositoryId : null,
    };
    const earlierUncertain = intent.uncertain === true;
    // Persist before IPC so a desktop crash cannot turn a transmission into "not sent".
    const transmitted = { ...intent, uncertain: true };
    try {
      localStorage.setItem(PENDING_KEY, JSON.stringify(transmitted));
    } catch {
      setActionError({
        code: "storage",
        message:
          "The desktop could not retain this action ID. Registration was not submitted. Check local storage and try again.",
      });
      return false;
    }
    currentPending.current = transmitted;
    setPending(transmitted);
    setActionError(undefined);
    setSubmitting(true);
    sending.current = true;
    pendingNavigation.current = navigation.capture();
    const generation = actionEpoch.current;
    return completeOperation(
      async () => {
        const result = await repositoryRequest(observedIncarnation, {
          operation: "register",
          request: intent.request,
        });
        if (
          mounted.current &&
          actionEpoch.current === generation &&
          app.freshness.matches(observedIncarnation) &&
          result.kind === "registration"
        )
          finish(result.registration);
        return true;
      },
      (error) => {
        if (
          mounted.current &&
          actionEpoch.current === generation &&
          app.freshness.matches(observedIncarnation) &&
          currentPending.current?.request.action_id === intent.request.action_id
        ) {
          const failure = repositoryError(error);
          setActionError(pendingFailure(earlierUncertain, failure));
          const uncertain = uncertainAfter(earlierUncertain, failure);
          if (
            !uncertain &&
            (failure.code === "invalid_path" || failure.code === "capacity")
          ) {
            currentPending.current = null;
            setPending(null);
            try {
              localStorage.removeItem(PENDING_KEY);
            } catch {
              setPreferenceError(
                "A rejected registration could not be removed from desktop preferences.",
              );
            }
          } else {
            const retained = { ...intent, uncertain };
            currentPending.current = retained;
            setPending(retained);
            try {
              localStorage.setItem(PENDING_KEY, JSON.stringify(retained));
            } catch {
              setPreferenceError(
                "The desktop could not update delivery evidence; the retained action remains uncertain.",
              );
            }
          }
        }
        return false;
      },
      () => {
        sending.current = false;
        if (mounted.current) setSubmitting(false);
        if (mounted.current && actionEpoch.current === generation)
          void refresh(false);
      },
    );
  }
  function select(repository: Repository) {
    navigation.go({ kind: "repository", repositoryId: repository.id });
  }
  function next() {
    if (!page?.next) return;
    setPrevious((last) => [...last, after]);
    setAfter(page.next);
  }
  function back() {
    if (!previous.length) return;
    setAfter(previous.at(-1) ?? null);
    setPrevious((last) => last.slice(0, -1));
  }
  const observation: {
    page?: RepositoryPage;
    selected?: Repository;
    error?: RepositoryError;
  } = { page, selected, error: error ? repositoryError(error) : undefined };
  return {
    ...observation,
    loading,
    pending,
    submitting,
    actionError,
    preferenceError,
    register,
    refresh,
    select,
    next,
    back,
    hasPrevious: previous.length > 0,
  };
}
export type Repositories = ReturnType<typeof useRepositories>;
