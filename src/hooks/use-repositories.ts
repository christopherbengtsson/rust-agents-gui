import {
  useCallback,
  useEffect,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import { useStore } from "zustand";
import type { AppObservations } from "@/lib/app-observations";
import { pendingFailure, uncertainAfter } from "@/lib/observation-contracts";
import { completeOperation } from "@/lib/complete-operation";
import {
  PENDING_KEY,
  SELECTION_KEY,
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
  owner: string | undefined,
  app: AppObservations,
) {
  const feature = app.repositories;
  useSyncExternalStore(feature.subscribe, feature.getSnapshot);
  const selectedId = useStore(app.presentation, (state) => state.repository);
  const [after, setAfter] = useState<string | null>(null);
  const [previous, setPrevious] = useState<(string | null)[]>([]);
  const [initialWorkspace, setInitialWorkspace] = useState<{
    repository_id: string;
    workspace_id: string;
  } | null>(null);
  const [pending, setPending] = useState(readPending);
  const currentPending = useRef(pending);
  const [actionError, setActionError] = useState<RepositoryError>();
  const [submitting, setSubmitting] = useState(false);
  const [preferenceError, setPreferenceError] = useState<string>();
  const mounted = useRef(false);
  const actionEpoch = useRef(0);
  const navigation = useRef(0);
  const pendingNavigation = useRef(0);
  const sending = useRef(false);
  const shownOwner = owner ?? feature.retainedOwner();
  const list = shownOwner
    ? feature.peek(shownOwner, { operation: "list", after })
    : undefined;
  const detail =
    shownOwner && selectedId
      ? feature.peek(shownOwner, { operation: "get", id: selectedId })
      : undefined;
  const page =
    list?.data?.value.kind === "repositories"
      ? list.data.value.page
      : undefined;
  const selected =
    detail?.data?.value.kind === "repository"
      ? (detail.data.value.repository ?? undefined)
      : page?.repositories.find((item) => item.id === selectedId);
  const observedOwner =
    (selectedId
      ? (detail?.data?.owner ?? list?.data?.owner)
      : list?.data?.owner) ?? owner;
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
    };
  }, [owner]);
  useEffect(() => {
    if (!owner) return;
    return feature.observe(owner, { operation: "list", after });
  }, [feature, owner, after]);
  useEffect(() => {
    if (!owner || !selectedId) return;
    return feature.observe(owner, { operation: "get", id: selectedId });
  }, [feature, owner, selectedId]);
  useEffect(() => {
    if (!owner || !actionId) return;
    return feature.observe(owner, { operation: "action", action_id: actionId });
  }, [feature, owner, actionId]);
  const rememberSelection = useCallback(
    (id: string | null) => {
      app.presentation.setState({ repository: id });
      try {
        if (id) localStorage.setItem(SELECTION_KEY, id);
        else localStorage.removeItem(SELECTION_KEY);
      } catch {
        setPreferenceError(
          "The desktop could not remember repository selection. Check its local storage before reopening.",
        );
      }
    },
    [app],
  );
  useEffect(
    () =>
      feature.subscribe(() => {
        if (!owner || app.presentation.getState().repository !== selectedId)
          return;
        const list = feature.peek(owner, { operation: "list", after });
        const page =
          list?.data?.value.kind === "repositories"
            ? list.data.value.page
            : undefined;
        const detail = selectedId
          ? feature.peek(owner, { operation: "get", id: selectedId })
          : undefined;
        if (!page || list?.status !== "success") return;
        if (
          !selectedId ||
          (detail?.status === "success" &&
            detail.data?.value.kind === "repository" &&
            !detail.data.value.repository)
        )
          rememberSelection(page.repositories[0]?.id ?? null);
      }),
    [feature, app, owner, after, selectedId, rememberSelection],
  );
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
      if (
        action.state === "completed" &&
        navigation.current === pendingNavigation.current &&
        app.presentation.getState().repository === retained.selection
      ) {
        rememberSelection(action.repository_id);
        if (action.initial_workspace_id)
          setInitialWorkspace({
            repository_id: action.repository_id,
            workspace_id: action.initial_workspace_id,
          });
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
    [app, rememberSelection],
  );
  useEffect(
    () =>
      feature.subscribe(() => {
        if (!owner || !actionId || !currentPending.current) return;
        const action = feature.peek(owner, {
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
                    "The supervisor has no recorded registration for this action. Submit the same request explicitly to try again.",
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
    [feature, owner, actionId, finish],
  );
  function refresh(recheck = true) {
    return owner
      ? feature.refreshSelection(owner, after, selectedId, actionId, recheck)
      : Promise.resolve();
  }
  async function register(path: string): Promise<boolean> {
    if (!owner || !observedOwner || sending.current) return false;
    const old = currentPending.current;
    if (old && old.request.path !== path) {
      setActionError({
        code: "conflict",
        message:
          "Retrieve the pending registration result before changing its path.",
      });
      return false;
    }
    const intent: PendingRegistration = old ?? {
      request: {
        action_id: crypto.randomUUID(),
        path,
        create_initial_workspace: true,
      },
      selection: app.presentation.getState().repository,
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
    pendingNavigation.current = navigation.current;
    const generation = actionEpoch.current;
    return completeOperation(
      async () => {
        const result = await repositoryRequest(observedOwner, {
          operation: "register",
          request: intent.request,
        });
        if (
          mounted.current &&
          actionEpoch.current === generation &&
          app.freshness.matches(observedOwner) &&
          result.kind === "registration"
        )
          finish(result.registration);
        return true;
      },
      (error) => {
        if (
          mounted.current &&
          actionEpoch.current === generation &&
          app.freshness.matches(observedOwner) &&
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
    navigation.current++;
    rememberSelection(repository.id);
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
    initialWorkspace,
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
