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

// One presentation incarnation above both tab panels; no workspace/provider actions.
export function useWorkspaceChanges(
  incarnation: string | undefined,
  workspaceId: string,
  active: boolean,
  visible = true,
  reader: WorkspaceChangesReader,
) {
  // App supplies the lifetime incarnation; view cleanup releases only its interest.
  const view = useInspectionView(reader);
  const shown = useSyncExternalStore(subscribeVisibility, documentVisible);
  useEffect(() => {
    reader.configure({
      incarnation,
      workspaceId,
      active,
      visible: visible && shown,
    });
  }, [reader, incarnation, workspaceId, active, visible, shown]);
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
