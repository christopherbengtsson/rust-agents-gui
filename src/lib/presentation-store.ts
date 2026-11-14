import { createStore } from "zustand/vanilla";
import { readSelection } from "./repositories";
import { readWorkspaceSelection } from "./workspaces";
import type { ChangedFile } from "./changes";

export type InspectionSelection = Pick<ChangedFile, "id" | "path"> & {
  comparison?: string;
};
// Presentation memory only. F05 owns validated durable preference persistence.
export function createPresentationStore() {
  return createStore<{
    repository: string | null;
    workspace: ReturnType<typeof readWorkspaceSelection>;
    inspection: Record<string, InspectionSelection | undefined>;
  }>(() => ({
    repository: readSelection(),
    workspace: readWorkspaceSelection(),
    inspection: {},
  }));
}
