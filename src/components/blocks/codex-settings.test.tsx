import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, test, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { CodexSettings } from "./codex-settings";
import type { Runtimes } from "@/hooks/use-runtimes";
import type { RuntimeReply } from "@/lib/runtimes";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(cleanup);
const observation: RuntimeReply = {
  target: { owner: "owner", runtime: "codex" },
  configuration: { revision: 1, executable: null },
  busy: false,
  active: null,
  lookup: null,
  diagnostic: null,
  blocked: null,
};
function runtime(): Runtimes {
  return {
    observation,
    pending: null,
    error: undefined,
    actionError: undefined,
    preferenceError: undefined,
    loading: false,
    sending: false,
    stale: false,
    canAct: true,
    refresh: vi.fn(),
    save: vi.fn(),
    check: vi.fn(),
  };
}
test("unsaved edits disable Check and Save settlement preserves newer input", async () => {
  const user = userEvent.setup();
  const r = runtime();
  const view = render(<CodexSettings runtimes={r} />);
  const input = screen.getByRole("textbox", { name: "Codex executable" });
  await user.type(input, "/first/codex");
  expect(
    screen
      .getByRole("button", { name: "Check setup" })
      .hasAttribute("disabled"),
  ).toBe(true);
  await user.click(screen.getByRole("button", { name: "Save" }));
  expect(r.save).toHaveBeenCalledWith("/first/codex");
  await user.clear(input);
  await user.type(input, "/newer/codex");
  view.rerender(
    <CodexSettings
      runtimes={{
        ...r,
        observation: {
          ...observation,
          configuration: { revision: 2, executable: "/first/codex" },
        },
      }}
    />,
  );
  expect((input as HTMLInputElement).value).toBe("/newer/codex");
});

test("leaving Settings discards only its unsaved draft", async () => {
  const user = userEvent.setup();
  const r = runtime();
  const view = render(<CodexSettings runtimes={r} />);
  await user.type(
    screen.getByRole("textbox", { name: "Codex executable" }),
    "/unsaved/codex",
  );
  view.unmount();
  render(<CodexSettings runtimes={r} />);
  expect(
    (
      screen.getByRole("textbox", {
        name: "Codex executable",
      }) as HTMLInputElement
    ).value,
  ).toBe("");
  expect(r.save).not.toHaveBeenCalled();
  expect(r.check).not.toHaveBeenCalled();
});

test("copy and scoped official documentation failures remain visible", async () => {
  const user = userEvent.setup();
  vi.mocked(invoke).mockRejectedValueOnce("open failed");
  render(<CodexSettings runtimes={runtime()} />);
  await user.click(
    screen.getByRole("link", { name: "Official installation instructions" }),
  );
  expect(
    await screen.findByText(/Could not open the official instructions/),
  ).toBeDefined();
  expect(invoke).toHaveBeenCalledWith("open_runtime_document", {
    document: "install",
  });
  vi.spyOn(navigator.clipboard, "writeText").mockRejectedValueOnce(
    new Error("denied"),
  );
  await user.click(screen.getByRole("button", { name: "Copy command" }));
  expect(await screen.findByText(/Could not copy the command/)).toBeDefined();
});
