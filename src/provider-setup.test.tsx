import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, test, vi } from "vitest";
import { ProviderSetup } from "./components/blocks/provider-setup";
import type { useProviderSetup } from "./hooks/use-provider-setup";
import type {
  ProviderObservation,
  SetupProblem,
  SetupEvidence,
} from "./generated/contracts/types";

const evidence: SetupEvidence = {
  version: "0.169.0",
  authentication: "chatgpt",
  billing: "chatgpt",
  account: "fixture@example.test",
  plan: "pro",
  model: "fixture-model",
  model_provider: "openai",
};
const observed: ProviderObservation = {
  target: {
    id: "00000000-0000-4000-8000-000000000001",
    provider: "codex",
    executable: "/fixture/codex",
    codex_home: "/fixture/home",
    working_directory: "/fixture/neutral",
    supported_version: "0.169.0",
  },
  revision: 2,
  phase: "observed",
  problem: null,
  diagnostic: null,
  login: null,
  evidence,
};
function view(
  patch: Partial<ReturnType<typeof useProviderSetup>> = {},
): ReturnType<typeof useProviderSetup> {
  return {
    target: observed.target,
    observation: observed,
    error: null,
    stale: false,
    connected: true,
    updatesAvailable: true,
    pending: false,
    refresh: vi.fn(async () => {}),
    ...patch,
  };
}
afterEach(cleanup);

test("healthy setup is compact, has a decorative status light, and discloses help and diagnostics by keyboard", async () => {
  const user = userEvent.setup();
  const onOpenHelp = vi.fn();
  render(<ProviderSetup setup={view()} onOpenHelp={onOpenHelp} />);
  const status = screen.getByRole("status");
  expect(status.textContent).toBe("Connected");
  expect(status.getAttribute("data-tone")).toBe("success");
  expect(status.getAttribute("aria-live")).toBe("polite");
  expect(status.querySelector("[aria-hidden=true]")).not.toBeNull();
  expect(screen.getByText("Signed in with ChatGPT")).toBeTruthy();
  expect(screen.getByText("fixture@example.test")).toBeTruthy();
  expect(screen.getByText("Pro")).toBeTruthy();
  expect(
    screen.queryByRole("button", { name: /refresh|check again/i }),
  ).toBeNull();
  expect(screen.queryByText("fixture-model")).toBeNull();
  expect(
    screen.queryByRole("button", { name: "Official sign-in instructions" }),
  ).toBeNull();
  const details = screen.getByRole("button", { name: "Technical details" });
  details.focus();
  await user.keyboard("{Enter}");
  expect(details.getAttribute("aria-expanded")).toBe("true");
  expect(screen.getByText("fixture-model")).toBeTruthy();
  expect(screen.getByText("/fixture/codex")).toBeTruthy();
  await user.click(screen.getByRole("button", { name: "Setup help" }));
  await user.click(
    screen.getByRole("button", { name: "Official sign-in instructions" }),
  );
  expect(onOpenHelp).toHaveBeenCalledExactlyOnceWith("authentication");
  expect(
    screen.queryByRole("button", { name: /refresh|check again/i }),
  ).toBeNull();
});

test.each([
  { pending: true, stale: false },
  { pending: false, stale: true },
  { pending: false, stale: true, connected: false },
  {
    pending: false,
    stale: true,
    error: new Error("controlled transport failure"),
  },
])("unverified evidence cannot retain a green Connected light: %j", (patch) => {
  render(<ProviderSetup setup={view(patch)} onOpenHelp={vi.fn()} />);
  expect(screen.getByRole("status").getAttribute("data-tone")).not.toBe(
    "success",
  );
  expect(screen.getByRole("status").textContent).not.toBe("Connected");
  expect(
    screen.getByText("Last known · current readiness unverified"),
  ).toBeTruthy();
  expect(screen.getByText("fixture@example.test")).toBeTruthy();
  if (patch.pending || patch.connected === false)
    expect(screen.queryByRole("button", { name: "Check again" })).toBeNull();
});

