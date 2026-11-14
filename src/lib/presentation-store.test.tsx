import { expect, test } from "vitest";
import {
  createPresentationStore,
  PRESENTATION_KEY,
} from "./presentation-store";

const workspace = "00000000-0000-4000-8000-000000000001";
const comparison = "00000000-0000-4000-8000-000000000002";
const file = "00000000-0000-4000-8000-000000000003";
function memory() {
  const values = new Map<string, string>();
  return {
    values,
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => {
      values.set(key, value);
    },
    removeItem: (key: string) => {
      values.delete(key);
    },
  };
}

test("reopening restores only allowlisted workspace presentation context", () => {
  const storage = memory();
  const store = createPresentationStore(() => storage);
  store
    .getState()
    .rememberFile(workspace, { id: file, path: "src/a.ts", comparison });
  store
    .getState()
    .rememberPosition(workspace, comparison, file, { top: 37, left: 12 });
  store
    .getState()
    .rememberLayout(workspace, { filesWidth: 41, collapsed: true });
  store.getState().rememberView(workspace, "changes");
  const reopened = createPresentationStore(() => storage);
  expect(reopened.getState().inspection[workspace]).toEqual({
    id: file,
    path: "src/a.ts",
    comparison,
    position: { top: 37, left: 12 },
  });
  expect(reopened.getState().contexts[workspace]).toEqual({
    view: "changes",
    filesWidth: 41,
    collapsed: true,
  });
  const written: unknown = JSON.parse(
    storage.getItem(PRESENTATION_KEY) ?? "null",
  );
  expect(written).toEqual({
    version: 2,
    state: {
      inspection: {
        [workspace]: {
          id: file,
          path: "src/a.ts",
          comparison,
          position: { top: 37, left: 12 },
        },
      },
      contexts: {
        [workspace]: { view: "changes", filesWidth: 41, collapsed: true },
      },
    },
  });
});

test("version-zero presentation migrates without touching pending identities or old navigation keys", () => {
  const storage = memory();
  storage.setItem(
    "agent-workspace.pending-registration",
    '{"action_id":"keep-registration","uncertain":true}',
  );
  storage.setItem(
    "agent-workspace.pending-workspace",
    '{"action_id":"keep-workspace","uncertain":true}',
  );
  storage.setItem("agent-workspace.selected-repository", file);
  storage.setItem(
    PRESENTATION_KEY,
    JSON.stringify({
      version: 0,
      state: {
        inspection: { [workspace]: { id: file, path: "a.ts", comparison } },
        credentials: "excluded",
        preview: { rows: ["excluded"] },
        pending: { uncertain: false },
      },
    }),
  );
  const before = new Map(storage.values);
  const store = createPresentationStore(() => storage);
  expect(store.getState().inspection[workspace]?.id).toBe(file);
  expect(store.getState().contexts).toEqual({});
  expect(store.getState().storageError).toBeUndefined();
  for (const [key, value] of before)
    if (key !== PRESENTATION_KEY) expect(storage.getItem(key)).toBe(value);
  expect(JSON.parse(storage.getItem(PRESENTATION_KEY) ?? "null")).toEqual({
    version: 2,
    state: {
      inspection: { [workspace]: { id: file, path: "a.ts", comparison } },
      contexts: {},
      rememberedMain: { kind: "repository", repositoryId: file },
    },
  });
});

test.each([
  "{",
  JSON.stringify({ version: 9, state: { inspection: {}, contexts: {} } }),
  JSON.stringify({ state: { inspection: {} } }),
  JSON.stringify({
    version: 2,
    state: {
      inspection: { [workspace]: { id: "bad", path: "a", comparison } },
    },
  }),
  JSON.stringify({
    version: 2,
    state: {
      inspection: {
        [workspace]: {
          id: file,
          path: "a",
          comparison,
          position: { top: -1, left: 0 },
        },
      },
    },
  }),
  JSON.stringify({
    version: 2,
    state: {
      inspection: {},
      contexts: {
        [workspace]: { view: "execute", filesWidth: 32, collapsed: false },
      },
    },
  }),
  JSON.stringify({
    version: 2,
    state: {
      inspection: {},
      contexts: {
        [workspace]: { view: "changes", filesWidth: 99, collapsed: false },
      },
    },
  }),
])(
  "unreadable preferences remain preserved while in-memory context stays usable: %s",
  (raw) => {
    const storage = memory();
    storage.setItem(PRESENTATION_KEY, raw);
    const store = createPresentationStore(() => storage);
    expect(store.getState().storageError).toMatch(/read or migrate/);
    expect(store.getState().inspection).toEqual({});
    store.getState().rememberView(workspace, "changes");
    expect(store.getState().contexts[workspace]?.view).toBe("changes");
    expect(storage.getItem(PRESENTATION_KEY)).toBe(raw);
  },
);

