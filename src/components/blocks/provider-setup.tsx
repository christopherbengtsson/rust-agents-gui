import type { ProviderHelpTopic } from "@/lib/provider-help";
import { ChevronRight } from "lucide-react";
import { StatusIndicator } from "@/components/primitives/status-indicator";
import { Button } from "@/components/ui/button";
import { Alert, AlertTitle, AlertDescription } from "@/components/ui/alert";
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "@/components/ui/collapsible";
import type { useProviderSetup } from "@/hooks/use-provider-setup";
import type { SetupProblem } from "@/generated/contracts/types";

const guidance: Record<SetupProblem, { title: string; message: string }> = {
  missing_executable: {
    title: "Install the supported Codex runtime",
    message:
      "Install Codex 0.169.0 using the official installation instructions, then choose Check again. If PATH changed, restart the owning supervisor with the updated environment, then reopen the app.",
  },
  incompatible_protocol: {
    title: "Use the supported Codex version",
    message:
      "This setup reader supports Codex 0.169.0. Check the executable in Technical details, install that version, then choose Check again. If the selected executable path changed, restart the owning supervisor and reopen the app. A malformed or unexpected runtime reply also requires rechecking the installed runtime.",
  },
  authentication_unavailable: {
    title: "Sign in with the official Codex runtime",
    message:
      "Run codex login in your terminal using the Codex home in Technical details, then choose Check again.",
  },
  status_read_failed: {
    title: "Codex setup could not be checked",
    message:
      "Check that the executable and its configuration are accessible, then choose Check again. Previously observed account details remain stale until a successful check.",
  },
  cleanup_uncertain: {
    title: "Waiting for owned Codex cleanup",
    message:
      "Codex cleanup is still unconfirmed. Another check is blocked until the supervisor confirms the process has stopped. See Technical details for the retained diagnostic.",
  },
};
export function ProviderSetup({
  setup,
  onOpenHelp,
  helpError,
}: {
  setup: ReturnType<typeof useProviderSetup>;
  onOpenHelp: (topic: ProviderHelpTopic) => void;
  helpError?: string;
}) {
  const {
    target,
    observation,
    error,
    stale,
    pending,
    connected,
    updatesAvailable,
  } = setup;
  const available = connected && updatesAvailable;
  const evidence = observation?.evidence;
  const problem = observation?.problem;
  const cleanup = observation?.phase === "cleanup_uncertain";
  // The setup view excludes failed retained phases from pending, while an
  // actual retry can be pending alongside its previous read error.
  const checking = available && pending;
  const ready =
    available &&
    !error &&
    !checking &&
    !stale &&
    !problem &&
    !!evidence &&
    evidence.authentication !== "unavailable" &&
    evidence.authentication !== "unknown";
  const authentication = evidence
    ? {
        chatgpt: "Signed in with ChatGPT",
        api_key: "Existing API-key configuration",
        not_required: "OpenAI authentication not required",
        unavailable: "Not signed in",
        unknown: "Authentication unknown",
      }[evidence.authentication]
    : undefined;
  const label = !available
    ? "Status unavailable"
    : cleanup
      ? "Cleanup unconfirmed"
      : checking
        ? "Checking…"
        : error
          ? "Status unavailable"
          : problem === "authentication_unavailable"
            ? "Not connected"
            : problem
              ? "Setup needs attention"
              : ready
                ? "Connected"
                : "Readiness unknown";
  const recovery = !connected
    ? {
        title: "Supervisor unavailable",
        message:
          "Reconnect using the supervisor diagnostics. Current Codex setup could not be verified.",
      }
    : !updatesAvailable
      ? {
          title: "Live setup updates unavailable",
          message:
            "Return to your workspace and refresh the supervisor connection. Then reopen Providers to verify setup. Retained information cannot establish current readiness.",
        }
      : cleanup
        ? guidance.cleanup_uncertain
        : error
          ? {
              title: "Codex setup could not be checked",
              message:
                "The setup observation is unavailable. Choose Check again to retry the status read.",
            }
          : problem
            ? guidance[problem]
            : undefined;
  const showRecovery = !checking || cleanup || !!error;
  const canCheck = available && !pending && !cleanup && !ready;
  return (
    <section aria-label="Codex setup" className="provider-setup">
      <div className="provider-status">
        <StatusIndicator
          tone={
            ready
              ? "success"
              : problem && problem !== "authentication_unavailable" && !pending
                ? "error"
                : "neutral"
          }
          label={label}
        />
        {evidence && <p className="provider-status-detail">{authentication}</p>}
      </div>
      {showRecovery && recovery && (
        <Alert
          variant={
            problem && problem !== "authentication_unavailable"
              ? "destructive"
              : "default"
          }
        >
          <AlertTitle>{recovery.title}</AlertTitle>
          <AlertDescription>{recovery.message}</AlertDescription>
          {canCheck && (
            <div className="pt-3">
              <Button
                variant="outline"
                size="sm"
                onClick={() => void setup.refresh()}
              >
                Check again
              </Button>
            </div>
          )}
          {(problem === "missing_executable" ||
            problem === "incompatible_protocol" ||
            problem === "authentication_unavailable") && (
            <div className="pt-2">
              <Button
                variant="link"
                size="sm"
                onClick={() =>
                  onOpenHelp(
                    problem === "authentication_unavailable"
                      ? "authentication"
                      : "install",
                  )
                }
              >
                {problem === "authentication_unavailable"
                  ? "Official sign-in instructions"
                  : "Official installation instructions"}
              </Button>
            </div>
          )}
        </Alert>
      )}
      {canCheck && !recovery && (
        <Button
          variant="outline"
          size="sm"
          className="self-start"
          onClick={() => void setup.refresh()}
        >
          Check again
        </Button>
      )}
      {evidence && (
        <div className="provider-account">
          {(stale || pending) && (
            <p className="provider-retained-label">
              Last known · current readiness unverified
            </p>
          )}
          <dl className="provider-account-facts">
            {(evidence.account || evidence.authentication === "chatgpt") && (
              <>
                <dt>Account</dt>
                <dd>{evidence.account ?? "Unknown"}</dd>
              </>
            )}
            {evidence.plan && evidence.plan !== "unknown" && (
              <>
                <dt>Plan</dt>
                <dd>
                  {evidence.plan.charAt(0).toUpperCase() +
                    evidence.plan.slice(1)}
                </dd>
              </>
            )}
            <dt>Billing</dt>
            <dd>
              {
                { chatgpt: "ChatGPT", api_key: "API key", unknown: "Unknown" }[
                  evidence.billing
                ]
              }
            </dd>
          </dl>
        </div>
      )}
      <div className="provider-disclosures">
        {target && (
          <Collapsible className="provider-disclosure">
            <CollapsibleTrigger render={<Button variant="ghost" size="sm" />}>
              <ChevronRight aria-hidden="true" data-icon="inline-start" />
              Technical details
            </CollapsibleTrigger>
            <CollapsibleContent className="provider-disclosure-content">
              <p className="text-muted-foreground">
                User-level setup. Project overrides are not inspected here.
              </p>
              <dl className="provider-facts">
                <dt>Model provider</dt>
                <dd>{evidence?.model_provider ?? "Runtime default"}</dd>
                <dt>Model</dt>
                <dd>{evidence?.model ?? "Runtime default"}</dd>
                <dt>Executable</dt>
                <dd>{target.executable}</dd>
                <dt>Codex home</dt>
                <dd>{target.codex_home}</dd>
                <dt>Setup directory</dt>
                <dd>{target.working_directory}</dd>
                <dt>Supported runtime</dt>
                <dd>{target.supported_version}</dd>
                <dt>Observed runtime</dt>
                <dd>{evidence?.version ?? "Unknown"}</dd>
                <dt>Diagnostic</dt>
                <dd>
                  {observation?.diagnostic ??
                    problem ??
                    "No setup error observed"}
                </dd>
              </dl>
            </CollapsibleContent>
          </Collapsible>
        )}
        <Collapsible className="provider-disclosure">
          <CollapsibleTrigger render={<Button variant="ghost" size="sm" />}>
            <ChevronRight aria-hidden="true" data-icon="inline-start" />
            Setup help
          </CollapsibleTrigger>
          <CollapsibleContent className="provider-disclosure-content">
            <p>
              To sign in outside this app, run <code>codex login</code> in your
              terminal using the configuration in Technical details, then return
              to Providers. Setup is checked automatically when you open the
              Codex tab.
            </p>
            <p className="text-muted-foreground">
              Existing API-key configuration is detected through Codex.
            </p>
            <div className="flex flex-wrap gap-2">
              <Button
                variant="link"
                size="sm"
                onClick={() => onOpenHelp("install")}
              >
                Official installation instructions
              </Button>
              <Button
                variant="link"
                size="sm"
                onClick={() => onOpenHelp("authentication")}
              >
                Official sign-in instructions
              </Button>
            </div>
          </CollapsibleContent>
        </Collapsible>
      </div>
      {helpError && (
        <p role="alert" className="text-sm text-destructive">
          {helpError}
        </p>
      )}
    </section>
  );
}

export function ClaudeSetup() {
  return (
    <section aria-label="Claude Code setup" className="flex flex-col gap-4">
      <h3 className="font-medium">Claude connection isn't implemented yet</h3>
      <p className="text-sm text-muted-foreground">
        This tab is informational. Claude setup cannot be checked or connected
        here yet.
      </p>
      <div className="flex flex-wrap gap-2">
        <Button disabled>Sign in through browser</Button>
        <Button variant="outline" disabled>
          Run Claude login
        </Button>
      </div>
    </section>
  );
}
