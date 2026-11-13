import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, test } from "vitest";
import { WorkspaceDetails } from "./components/blocks/workspace-details";
import type { Workspace } from "./lib/workspaces";
const id = "00000000-0000-4000-8000-000000000001";
const workspace: Workspace = {
  id,
  repository_id: id,
  creation_action_id: id,
  name: "kyoto",
  branch: `workspace/kyoto-${id}`,
  path: `/state/${id}/kyoto-${id}/checkout`,
  legacy: true,
  provider: "codex",
  base: null,
  artifacts: { branch: "unknown", folder: "unknown" },
  status: { state: "queued" },
  cleanup_unconfirmed: false,
};
afterEach(cleanup);
test("legacy UUID locations appear only after explicit disclosure", async () => {
  const user = userEvent.setup();
  const view = render(
    <WorkspaceDetails
      workspace={workspace}
      connected
      stale={false}
      pending={false}
      onReconcile={() => {}}
    />,
  );
  expect(screen.getByText("kyoto")).toBeDefined();
  expect(view.container.textContent).not.toContain(id);
  const trigger = screen.getByRole("button", {
    name: "Show saved branch and folder",
  });
  trigger.focus();
  await user.keyboard("{Enter}");
  expect(screen.getByText(workspace.path!)).toBeDefined();
  expect(trigger.getAttribute("aria-expanded")).toBe("true");
});
test("pending allocation reports missing location without inventing a path or showing IDs", () => {
  const view = render(
    <WorkspaceDetails
      workspace={{
        ...workspace,
        name: null,
        branch: null,
        path: null,
        legacy: false,
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