test.each<SetupProblem>([
  "missing_executable",
  "incompatible_protocol",
  "authentication_unavailable",
  "status_read_failed",
])(
  "%s has contextual recheck and recovery without opening details",
  async (problem) => {
    const setup = view({ stale: true, observation: { ...observed, problem } });
    render(<ProviderSetup setup={setup} onOpenHelp={vi.fn()} />);
    expect(screen.getByRole("alert")).toBeTruthy();
    expect(screen.getByRole("status").getAttribute("data-tone")).not.toBe(
      "success",
    );
    await userEvent
      .setup()
      .click(screen.getByRole("button", { name: "Check again" }));
    expect(setup.refresh).toHaveBeenCalledTimes(1);
  },
);

test("uncertain cleanup explains why a replacement check is unavailable", () => {
  render(
    <ProviderSetup
      setup={view({
        stale: true,
        observation: {
          ...observed,
          phase: "cleanup_uncertain",
          problem: "cleanup_uncertain",
        },
      })}
      onOpenHelp={vi.fn()}
    />,
  );
  expect(screen.getByRole("status").textContent).toBe("Cleanup unconfirmed");
  expect(screen.getByText(/Another check is blocked/)).toBeTruthy();
  expect(screen.queryByRole("button", { name: "Check again" })).toBeNull();
});

test("a read failure cannot remove the retained cleanup fence", () => {
  render(
    <ProviderSetup
      setup={view({
        stale: true,
        pending: true,
        error: new Error("Controlled status read failure"),
        observation: {
          ...observed,
          phase: "cleanup_uncertain",
          problem: "cleanup_uncertain",
        },
      })}
      onOpenHelp={vi.fn()}
    />,
  );
  expect(screen.getByRole("status").textContent).toBe("Cleanup unconfirmed");
  expect(screen.getByRole("alert").textContent).toContain(
    "Another check is blocked",
  );
  expect(screen.queryByRole("button", { name: "Check again" })).toBeNull();
});

test.each([false, true])(
  "transport failure overrides retained checking; connected=%s",
  (connected) => {
    render(
      <ProviderSetup
        setup={view({
          connected,
          pending: !connected,
          stale: true,
          error: connected ? new Error("controlled observation failure") : null,
          observation: { ...observed, phase: "checking" },
        })}
        onOpenHelp={vi.fn()}
      />,
    );
    expect(screen.getByRole("status").textContent).toBe("Status unavailable");
    expect(screen.getByRole("alert").textContent).toContain(
      connected ? "Codex setup could not be checked" : "Supervisor unavailable",
    );
    expect(screen.getByRole("status").getAttribute("data-tone")).not.toBe(
      "success",
    );
    if (connected)
      expect(screen.getByRole("button", { name: "Check again" })).toBeTruthy();
    else
      expect(screen.queryByRole("button", { name: "Check again" })).toBeNull();
  },
);

test("unknown authentication and billing are explicit and never appear connected", () => {
  render(
    <ProviderSetup
      setup={view({
        observation: {
          ...observed,
          evidence: {
            ...evidence,
            authentication: "unknown",
            billing: "unknown",
            account: null,
            plan: null,
          },
        },
      })}
      onOpenHelp={vi.fn()}
    />,
  );
  expect(screen.getByRole("status").textContent).toBe("Readiness unknown");
  expect(screen.getByText("Authentication unknown")).toBeTruthy();
  expect(screen.getByText("Unknown")).toBeTruthy();
});

test("stopped observation delivery exposes supervisor reconnection guidance instead of retained checking", () => {
  render(
    <ProviderSetup
      setup={view({
        updatesAvailable: false,
        stale: true,
        pending: false,
        observation: { ...observed, phase: "checking" },
      })}
      onOpenHelp={vi.fn()}
    />,
  );
  expect(screen.getByRole("status").textContent).toBe("Status unavailable");
  expect(screen.getByRole("alert").textContent).toContain(
    "Live setup updates unavailable",
  );
  expect(screen.getByRole("alert").textContent).toContain(
    "supervisor connection",
  );
  expect(screen.queryByRole("button", { name: "Check again" })).toBeNull();
});
