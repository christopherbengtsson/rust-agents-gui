import { useEffect, useState, useSyncExternalStore } from "react";
import {
  WorkspaceChangesReader,
  type ChangesView,
} from "@/lib/workspace-changes-reader";

function subscribeVisibility(changed: () => void) {
  document.addEventListener("visibilitychange", changed);
  return () => document.removeEventListener("visibilitychange", changed);
}
const documentVisible = () => document.visibilityState !== "hidden";

// One presentation owner above both tab panels; no workspace/provider actions.
export function useWorkspaceChanges(
  owner: string | undefined,
  workspaceId: string,
  active: boolean,
  visible = true,
  sharedReader?: WorkspaceChangesReader,
) {
  const [view, setView] = useState<ChangesView>({ loading: false });
  // WorkspacePanel supplies a stable lane above keyed presentation resets.
  const [reader] = useState(() => sharedReader ?? new WorkspaceChangesReader());
  const shown = useSyncExternalStore(subscribeVisibility, documentVisible);
  useEffect(() => {
    reader.configure(
      { owner, workspaceId, active, visible: visible && shown },
      setView,
    );
  }, [reader, owner, workspaceId, active, visible, shown]);
  useEffect(() => () => reader.dispose(), [reader]);
  return {
    ...view,
    select: reader.select,
    refresh: reader.refresh,
    first: reader.first,
    next: reader.next,
  };
}
export type WorkspaceChanges = ReturnType<typeof useWorkspaceChanges>;
