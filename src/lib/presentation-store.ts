import { createStore, type StateCreator } from "zustand/vanilla";
import { persist, type PersistStorage } from "zustand/middleware";
import { savedDestination, type MainDestination } from "./navigation";
import { identity, object } from "./observation-contracts";
import type { ChangedFile } from "./changes";

export const PRESENTATION_KEY = "agent-workspace.presentation";
export type ReadingPosition = { top: number; left: number };
export type InspectionSelection = Pick<ChangedFile, "id" | "path"> & {
  comparison?: string;
  position?: ReadingPosition;
};
export type WorkspacePresentation = {
  view: "workspace" | "changes";
  filesWidth: number;
  collapsed: boolean;
};
export const defaultWorkspacePresentation: WorkspacePresentation = {
  view: "workspace",
  filesWidth: 32,
  collapsed: false,
};
type Preferences = {
  rememberedMain?: MainDestination;
  inspection: Record<string, InspectionSelection | undefined>;
  contexts: Record<string, WorkspacePresentation | undefined>;
};
type Presentation = Preferences & {
  rememberMain: (destination: MainDestination) => void;
  storageError?: string;
  rememberFile: (
    workspace: string,
    file: InspectionSelection | undefined,
  ) => void;
  rememberPosition: (
    workspace: string,
    comparison: string,
    file: string,
    position: ReadingPosition,
  ) => void;
  rememberLayout: (
    workspace: string,
    layout: Pick<WorkspacePresentation, "filesWidth" | "collapsed">,
  ) => void;
  rememberView: (
    workspace: string,
    view: WorkspacePresentation["view"],
  ) => void;
};
const nonnegative = (value: unknown): value is number =>
  typeof value === "number" && Number.isFinite(value) && value >= 0;
function position(value: unknown): value is ReadingPosition {
  return object(value) && nonnegative(value.top) && nonnegative(value.left);
}
function selection(value: unknown): InspectionSelection | undefined {
  if (
    !object(value) ||
    !identity(value.id) ||
    !identity(value.comparison) ||
    typeof value.path !== "string" ||
    !value.path ||
    new TextEncoder().encode(value.path).length > 4096 ||
    (value.position !== undefined && !position(value.position))
  )
    return;
  return {
    id: value.id,
    path: value.path,
    comparison: value.comparison,
    ...(value.position === undefined
      ? {}
      : { position: { top: value.position.top, left: value.position.left } }),
  };
}
// Decode fields explicitly on both hydration and serialization. Unknown payloads
// cannot introduce observations, executable actions or middleware methods.
function preferences(value: unknown): Preferences {
  if (
    !object(value) ||
    !object(value.inspection) ||
    (value.contexts !== undefined && !object(value.contexts))
  )
    throw new Error("Invalid presentation preferences");
  const inspection: Preferences["inspection"] = {};
  for (const [workspace, saved] of Object.entries(value.inspection)) {
    if (!identity(workspace))
      throw new Error("Invalid saved workspace identity");
    if (saved === undefined) continue;
    const file = selection(saved);
    if (!file) throw new Error("Invalid saved file context");
    inspection[workspace] = file;
  }
  const contexts: Preferences["contexts"] = {};
  for (const [workspace, saved] of Object.entries(value.contexts ?? {})) {
    if (!identity(workspace))
      throw new Error("Invalid saved workspace identity");
    if (saved === undefined) continue;
    if (
      !object(saved) ||
      (saved.view !== "workspace" && saved.view !== "changes") ||
      !nonnegative(saved.filesWidth) ||
      saved.filesWidth === 0 ||
      saved.filesWidth > 55 ||
      typeof saved.collapsed !== "boolean"
    )
      throw new Error("Invalid saved panel context");
    contexts[workspace] = {
      view: saved.view,
      filesWidth: saved.filesWidth,
      collapsed: saved.collapsed,
    };
  }
  const rememberedMain = savedDestination(value.rememberedMain);
  if (value.rememberedMain !== undefined && !rememberedMain)
    throw new Error("Invalid remembered destination");
  return {
    inspection,
    contexts,
    ...(rememberedMain ? { rememberedMain } : {}),
  };
}

