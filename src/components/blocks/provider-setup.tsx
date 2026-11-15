import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Alert, AlertTitle, AlertDescription } from "@/components/ui/alert";
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "@/components/ui/collapsible";
import { Spinner } from "@/components/ui/spinner";
import type { useProviderSetup } from "@/hooks/use-provider-setup";
import type { SetupProblem } from "@/generated/contracts/types";

const guidance: Record<SetupProblem, { title: string; message: string }> = {
  missing_executable: {
    title: "Install the supported Codex runtime",
    message:
      "Install Codex 0.169.0 using the official installation instructions, then choose Refresh. If PATH changed, restart the owning supervisor with the updated environment, then reopen the app.",
  },
  incompatible_protocol: {
    title: "Use the supported Codex version",
    message:
      "This setup reader supports Codex 0.169.0. Check the executable below, install that version, then choose Refresh. If the selected executable path changed, restart the owning supervisor and reopen the app. A malformed or unexpected runtime reply also requires rechecking the installed runtime.",
  },
  authentication_unavailable: {
    title: "Sign in with the official Codex runtime",
    message:
      "Run codex login in your terminal using the Codex home shown below, then choose Refresh. Browser sign-in and an embedded login terminal will be added here later.",
  },
  status_read_failed: {
    title: "Codex setup could not be checked",
    message:
      "Check that the executable and its configuration are accessible, then choose Refresh. Previously observed account details remain stale until a successful check.",
  },
  cleanup_uncertain: {
    title: "Waiting for owned Codex cleanup",
    message:
      "The setup process has not confirmed its exit. A replacement is blocked while the supervisor retains ownership. Recheck status; do not launch another setup process to bypass this state.",
  },
};
export function ProviderSetup({
  setup,
}: {
  setup: ReturnType<typeof useProviderSetup>;
}) {
  const { target, observation, error, stale, pending, connected } = setup;
  const evidence = observation?.evidence;
  const problem = observation?.problem;
  const recovery = problem ? guidance[problem] : undefined;
  const authentication = evidence
    ? {
        chatgpt: "Signed in with ChatGPT",
        api_key: "Existing API-key configuration",
        not_required: "OpenAI authentication not required",
        unavailable: "Not signed in",
        unknown: "Unknown authentication route",
      }[evidence.authentication]
    : "Not yet observed";
  return (
    <section aria-label="Codex setup" className="flex min-w-0 flex-col gap-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-col gap-2">
          <h3 className="font-medium">Existing Codex setup</h3>
          <div className="flex flex-wrap items-center gap-2" aria-live="polite">
            <Badge variant={problem ? "destructive" : "secondary"}>
              {pending ? "Checking…" : authentication}
            </Badge>
            {stale && evidence && (
              <Badge variant="outline">Stale · readiness unknown</Badge>
            )}
          </div>
        </div>
        <Button
          variant="outline"
          size="sm"
          disabled={
            !connected || pending || observation?.phase === "cleanup_uncertain"
          }
          onClick={() => void setup.refresh()}
        >
          {pending && <Spinner data-icon="inline-start" />}Refresh
        </Button>
      </div>
      <p className="text-sm text-muted-foreground">
        User-level setup. Project overrides are not inspected here. This check
        starts no conversation.
      </p>
      {!connected && (
        <Alert>
          <AlertTitle>Supervisor unavailable</AlertTitle>
          <AlertDescription>
            Reconnect using the supervisor diagnostics. Retained provider
            evidence cannot establish current readiness.
          </AlertDescription>
        </Alert>
      )}
      {!!error && (
        <Alert variant="destructive">
          <AlertTitle>Setup observation unavailable</AlertTitle>
          <AlertDescription>
            Refresh after reconnecting to the current supervisor. Retained
            information is stale.
          </AlertDescription>
        </Alert>
      )}
      {evidence && (
        <dl className="provider-facts text-sm">
          <dt>Authentication</dt>
          <dd>{authentication}</dd>
          <dt>Account</dt>
          <dd>{evidence.account ?? "Unknown"}</dd>
          <dt>Plan</dt>
          <dd>{evidence.plan ?? "Unknown"}</dd>
          <dt>Billing route</dt>
          <dd>
            {
              { chatgpt: "ChatGPT", api_key: "API key", unknown: "Unknown" }[
                evidence.billing
              ]
            }
          </dd>
          <dt>Model provider</dt>
          <dd>{evidence.model_provider ?? "Runtime default"}</dd>
          <dt>Model</dt>
          <dd>{evidence.model ?? "Runtime default"}</dd>
        </dl>
      )}
      {recovery && (
        <Alert
          variant={
            problem === "authentication_unavailable" ? "default" : "destructive"
          }
        >
          <AlertTitle>{recovery.title}</AlertTitle>
          <AlertDescription>{recovery.message}</AlertDescription>
        </Alert>
      )}
      {target && (
        <Collapsible>
          <CollapsibleTrigger render={<Button variant="ghost" size="sm" />}>
            Technical setup details
          </CollapsibleTrigger>
          <CollapsibleContent className="pt-3">
            <dl className="provider-facts text-sm">
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
              <dd>{problem ?? "No setup error observed"}</dd>
            </dl>
          </CollapsibleContent>
        </Collapsible>
      )}
      <section
        aria-label="Codex setup guidance"
        className="flex flex-col gap-3 border-t pt-5 text-sm"
      >
        <h3 className="font-medium">Official setup options</h3>
        <p>
          <strong>ChatGPT</strong> — Sign in through your browser using the
          official Codex runtime.
        </p>
        <p>
          <strong>CLI</strong> — Use your existing Codex setup, or run{" "}
          <code>codex login</code> in your terminal and return here to Refresh.
        </p>
        <p className="text-muted-foreground">
          Existing API-key configuration stays with Codex. This app does not
          collect or edit API keys.
        </p>
        <p className="text-muted-foreground">
          Official installation:{" "}
          <code className="break-all select-text">
            https://developers.openai.com/codex/cli
          </code>
          <br />
          Authentication:{" "}
          <code className="break-all select-text">
            https://developers.openai.com/codex/auth
          </code>
        </p>
      </section>
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
