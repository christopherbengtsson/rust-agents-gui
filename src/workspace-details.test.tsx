import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { WorkspaceDetails } from "./components/blocks/workspace-details";
import type { Workspace } from "./lib/workspaces";
const id = "00000000-0000-4000-8000-000000000001";
const workspace: Workspace = {
  id,
  repository_id: id,
  creation_action_id: id,
  name: "kyoto",
  branch: "workspace/kyoto",
  path: "/state/workspaces/app/kyoto/checkout",
  base: null,
  artifacts: { branch: "unknown", folder: "unknown" },
  status: { state: "queued" },
  cleanup_unconfirmed: false,
};
afterEach(cleanup);
test("pending allocation reports missing location without inventing a path or showing IDs", () => {
  const view = render(
    <WorkspaceDetails
      workspace={{
        ...workspace,
        name: null,
        branch: null,
        path: null,

        status: { state: "running", stage: "allocating_name" },
      }}
      connected
      stale={false}
      pending={false}
      onReconcile={() => {}}
    />,
  );
  expect(screen.getByText("New workspace")).toBeDefined();
  expect(
    screen.getByText("A name and folder have not been reserved yet."),
  ).toBeDefined();
  expect(view.container.textContent).not.toContain(id);
});

test("workspace facts and recovery have no provider ownership badge", () => {
  const view = render(
    <WorkspaceDetails
      workspace={{
        ...workspace,

        status: {
          state: "failed",
          error: { code: "git", message: "Worktree failed; files retained" },
        },
      }}
      connected
      stale={false}
      pending={false}
      onReconcile={() => {}}
    />,
  );
  expect(screen.getByText(workspace.path!)).toBeDefined();
  expect(screen.getByText(workspace.branch!)).toBeDefined();
  expect(
    screen
      .getByRole("button", { name: "Reconcile workspace" })
      .hasAttribute("disabled"),
  ).toBe(false);
  expect(view.container.textContent).not.toContain("Codex");
});