export function createPresentationStore(
  getStorage: () => Pick<Storage, "getItem" | "setItem" | "removeItem"> = () =>
    localStorage,
) {
  let reportError = (_message: string) => {};
  let writable = true;
  const storage: PersistStorage<Preferences> = {
    getItem: (key) => {
      const raw = getStorage().getItem(key);
      if (raw === null)
        return { version: 2, state: { inspection: {}, contexts: {} } };
      const envelope: unknown = JSON.parse(raw);
      if (
        !object(envelope) ||
        envelope.version !== 2 ||
        !object(envelope.state)
      )
        throw new Error("Invalid presentation record");
      const decoded = preferences(envelope.state);
      return { version: envelope.version, state: decoded };
    },
    setItem: (key, value) => {
      if (!writable) return;
      try {
        getStorage().setItem(key, JSON.stringify(value));
      } catch {
        reportError(
          "The desktop could not save presentation preferences. Your current context remains available in this window; check local storage before reopening.",
        );
      }
    },
    removeItem: (key) => {
      if (!writable) return;
      try {
        getStorage().removeItem(key);
      } catch {
        reportError(
          "The desktop could not remove presentation preferences. Check local storage before reopening.",
        );
      }
    },
  };
  const persisted = persist<Presentation, [], [], Preferences>(
    (set, get) => ({
      rememberMain: (destination) => {
        const saved = savedDestination(destination);
        if (
          saved &&
          JSON.stringify(saved) !== JSON.stringify(get().rememberedMain)
        )
          set({ rememberedMain: saved });
      },
      inspection: {},
      contexts: {},
      rememberFile: (workspace, file) => {
        const previous = get().inspection[workspace];
        if (
          !identity(workspace) ||
          previous === file ||
          (file &&
            previous?.id === file.id &&
            previous.comparison === file.comparison)
        )
          return;
        const saved = file ? selection(file) : undefined;
        set((state) => ({
          inspection: { ...state.inspection, [workspace]: saved },
        }));
      },
      rememberPosition: (workspace, comparison, file, reading) => {
        const selected = get().inspection[workspace];
        if (
          !selected ||
          selected.comparison !== comparison ||
          selected.id !== file ||
          !position(reading) ||
          (selected.position?.top === reading.top &&
            selected.position.left === reading.left)
        )
          return;
        set((state) => ({
          inspection: {
            ...state.inspection,
            [workspace]: {
              ...selected,
              position: { top: reading.top, left: reading.left },
            },
          },
        }));
      },
      rememberLayout: (workspace, layout) => {
        const previous =
          get().contexts[workspace] ?? defaultWorkspacePresentation;
        if (
          !identity(workspace) ||
          !nonnegative(layout.filesWidth) ||
          layout.filesWidth === 0 ||
          layout.filesWidth > 55 ||
          (previous.filesWidth === layout.filesWidth &&
            previous.collapsed === layout.collapsed)
        )
          return;
        set((state) => ({
          contexts: {
            ...state.contexts,
            [workspace]: {
              ...previous,
              filesWidth: layout.filesWidth,
              collapsed: layout.collapsed,
            },
          },
        }));
      },
      rememberView: (workspace, view) => {
        const previous =
          get().contexts[workspace] ?? defaultWorkspacePresentation;
        if (!identity(workspace) || previous.view === view) return;
        set((state) => ({
          contexts: { ...state.contexts, [workspace]: { ...previous, view } },
        }));
      },
    }),
    {
      name: PRESENTATION_KEY,
      version: 2,
      storage,
      skipHydration: true,
      partialize: (state) =>
        preferences({
          inspection: Object.fromEntries(
            Object.entries(state.inspection).filter(([, file]) =>
              selection(file),
            ),
          ),
          contexts: state.contexts,
          ...(state.rememberedMain
            ? { rememberedMain: state.rememberedMain }
            : {}),
        }),
      merge: (saved, current) =>
        saved === undefined ? current : { ...current, ...preferences(saved) },
      onRehydrateStorage: () => (_state, error) => {
        if (error) {
          writable = false; // Preserve unreadable/future data instead of overwriting it.
          reportError(
            "The desktop could not read presentation preferences. Use fresh private development state. Using defaults in this window; the saved record has been preserved.",
          );
        }
      },
    },
  );
  const initialize: StateCreator<
    Presentation,
    [],
    [["zustand/persist", Preferences]]
  > = (set, get, api) => {
    // Raw store set bypasses persistence: reporting a failed write cannot trigger
    // another write, and hydration errors never overwrite the unreadable record.
    reportError = (storageError) => set({ storageError });
    return persisted(set, get, api);
  };
  const store = createStore<Presentation>()(initialize);
  void store.persist.rehydrate();
  return store;
}
