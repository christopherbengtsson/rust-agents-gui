import { useCallback, useEffect, useRef, useState } from "react";
import { workspaceRequest } from "@/lib/workspaces";
import {
  pendingInspection,
  type ChangedFile,
  type ChangesObservation,
  type PreviewObservation,
} from "@/lib/changes";
import { completeOperation } from "@/lib/complete-operation";
import { repositoryError } from "@/lib/repositories";

// One owner above the tab panels. This slice polls admitted jobs, never edits.
export function useWorkspaceChanges(
  owner: string | undefined,
  workspaceId: string,
  active: boolean,
) {
  const [observation, setObservation] = useState<ChangesObservation>();
  const [preview, setPreview] = useState<PreviewObservation>();
  const [selected, setSelected] = useState<Pick<ChangedFile, "id" | "path">>();
  const [error, setError] = useState<string>();
  const [loading, setLoading] = useState(false);
  const selection = useRef(selected);
  const page = useRef<{ comparison_id: string | null; after: string | null }>({
    comparison_id: null,
    after: null,
  });
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const lifetime = useRef(0);
  const mounted = useRef(false);
  const reading = useRef<symbol | null>(null);
  const queued = useRef(false);
  const loaded = useRef<string | undefined>(undefined);

  const read = useCallback(
    async function readChanges(refresh = false): Promise<void> {
      if (!owner || !mounted.current) return;
      if (reading.current) {
        queued.current = true;
        return;
      }
      clearTimeout(timer.current);
      const token = Symbol();
      reading.current = token;
      const generation = lifetime.current;
      const current = () =>
        mounted.current &&
        lifetime.current === generation &&
        reading.current === token;
      setLoading(true);
      if (refresh) page.current = { comparison_id: null, after: null };
      let pending = false;
      return completeOperation(
        async () => {
          const result = await workspaceRequest({
            operation: "changes",
            workspace_id: workspaceId,
            refresh,
            ...page.current,
          });
          if (!current() || result.kind !== "changes") return;
          const data = result.observation;
          setObservation(data);
          setError(undefined);
          pending = pendingInspection(data.phase);
          const file = selection.current;
          if (active && file && data.page && !pending) {
            // The server keeps stable file IDs for paths that survive a refresh.
            const target = {
              workspace_id: workspaceId,
              comparison_id: data.page.comparison_id,
              file_id: file.id,
            };
            const key = `${target.comparison_id}/${target.file_id}`;
            const value = await workspaceRequest({
              operation: "change_preview",
              target,
              load: loaded.current !== key,
            });
            if (
              !current() ||
              selection.current?.id !== file.id ||
              value.kind !== "change_preview"
            )
              return;
            loaded.current = key;
            setPreview(value.observation);
            pending = pending || pendingInspection(value.observation.phase);
          }
        },
        (failure) => {
          if (current()) setError(repositoryError(failure).message);
        },
        () => {
          if (!current()) return;
          reading.current = null;
          setLoading(false);
          if (queued.current) {
            queued.current = false;
            void readChanges();
          } else if (pending)
            timer.current = setTimeout(() => void readChanges(), 250);
        },
      );
    },
    [owner, workspaceId, active],
  );

  useEffect(() => {
    mounted.current = true;
    void read(); // Reconnect reads authoritative observations; no refresh replay.
    return () => {
      mounted.current = false;
      lifetime.current += 1;
      reading.current = null;
      queued.current = false;
      clearTimeout(timer.current);
    };
  }, [read]);
  const select = (file: ChangedFile) => {
    selection.current = { id: file.id, path: file.path };
    setSelected(selection.current);
    loaded.current = undefined;
    setPreview(undefined);
    void read();
  };
  const refresh = () => {
    loaded.current = undefined;
    return read(true);
  };
  const first = () => {
    page.current = { comparison_id: null, after: null };
    return read();
  };
  const next = () => {
    if (!observation?.page?.next) return;
    page.current = {
      comparison_id: observation.page.comparison_id,
      after: observation.page.next,
    };
    return read();
  };
  return {
    observation,
    preview,
    selected,
    loading,
    error,
    select,
    refresh,
    first,
    next,
  };
}
export type WorkspaceChanges = ReturnType<typeof useWorkspaceChanges>;
