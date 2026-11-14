import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import type { Navigation } from "@/lib/navigation";
import type { AppObservations } from "@/lib/app-observations";
import { pendingFailure, uncertainAfter } from "@/lib/observation-contracts";
import { completeOperation } from "@/lib/complete-operation";
import { repositoryError, type RepositoryError } from "@/lib/repositories";
import {
  readWorkspacePending,
  WORKSPACE_PENDING,
  workspaceRequest,
  type Workspace,
  type WorkspaceAction,
  type WorkspacePage,
} from "@/lib/workspaces";

export function useWorkspaces(
  owner: string | undefined,
  repositoryId: string | undefined,
  repositoryIds: string,
  workspaceId: string | undefined,
  app: AppObservations,
  navigation: Navigation,
) {
  const feature = app.workspaces;
  useSyncExternalStore(feature.subscribe, feature.getSnapshot);
  const [restoredPending] = useState(readWorkspacePending);
  const [pending, setPending] = useState(restoredPending?.request ?? null);
  const uncertain = useRef(restoredPending?.uncertain ?? false);
  const intent = useRef(pending);
  const [actionError, setActionError] = useState<RepositoryError>();
  const [submitting, setSubmitting] = useState(false);
  const [preferenceError, setPreferenceError] = useState<string>();
  const mounted = useRef(false);
  const actionEpoch = useRef(0);
  const pendingNavigation = useRef<number | undefined>(undefined);
  const sending = useRef(false);
  const [cursors, setCursors] = useState<Record<string, (string | null)[]>>({});
  const shownOwner = owner ?? feature.retainedOwner();
  const targets = useMemo(
    () =>
      Array.from(
        new Set([
          ...repositoryIds.split(",").filter(Boolean),
          ...(repositoryId ? [repositoryId] : []),
        ]),
      ).map((id) => ({
        operation: "list" as const,
        repository_id: id,
        after: cursors[id]?.at(-1) ?? null,
      })),
    [repositoryIds, repositoryId, cursors],
  );
  const wanted = workspaceId;
  const lists = shownOwner
    ? targets.map((target) => feature.peek(shownOwner, target))
    : [];
  const detail =
    shownOwner && wanted
      ? feature.peek(shownOwner, { operation: "get", id: wanted })
      : undefined;
  const pages: Record<string, WorkspacePage> = {};
  for (let index = 0; index < targets.length; index++) {
    const value = lists[index]?.data?.value;
    if (value?.kind === "workspaces")
      pages[targets[index].repository_id] = value.page;
  }
  const selected =
    detail?.data?.value.kind === "workspace"
      ? (detail.data.value.workspace ?? undefined)
      : undefined;
  const observedOwner =
    detail?.data?.owner ??
    lists.find((state) => state?.data)?.data?.owner ??
    owner;
  const actionId = pending?.action_id;
  const observationError =
    lists.find((state) => state?.error)?.error ?? detail?.error;
  const loading = lists.some(
    (state) => !state || state.fetchStatus === "fetching",
  );
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      actionEpoch.current += 1;
      pendingNavigation.current = undefined;
    };
  }, [owner]);
  useEffect(() => {
    if (!owner) return;
    const releases = targets.map((target) => feature.observe(owner, target));
    return () => releases.forEach((release) => release());
  }, [feature, owner, targets]);
  useEffect(() => {
    if (!owner || !wanted) return;
    return feature.observe(owner, { operation: "get", id: wanted });
  }, [feature, owner, wanted]);
  useEffect(() => {
    if (!owner || !actionId) return;
    return feature.observe(owner, { operation: "action", action_id: actionId });
  }, [feature, owner, actionId]);
  const settleAction = useCallback(
    (request: WorkspaceAction, workspace: Workspace) => {
      if (intent.current?.action_id !== request.action_id) return;
      const target = navigation.current();
      if (
        target &&
        target.kind !== "home" &&
        target.repositoryId === workspace.repository_id
      )
        navigation.complete(pendingNavigation.current, {
          kind: "workspace",
          repositoryId: workspace.repository_id,
          workspaceId: workspace.id,
          view: "workspace",
        });
      setActionError(undefined);
      intent.current = null;
      setPending(null);
      try {
        localStorage.removeItem(WORKSPACE_PENDING);
      } catch {
        setPreferenceError(
          "The recorded workspace action could not be cleared from desktop preferences; reopening will retrieve its status.",
        );
      }
    },
    [navigation],
  );
  useEffect(
    () =>
      feature.subscribe(() => {
        if (!owner || !actionId || !intent.current) return;
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
        if (result.action && result.workspace) {
          const outstanding = intent.current;
          const matches =
            result.action.action_id === outstanding.action_id &&
            result.action.operation === outstanding.operation &&
            (result.action.operation === "create" &&
            outstanding.operation === "create"
              ? result.action.repository_id === outstanding.repository_id
              : result.action.operation === "reconcile" &&
                outstanding.operation === "reconcile" &&
                result.action.workspace_id === outstanding.workspace_id);
          if (!matches) {
            setActionError({
              code: "conflict",
              message:
                "Workspace action payload conflicts with the retained request.",
            });
            return;
          }
          settleAction(outstanding, result.workspace);
          feature.recordAction(owner, result);
        } else if (!sending.current)
          setActionError(
            (last) =>
              last ?? {
                code: "not_found",
                message:
                  "No recorded workspace action was found. Retry the same submission explicitly; refresh does not create another workspace.",
              },
          );
      }),
    [owner, actionId, feature, settleAction],
  );
  async function submit(request: WorkspaceAction) {
    if (!owner || !observedOwner || sending.current) return;
    if (
      intent.current &&
      JSON.stringify(intent.current) !== JSON.stringify(request)
    ) {
      setActionError({
        code: "conflict",
        message:
          "Retrieve the outstanding workspace action before submitting another.",
      });
      return;
    }
    const earlierUncertain = intent.current ? uncertain.current : false;
    try {
      localStorage.setItem(
        WORKSPACE_PENDING,
        JSON.stringify({ request, uncertain: true }),
      );
    } catch {
      setActionError({
        code: "storage",
        message:
          "The desktop could not retain this action ID. No workspace request was submitted.",
      });
      return;
    }
    pendingNavigation.current = navigation.capture();
    intent.current = request;
    uncertain.current = true;
    setPending(request);
    sending.current = true;
    setSubmitting(true);
    setActionError(undefined);
    const actionGeneration = actionEpoch.current;
    return completeOperation(
      async () => {
        const result = await workspaceRequest(observedOwner, request);
        if (
          !mounted.current ||
          actionEpoch.current !== actionGeneration ||
          !app.freshness.matches(observedOwner) ||
          intent.current?.action_id !== request.action_id
        )
          return;
        if (result.kind === "action" && result.workspace) {
          settleAction(request, result.workspace);
          feature.recordAction(observedOwner, result);
        }
      },
      (error) => {
        if (
          mounted.current &&
          actionEpoch.current === actionGeneration &&
          app.freshness.matches(observedOwner) &&
          intent.current?.action_id === request.action_id
        ) {
          const failure = repositoryError(error);
          setActionError(pendingFailure(earlierUncertain, failure));
          uncertain.current = uncertainAfter(earlierUncertain, failure);
          if (
            !uncertain.current &&
            [
              "capacity",
              "not_found",
              "conflict",
              "cleanup_unconfirmed",
            ].includes(failure.code)
          ) {
            // These typed responses reject admission before a durable creation intent.
            // Storage/transport failures retain the identity for authoritative lookup.
            intent.current = null;
            setPending(null);
            try {
              localStorage.removeItem(WORKSPACE_PENDING);
            } catch {
              setPreferenceError(
                "The rejected workspace request could not be cleared from desktop preferences.",
              );
            }
          } else {
            try {
              localStorage.setItem(
                WORKSPACE_PENDING,
                JSON.stringify({ request, uncertain: uncertain.current }),
              );
            } catch {
              setPreferenceError(
                "The desktop could not update delivery evidence; the retained action remains uncertain.",
              );
            }
          }
        }
      },
      () => {
        sending.current = false;
        if (mounted.current) {
          setSubmitting(false);
          if (actionEpoch.current === actionGeneration)
            void feature.refresh(owner);
        }
      },
    );
  }
  function select(workspace: Workspace) {
    if (shownOwner) feature.recordWorkspace(shownOwner, workspace);
    navigation.go({
      kind: "workspace",
      repositoryId: workspace.repository_id,
      workspaceId: workspace.id,
      view:
        app.presentation.getState().contexts[workspace.id]?.view ?? "workspace",
    });
  }
  function selectRepository() {
    if (repositoryId) navigation.go({ kind: "repository", repositoryId });
  }
  function create(repository_id: string) {
    return submit({
      operation: "create",
      action_id: crypto.randomUUID(),
      repository_id,
    });
  }
  function reconcile(workspace: Workspace) {
    return submit({
      operation: "reconcile",
      action_id: crypto.randomUUID(),
      workspace_id: workspace.id,
    });
  }
  function refresh() {
    return owner ? feature.refresh(owner) : Promise.resolve();
  }
  function next(id: string) {
    const cursor = pages[id]?.next;
    if (cursor)
      setCursors((last) => ({
        ...last,
        [id]: [...(last[id] ?? [null]), cursor],
      }));
  }
  function back(id: string) {
    setCursors((last) => ({
      ...last,
      [id]: (last[id] ?? [null]).slice(0, -1),
    }));
  }
  return {
    pages,
    selected: selected?.repository_id === repositoryId ? selected : undefined,
    pending,
    error:
      actionError ??
      (observationError ? repositoryError(observationError) : undefined),
    loading,
    submitting,
    preferenceError,
    select,
    selectRepository,
    create,
    reconcile,
    refresh,
    next,
    back,
    hasPrevious: (id: string) => (cursors[id]?.length ?? 0) > 1,
    retry: () => (intent.current ? submit(intent.current) : Promise.resolve()),
  };
}
export type Workspaces = ReturnType<typeof useWorkspaces>;
