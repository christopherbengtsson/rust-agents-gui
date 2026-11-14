import { useEffect, useSyncExternalStore } from "react";
import {
  WorkspaceChangesReader,
  useInspectionView,
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
  reader: WorkspaceChangesReader,
) {
  // App supplies the lifetime owner; view cleanup releases only its interest.
  const view = useInspectionView(reader);
  const shown = useSyncExternalStore(subscribeVisibility, documentVisible);
  useEffect(() => {
    reader.configure({ owner, workspaceId, active, visible: visible && shown });
  }, [reader, owner, workspaceId, active, visible, shown]);
  useEffect(() => () => reader.release(), [reader]);
  return {
    ...view,
    select: reader.select,
    refresh: reader.refresh,
    first: reader.first,
    next: reader.next,
  };
}
export type WorkspaceChanges = ReturnType<typeof useWorkspaceChanges>;