test("storage acquisition/read/write failures are visible and cannot recurse or interrupt interaction", () => {
  const unavailable = createPresentationStore(() => {
    throw new Error("denied");
  });
  expect(unavailable.getState().storageError).toMatch(/read or migrate/);
  const denied = createPresentationStore(() => ({
    getItem: () => {
      throw new Error("read denied");
    },
    setItem: () => {
      throw new Error("must not overwrite");
    },
    removeItem: () => {
      throw new Error("must not remove");
    },
  }));
  expect(denied.getState().storageError).toMatch(/read or migrate/);
  let writes = 0;
  const full = createPresentationStore(() => ({
    getItem: () => null,
    setItem: () => {
      writes++;
      throw new Error("quota");
    },
    removeItem: () => {
      throw new Error("must not remove");
    },
  }));
  expect(() =>
    full.getState().rememberView(workspace, "changes"),
  ).not.toThrow();
  expect(full.getState().storageError).toMatch(/could not save/);
  expect(full.getState().contexts[workspace]?.view).toBe("changes");
  expect(writes).toBe(1);
});

test("late reading callbacks cannot replace the selected file's reading position", () => {
  const store = createPresentationStore(() => memory());
  store.getState().rememberFile(workspace, { id: file, path: "a", comparison });
  store
    .getState()
    .rememberPosition(workspace, workspace, file, { top: 99, left: 0 });
  store
    .getState()
    .rememberPosition(workspace, comparison, workspace, { top: 99, left: 0 });
  expect(store.getState().inspection[workspace]?.position).toBeUndefined();
});

test("reselecting the same comparison and file preserves reading context", () => {
  const storage = memory();
  const store = createPresentationStore(() => storage);
  store.getState().rememberFile(workspace, { id: file, path: "a", comparison });
  store
    .getState()
    .rememberPosition(workspace, comparison, file, { top: 37, left: 12 });
  store.getState().rememberFile(workspace, { id: file, path: "a", comparison });
  expect(store.getState().inspection[workspace]?.position).toEqual({
    top: 37,
    left: 12,
  });
});

test.each([0, 9])(
  "preference migration version %i preserves usable pending action payloads and sticky uncertainty",
  async (version) => {
    const { readPending, PENDING_KEY } = await import("./repositories");
    const { readWorkspacePending, WORKSPACE_PENDING } =
      await import("./workspaces");
    localStorage.clear();
    const registration = {
      request: {
        action_id: file,
        path: "/fixture/repository",
        create_initial_workspace: true,
      },
      selection: null,
      uncertain: true,
    };
    const creation = {
      request: {
        operation: "create",
        action_id: comparison,
        repository_id: workspace,
      },
      uncertain: true,
    };
    localStorage.setItem(PENDING_KEY, JSON.stringify(registration));
    localStorage.setItem(WORKSPACE_PENDING, JSON.stringify(creation));
    localStorage.setItem(
      PRESENTATION_KEY,
      JSON.stringify({ version, state: { inspection: {} } }),
    );
    const store = createPresentationStore();
    store.getState().rememberView(workspace, "changes");
    expect(readPending()).toEqual(registration);
    expect(readWorkspacePending()).toEqual(creation);
    expect(localStorage.getItem(PENDING_KEY)).toBe(
      JSON.stringify(registration),
    );
    expect(localStorage.getItem(WORKSPACE_PENDING)).toBe(
      JSON.stringify(creation),
    );
    localStorage.clear();
  },
);

test("legacy selection becomes a restoration default and preserves pending bytes", () => {
  const storage = memory();
  storage.setItem("agent-workspace.selected-repository", file);
  storage.setItem(
    "agent-workspace.selected-workspace",
    JSON.stringify({ repository_id: file, workspace_id: workspace }),
  );
  storage.setItem("agent-workspace.pending-workspace", "retained bytes");
  const store = createPresentationStore(() => storage);
  expect(store.getState().rememberedMain).toEqual({
    kind: "workspace",
    repositoryId: file,
    workspaceId: workspace,
    view: "workspace",
  });
  store.getState().rememberMain({ kind: "repository", repositoryId: file });
  expect(
    createPresentationStore(() => storage).getState().rememberedMain,
  ).toEqual({ kind: "repository", repositoryId: file });
  expect(storage.getItem("agent-workspace.pending-workspace")).toBe(
    "retained bytes",
  );
});
