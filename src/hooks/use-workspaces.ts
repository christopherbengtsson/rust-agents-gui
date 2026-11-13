import { useCallback, useEffect, useRef, useState } from "react";
import { repositoryError, type RepositoryError } from "@/lib/repositories";
import {
  readWorkspacePending,
  readWorkspaceSelection,
  WORKSPACE_PENDING,
  WORKSPACE_SELECTION,
  workspaceRequest,
  type Workspace,
  type WorkspaceAction,
  type WorkspacePage,
} from "@/lib/workspaces";

// App owns this once. Disclosure/selection never owns or repeats Rust execution.
export function useWorkspaces(
  owner: string | undefined,
  repositoryId: string | undefined,
  repositoryIds: string,
  initial: { repository_id: string; workspace_id: string } | null,
) {
  const [pages, setPages] = useState<Record<string, WorkspacePage>>({});
  const [selected, setSelected] = useState<Workspace>();
  const [pending, setPending] = useState(readWorkspacePending);
  const [error, setError] = useState<RepositoryError>();
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [preferenceError, setPreferenceError] = useState<string>();
  const selection = useRef(readWorkspaceSelection());
  const intent = useRef(pending);
  const mounted = useRef(false);
  const epoch = useRef(0);
  const navigation = useRef(0);
  const pendingNavigation = useRef(0);
  const reading = useRef(false);
  const queued = useRef(false);
  const sending = useRef(false);
  const cursors = useRef<Record<string, (string | null)[]>>({});
  const handledInitial = useRef<string | undefined>(undefined);
  const remember = useCallback((workspace: Workspace | null) => {
    selection.current = workspace
      ? { repository_id: workspace.repository_id, workspace_id: workspace.id }
      : null;
    setSelected(workspace ?? undefined);
    try {
      if (selection.current)
        localStorage.setItem(
          WORKSPACE_SELECTION,
          JSON.stringify(selection.current),
        );
      else localStorage.removeItem(WORKSPACE_SELECTION);
    } catch {
      setPreferenceError("The desktop could not remember workspace selection.");
    }
  }, []);
  // Stable synchronization callback while finally cleanup remains outside compiler lowering.
  const read = useCallback(
    async function refresh(): Promise<void> {
      if (!mounted.current || !owner) return;
      if (reading.current) {
        queued.current = true;
        return;
      }
      reading.current = true;
      const generation = epoch.current;
      const active = () => mounted.current && epoch.current === generation;
      try {
        const outstanding = intent.current;
        if (outstanding) {
          const result = await workspaceRequest({
            operation: "action",
            action_id: outstanding.action_id,
          });
          if (!active()) return;
          if (result.kind === "action" && result.action && result.workspace) {
            // A lookup validates its ID; also match the retained request payload.
            if (
              JSON.stringify(result.action) !== JSON.stringify(outstanding) &&
              (result.action.operation !== outstanding.operation ||
                (result.action.operation === "create" &&
                outstanding.operation === "create"
                  ? result.action.repository_id !== outstanding.repository_id
                  : result.action.operation === "reconcile" &&
                    outstanding.operation === "reconcile" &&
                    result.action.workspace_id !== outstanding.workspace_id))
            )
              throw new Error(
                "Workspace action payload conflicts with the retained request.",
              );
            if (
              navigation.current === pendingNavigation.current &&
              result.workspace.repository_id === repositoryId
            )
              remember(result.workspace);
            intent.current = null;
            setPending(null);
            try {
              localStorage.removeItem(WORKSPACE_PENDING);
            } catch {
              setPreferenceError(
                "The recorded workspace action could not be cleared from desktop preferences; reopening will retrieve its status.",
              );
            }
          } else
            setError({
              code: "not_found",
              message:
                "No recorded workspace action was found. Retry the same submission explicitly; refresh does not create another workspace.",
            });
        }
        const ids = Array.from(
          new Set([
            ...repositoryIds.split(",").filter(Boolean),
            ...(repositoryId ? [repositoryId] : []),
          ]),
        );
        const target = selection.current;
        const wanted =
          target && target.repository_id === repositoryId
            ? target.workspace_id
            : initial &&
                initial.repository_id === repositoryId &&
                handledInitial.current !== initial.workspace_id
              ? initial.workspace_id
              : undefined;
        const [lists, detail] = await Promise.all([
          Promise.all(
            ids.map(async (id) => {
              const history = cursors.current[id];
              const result = await workspaceRequest({
                operation: "list",
                repository_id: id,
                after: history?.at(-1) ?? null,
              });
              return [id, result] as const;
            }),
          ),
          wanted
            ? workspaceRequest({ operation: "get", id: wanted })
            : Promise.resolve(null),
        ]);
        if (!active()) return;
        const next: Record<string, WorkspacePage> = {};
        for (const [id, result] of lists)
          if (result.kind === "workspaces") next[id] = result.page;
        setPages(next);
        if (
          detail?.kind === "workspace" &&
          detail.workspace &&
          selection.current === target
        ) {
          if (initial?.workspace_id === detail.workspace.id)
            handledInitial.current = detail.workspace.id;
          remember(detail.workspace);
        }
        if (!outstanding || !intent.current) setError(undefined);
      } catch (error) {
        if (active()) setError(repositoryError(error));
      } finally {
        if (active()) {
          reading.current = false;
          setLoading(false);
          if (queued.current) {
            queued.current = false;
            void refresh();
          }
        }
      }
    },
    [owner, repositoryId, repositoryIds, initial, remember],
  );
  useEffect(() => {
    mounted.current = true;
    void Promise.resolve().then(read);
    return () => {
      mounted.current = false;
      epoch.current += 1;
      reading.current = false;
      queued.current = false;
    };
  }, [read]);
  const busy =
    Object.values(pages).some((p) =>
      p.workspaces.some(
        (w) => w.status.state === "queued" || w.status.state === "running",
      ),
    ) ||
    selected?.status.state === "queued" ||
    selected?.status.state === "running";
  useEffect(() => {
    if (!owner || error || (!busy && !pending)) return;
    const timer = setTimeout(() => void read(), 250);
    return () => clearTimeout(timer);
  }, [owner, error, busy, pending, pages, selected, read]);
  async function submit(request: WorkspaceAction) {
    if (!owner || sending.current) return;
    if (
      intent.current &&
      JSON.stringify(intent.current) !== JSON.stringify(request)
    ) {
      setError({
        code: "conflict",
        message:
          "Retrieve the outstanding workspace action before submitting another.",
      });
      return;
    }
    try {
      localStorage.setItem(WORKSPACE_PENDING, JSON.stringify(request));
    } catch {
      setError({
        code: "storage",
        message:
          "The desktop could not retain this action ID. No workspace request was submitted.",
      });
      return;
    }
    pendingNavigation.current = navigation.current;
    intent.current = request;
    setPending(request);
    sending.current = true;
    setSubmitting(true);
    setError(undefined);
    const generation = epoch.current;
    const nav = navigation.current;
    try {
      const result = await workspaceRequest(request);
      if (!mounted.current || epoch.current !== generation) return;
      if (result.kind === "action" && result.workspace) {
        if (
          navigation.current === nav &&
          result.workspace.repository_id === repositoryId
        )
          remember(result.workspace);
        intent.current = null;
        setPending(null);
        try {
          localStorage.removeItem(WORKSPACE_PENDING);
        } catch {
          setPreferenceError(
            "The desktop could not clear the recorded workspace request.",
          );
        }
      }
    } catch (error) {
      if (mounted.current && epoch.current === generation)
        setError(repositoryError(error));
    } finally {
      sending.current = false;
      if (mounted.current) {
        setSubmitting(false);
        if (epoch.current === generation) void read();
      }
    }
  }
  function select(workspace: Workspace) {
    navigation.current++;
    remember(workspace);
  }
  function selectRepository() {
    navigation.current++;
    remember(null);
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
    setLoading(true);
    return read();
  }
  function next(id: string) {
    const cursor = pages[id]?.next;
    if (!cursor) return;
    const history = cursors.current[id] ?? [null];
    history.push(cursor);
    cursors.current[id] = history;
    void refresh();
  }
  function back(id: string) {
    cursors.current[id]?.pop();
    void refresh();
  }
  return {
    pages,
    selected: selected?.repository_id === repositoryId ? selected : undefined,
    pending,
    error,
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
    hasPrevious: (id: string) => (cursors.current[id]?.length ?? 0) > 1,
    retry: () => (intent.current ? submit(intent.current) : Promise.resolve()),
  };
}
export type Workspaces = ReturnType<typeof useWorkspaces>;
