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
    selected_executable: null,
    discovery_source: "fixture",
    codex_home: "/fixture/home",
    working_directory: "/fixture/neutral",
    minimum_supported_version: "0.169.0",
  },
  selection: null,
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

test("verified connection shows only status and identity, without setup controls or billing", () => {
  render(
    <ProviderSetup
      setup={view()}
      onOpenHelp={vi.fn()}
      signIn={<button>Sign in</button>}
      helpError="Earlier help failure"
      locateError="Earlier chooser failure"
    />,
  );
  expect(screen.getByRole("status").textContent).toBe("Connected");
  expect(screen.getByRole("status").getAttribute("data-tone")).toBe("success");
  expect(screen.getByText("fixture@example.test")).toBeTruthy();
  for (const label of [
    "Sign in",
    "Technical details",
    "Setup help",
    "Check again",
  ])
    expect(screen.queryByRole("button", { name: label })).toBeNull();
  expect(screen.queryByText("Plan")).toBeNull();
  expect(screen.queryByText("Billing")).toBeNull();
  expect(screen.queryByRole("alert")).toBeNull();
});

test("existing API key is Connected without setup controls", () => {
  render(
    <ProviderSetup
      setup={view({
        observation: {
          ...observed,
          evidence: { ...evidence, authentication: "api_key", account: null },
        },
      })}
      onOpenHelp={vi.fn()}
    />,
  );
  expect(screen.getByRole("status").textContent).toBe(
    "Connected · Using API key",
  );
  expect(screen.queryByRole("button")).toBeNull();
});

test("checking replaces the entire body and returns explicit-action focus to the resulting status", async () => {
  const user = userEvent.setup();
  const failed = view({
    stale: true,
    observation: { ...observed, problem: "status_read_failed" },
  });
  const { rerender } = render(
    <ProviderSetup setup={failed} onOpenHelp={vi.fn()} />,
  );
  await user.click(screen.getByRole("button", { name: "Check again" }));
  expect(failed.refresh).toHaveBeenCalledTimes(1);
  rerender(
    <ProviderSetup setup={view({ pending: true })} onOpenHelp={vi.fn()} />,
  );
  expect(screen.getByRole("status").textContent).toBe("Checking Codex…");
  expect(screen.queryByText("fixture@example.test")).toBeNull();
  expect(screen.queryByRole("button")).toBeNull();
  rerender(<ProviderSetup setup={view()} onOpenHelp={vi.fn()} />);
  expect(document.activeElement?.contains(screen.getByRole("status"))).toBe(
    true,
  );
});

test.each([
  "status_read_failed",
  "authentication_unavailable",
  "missing_executable",
  "incompatible_protocol",
  "selection_failed",
] satisfies SetupProblem[])(
  "%s has truthful tone and relevant recovery",
  async (problem) => {
    const setup = view({ stale: true, observation: { ...observed, problem } });
    render(<ProviderSetup setup={setup} onOpenHelp={vi.fn()} />);
    const red = [
      "missing_executable",
      "incompatible_protocol",
      "selection_failed",
    ].includes(problem);
    expect(screen.getByRole("status").getAttribute("data-tone")).toBe(
      red ? "error" : "warning",
    );
    expect(screen.getByText(/Last verified account:/)).toBeTruthy();
    if (problem === "authentication_unavailable") {
      expect(screen.getByRole("status").textContent).toBe("Not connected");
    } else {
      await userEvent
        .setup()
        .click(screen.getByRole("button", { name: "Check again" }));
      expect(setup.refresh).toHaveBeenCalledTimes(1);
    }
  },
);

test("missing runtime explains lookup failure and discloses native locating only on request", async () => {
  const locate = vi.fn(async () => {});
  const help = vi.fn();
  render(
    <ProviderSetup
      setup={view({
        stale: true,
        observation: { ...observed, problem: "missing_executable" },
      })}
      onOpenHelp={help}
      onLocate={locate}
    />,
  );
  expect(screen.getByRole("status").textContent).toBe("Codex not found");
  expect(screen.queryByRole("button", { name: "Locate Codex…" })).toBeNull();
  await userEvent
    .setup()
    .click(screen.getByRole("button", { name: "Already installed?" }));
  await userEvent
    .setup()
    .click(screen.getByRole("button", { name: "Locate Codex…" }));
  expect(locate).toHaveBeenCalledTimes(1);
  await userEvent
    .setup()
    .click(screen.getByRole("button", { name: "Installation guide" }));
  expect(help).toHaveBeenCalledExactlyOnceWith("install");
});

