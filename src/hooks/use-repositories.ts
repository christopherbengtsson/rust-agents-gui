import { completeOperation } from "@/lib/complete-operation";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  PENDING_KEY,
  SELECTION_KEY,
  readPending,
  readSelection,
  repositoryError,
  repositoryRequest,
  type PendingRegistration,
  type Registration,
  type Repository,
  type RepositoryError,
  type RepositoryPage,
} from "@/lib/repositories";

type Navigation = {
  page?: RepositoryPage;
  selected?: Repository;
  loading: boolean;
  error?: RepositoryError;
};

// App calls this once. Selection and action ownership survive disclosure/dialog changes.
export function useRepositories(owner: string | undefined) {
  const [initialWorkspace, setInitialWorkspace] = useState<{
    repository_id: string;
    workspace_id: string;
  } | null>(null);
  const [navigation, setNavigation] = useState<Navigation>({ loading: true });
  const [pending, setPending] = useState(readPending);
  const [actionError, setActionError] = useState<RepositoryError>();
  const [submitting, setSubmitting] = useState(false);
  const [preferenceError, setPreferenceError] = useState<string>();
  const [restoredSelection] = useState(readSelection);
  const selection = useRef(restoredSelection);
  const currentPending = useRef(pending);
  const after = useRef<string | null>(null);
  const [previous, setPrevious] = useState<(string | null)[]>([]);
  const epoch = useRef(0);
  const mounted = useRef(false);
  const reading = useRef(false);
  const queued = useRef<boolean | null>(null);
  const sending = useRef(false);

  function rememberSelection(id: string | null) {
    selection.current = id;
    try {
      if (id) localStorage.setItem(SELECTION_KEY, id);
      else localStorage.removeItem(SELECTION_KEY);
    } catch {
      setPreferenceError(
        "The desktop could not remember repository selection. Check its local storage before reopening.",
      );
    }
  }

  // Stable callbacks bind observation subscriptions to their lifecycle owner.
  const finish = useCallback((action: Registration) => {
    const pending = currentPending.current;
    if (
      !pending ||
      action.action_id !== pending.request.action_id ||
      action.path !== pending.request.path ||
      action.state === "pending"
    )
      return;
    if (
      action.state === "completed" &&
      selection.current === pending.selection
    ) {
      selection.current = action.repository_id;
      if (action.initial_workspace_id)
        setInitialWorkspace({
          repository_id: action.repository_id,
          workspace_id: action.initial_workspace_id,
        });
      try {
        localStorage.setItem(SELECTION_KEY, action.repository_id);
      } catch {
        setPreferenceError(
          "The desktop could not remember repository selection.",
        );
      }
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
  }, []);

  const readRepositories = useCallback(
    async function read(recheck = true): Promise<void> {
      if (!mounted.current) return;
      if (reading.current) {
        queued.current = recheck || queued.current === true;
        return;
      }
      reading.current = true;
      const generation = epoch.current;
      const pageAfter = after.current;
      const active = () => mounted.current && epoch.current === generation;
      return completeOperation(
        async () => {
          const pending = currentPending.current;
          if (pending) {
            const result = await repositoryRequest({
              operation: "action",
              action_id: pending.request.action_id,
            }).catch((error: unknown) => ({
              kind: "error" as const,
              error: repositoryError(error),
            }));
            if (!active()) return;
            if (result.kind === "error") {
              setActionError(result.error);
            } else if (result.kind === "action") {
              if (result.registration) {
                if (result.registration.path !== pending.request.path) {
                  setActionError({
                    code: "conflict",
                    message:
                      "This action ID records a different path. Preserve its ID and inspect the recorded registration before submitting another request.",
                  });
                } else {
                  setActionError(undefined);
                  finish(result.registration);
                }
              } else
                setActionError(
                  (last) =>
                    last ?? {
                      code: "not_found",
                      message:
                        "The supervisor has no recorded registration for this action. Submit the same request explicitly to try again.",
                    },
                );
            }
          }
          const requestedSelection = selection.current;
          const [list, selected] = await Promise.all([
            repositoryRequest({
              operation: "list",
              after: pageAfter,
              refresh: recheck,
            }),
            requestedSelection
              ? repositoryRequest({
                  operation: "get",
                  id: requestedSelection,
                  refresh: recheck,
                })
              : Promise.resolve(null),
          ]);
          if (
            !active() ||
            after.current !== pageAfter ||
            list.kind !== "repositories"
          )
            return;
          const candidate =
            selected?.kind === "repository" ? selected.repository : null;
          const chosen = candidate ?? list.page.repositories[0];
          if (selection.current === requestedSelection) {
            selection.current = chosen?.id ?? null;
            try {
              if (chosen) localStorage.setItem(SELECTION_KEY, chosen.id);
              else localStorage.removeItem(SELECTION_KEY);
            } catch {
              setPreferenceError(
                "The desktop could not remember repository selection.",
              );
            }
          }
          setNavigation((last) => ({
            page: list.page,
            selected: selection.current === chosen?.id ? chosen : last.selected,
            loading: false,
          }));
        },
        (error) => {
          if (active())
            setNavigation((last) => ({
              ...last,
              loading: false,
              error: repositoryError(error),
            }));
        },
        () => {
          if (active()) {
            reading.current = false;
            if (queued.current !== null) {
              const next = queued.current;
              queued.current = null;
              void read(next);
            }
          }
        },
      );
    },
    [finish],
  );

  useEffect(() => {
    mounted.current = true;
    if (owner) void readRepositories();
    return () => {
      mounted.current = false;
      epoch.current += 1;
      reading.current = false;
      queued.current = null;
    };
  }, [owner, readRepositories]);

  const checking =
    navigation.page?.repositories.some(
      (r) => r.availability.state === "checking",
    ) || navigation.selected?.availability.state === "checking";
  useEffect(() => {
    if (
      !owner ||
      navigation.loading ||
      navigation.error ||
      ((!pending || actionError) && !checking)
    )
      return;
    const timer = setTimeout(() => void readRepositories(false), 250);
    return () => clearTimeout(timer);
  }, [owner, navigation, pending, actionError, checking, readRepositories]);

  // Explicit intents show loading immediately; observation Effects only start reads.
  function refresh(recheck = true): Promise<void> {
    if (!mounted.current) return Promise.resolve();
    setNavigation((last) => ({ ...last, loading: true }));
    return readRepositories(recheck);
  }

  async function register(path: string): Promise<boolean> {
    if (!owner || sending.current) return false;
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
      selection: selection.current,
    };
    try {
      localStorage.setItem(PENDING_KEY, JSON.stringify(intent));
    } catch {
      setActionError({
        code: "storage",
        message:
          "The desktop could not retain this action ID. Registration was not submitted. Check local storage and try again.",
      });
      return false;
    }
    currentPending.current = intent;
    setPending(intent);
    setActionError(undefined);
    setSubmitting(true);
    sending.current = true;
    const generation = epoch.current;
    return completeOperation(
      async () => {
        const result = await repositoryRequest({
          operation: "register",
          request: intent.request,
        });
        if (
          mounted.current &&
          epoch.current === generation &&
          result.kind === "registration"
        )
          finish(result.registration);
        return true;
      },
      (error) => {
        if (mounted.current && epoch.current === generation) {
          const failure = repositoryError(error);
          setActionError(failure);
          if (failure.code === "invalid_path" || failure.code === "capacity") {
            currentPending.current = null;
            setPending(null);
            try {
              localStorage.removeItem(PENDING_KEY);
            } catch {
              setPreferenceError(
                "A rejected registration could not be removed from desktop preferences.",
              );
            }
          }
        }
        return false;
      },
      () => {
        sending.current = false;
        if (mounted.current) setSubmitting(false);
        if (mounted.current && epoch.current === generation)
          void refresh(false);
      },
    );
  }
  function select(repository: Repository) {
    rememberSelection(repository.id);
    setNavigation((last) => ({ ...last, selected: repository }));
  }
  function next() {
    if (!navigation.page?.next) return;
    const current = after.current;
    setPrevious((last) => [...last, current]);
    after.current = navigation.page.next;
    void refresh(false);
  }
  function back() {
    if (!previous.length) return;
    after.current = previous[previous.length - 1];
    setPrevious((last) => last.slice(0, -1));
    void refresh(false);
  }
  return {
    ...navigation,
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