test("compatibility recovery retains minimum and observed release diagnostics", async () => {
  render(
    <ProviderSetup
      setup={view({
        stale: true,
        observation: {
          ...observed,
          problem: "incompatible_protocol",
          evidence: { ...evidence, version: "0.170.0" },
        },
      })}
      onOpenHelp={vi.fn()}
    />,
  );
  await userEvent
    .setup()
    .click(screen.getByRole("button", { name: "Technical details" }));
  expect(
    screen.getByText("Observed runtime").nextElementSibling?.textContent,
  ).toBe("0.170.0");
  expect(
    screen.getByText("Minimum runtime").nextElementSibling?.textContent,
  ).toBe("0.169.0");
});

test.each([false, true])(
  "cleanup fence survives a failed read and pending feedback; error=%s",
  (failure) => {
    render(
      <ProviderSetup
        setup={view({
          stale: true,
          pending: true,
          error: failure ? new Error("controlled") : null,
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
    expect(screen.getByRole("alert").textContent).toContain("blocked");
    expect(screen.queryByRole("button", { name: "Check again" })).toBeNull();
  },
);

test.each([
  { connected: false },
  { updatesAvailable: false },
  { error: new Error("controlled") },
])(
  "failed delivery cannot retain checking or claim signed out: %j",
  (patch) => {
    render(
      <ProviderSetup
        setup={view({
          ...patch,
          stale: true,
          pending: false,
          observation: { ...observed, phase: "checking" },
        })}
        onOpenHelp={vi.fn()}
      />,
    );
    expect(screen.getByRole("status").textContent).toBe(
      "Couldn’t verify Codex",
    );
    expect(screen.getByRole("status").getAttribute("data-tone")).toBe(
      "warning",
    );
    expect(screen.getByText(/Last verified account:/)).toBeTruthy();
    if (patch.connected === false || patch.updatesAvailable === false)
      expect(screen.queryByRole("button", { name: "Check again" })).toBeNull();
  },
);

test("unknown authentication remains unverified and does not invent an account or billing field", () => {
  render(
    <ProviderSetup
      setup={view({
        observation: {
          ...observed,
          evidence: {
            ...evidence,
            authentication: "unknown",
            account: null,
            billing: "unknown",
          },
        },
      })}
      onOpenHelp={vi.fn()}
    />,
  );
  expect(screen.getByRole("status").textContent).toBe("Couldn’t verify Codex");
  expect(screen.queryByText("Billing")).toBeNull();
});

test("pending browser attempt remains actionable during retrieval", () => {
  render(
    <ProviderSetup
      setup={view({
        pending: true,
        stale: true,
        observation: {
          ...observed,
          login: {
            id: "attempt",
            phase: "pending",
            outcome: null,
            browser_open_failed: false,
            diagnostic: null,
          },
        },
      })}
      onOpenHelp={vi.fn()}
      signIn={<button>Reopen browser</button>}
    />,
  );
  expect(screen.getByRole("button", { name: "Reopen browser" })).toBeTruthy();
  expect(screen.queryByText("Checking Codex…")).toBeNull();
  expect(screen.getByRole("status").getAttribute("data-tone")).toBe("warning");
});

test("a failed current verification outranks a retained failed login", () => {
  render(
    <ProviderSetup
      setup={view({
        stale: true,
        observation: {
          ...observed,
          problem: "status_read_failed",
          login: {
            id: "prior",
            phase: "failed",
            outcome: "failed",
            browser_open_failed: false,
            diagnostic: null,
          },
        },
      })}
      onOpenHelp={vi.fn()}
    />,
  );
  expect(screen.getByRole("status").textContent).toBe("Couldn’t verify Codex");
  expect(screen.getByRole("status").getAttribute("data-tone")).toBe("warning");
});
