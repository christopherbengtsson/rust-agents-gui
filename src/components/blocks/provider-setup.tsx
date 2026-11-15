import type { ProviderHelpTopic } from "@/lib/provider-help";
import { ChevronRight } from "lucide-react";
import { StatusIndicator } from "@/components/primitives/status-indicator";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { Alert, AlertTitle, AlertDescription } from "@/components/ui/alert";
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent,
} from "@/components/ui/collapsible";
import type { useProviderSetup } from "@/hooks/use-provider-setup";
import { useEffect, useRef, type ReactNode } from "react";
import { loginActive } from "@/hooks/use-provider-progress";

export function ProviderSetup({
  setup,
  onOpenHelp,
  helpError,
  signIn,
  onLocate,
  locating = false,
  locateError,
  actionUncertain = false,
  selectionUncertain = false,
  onReconcileSelection,
}: {
  setup: ReturnType<typeof useProviderSetup>;
  onOpenHelp: (topic: ProviderHelpTopic) => void;
  helpError?: string;
  signIn?: ReactNode;
  onLocate?: () => Promise<void>;
  locating?: boolean;
  locateError?: string;
  actionUncertain?: boolean;
  selectionUncertain?: boolean;
  onReconcileSelection?: () => void;
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
  const cleanup =
    observation?.phase === "cleanup_uncertain" ||
    observation?.login?.phase === "cleanup_uncertain";
  const signingIn = loginActive(observation);
  const verifying = observation?.login?.phase === "verifying";
  const checking =
    available &&
    !cleanup &&
    !actionUncertain &&
    ((pending && !signingIn) || verifying);
  const ready =
    available &&
    !error &&
    !actionUncertain &&
    !checking &&
    !stale &&
    !signingIn &&
    !problem &&
    !!evidence &&
    evidence.authentication !== "unavailable" &&
    evidence.authentication !== "unknown";
  const missing = problem === "missing_executable";
  const signedOut =
    !error && available && problem === "authentication_unavailable";
  const loginFailed =
    observation?.login?.phase === "failed" &&
    !ready &&
    problem !== "status_read_failed";
  const confirmedFailure =
    !error &&
    available &&
    (missing ||
      problem === "incompatible_protocol" ||
      problem === "selection_failed" ||
      loginFailed);
  const uncertain =
    !available ||
    !!error ||
    problem === "status_read_failed" ||
    (!ready && !signedOut && !confirmedFailure && !signingIn);
  const canCheck =
    available &&
    !pending &&
    !cleanup &&
    !ready &&
    !signingIn &&
    !actionUncertain;
  const label = cleanup
    ? "Cleanup unconfirmed"
    : signingIn
      ? "Codex sign-in in progress"
      : !available || error
        ? "Couldn’t verify Codex"
        : missing
          ? "Codex not found"
          : confirmedFailure
            ? "Codex needs attention"
            : signedOut
              ? "Not connected"
              : ready
                ? evidence.authentication === "api_key"
                  ? "Connected · Using API key"
                  : "Connected"
                : "Couldn’t verify Codex";
  const section = useRef<HTMLElement>(null);
  const focusOrigin = useRef<HTMLElement | undefined>(undefined);
  const status = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const origin = focusOrigin.current;
    if (
      origin &&
      !origin.isConnected &&
      document.activeElement === document.body
    )
      section.current?.focus({ preventScroll: true });
    if (
      !checking &&
      focusOrigin.current &&
      document.activeElement === section.current
    ) {
      status.current?.focus({ preventScroll: true });
      focusOrigin.current = undefined;
    }
  }, [checking, label]);
  function check() {
    focusOrigin.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : undefined;
    void setup.refresh();
  }
  return (
    <section
      ref={section}
      onFocusCapture={(event) => {
        focusOrigin.current = event.target;
      }}
      tabIndex={-1}
      aria-label="Codex setup"
      aria-busy={checking}
      className="provider-setup"
    >
      {checking ? (
        <>
          <span role="status" className="sr-only">
            Checking Codex…
          </span>
          <div aria-hidden="true" className="flex flex-col gap-4">
            <Skeleton className="h-6 w-40" />
            <Skeleton className="h-5 w-64 max-w-full" />
            <Skeleton className="h-9 w-28" />
          </div>
        </>
      ) : (
        <>
          <div ref={status} tabIndex={-1} className="provider-status">
            <StatusIndicator
              tone={
                ready
                  ? "success"
                  : confirmedFailure &&
                      !cleanup &&
                      !signingIn &&
                      !actionUncertain
                    ? "error"
                    : "warning"
              }
              label={label}
            />
            {ready && evidence.account && (
              <p className="provider-status-detail">{evidence.account}</p>
            )}
            {!ready && evidence?.account && (
              <p className="provider-status-detail">
                Last verified account: {evidence.account} · current readiness
                unverified
              </p>
            )}
          </div>
          {!ready && !signingIn && (
            <>
              {cleanup ? (
                <Alert>
                  <AlertTitle>Waiting for owned Codex cleanup</AlertTitle>
                  <AlertDescription>
                    Cleanup is unconfirmed. Another check or executable
                    selection is blocked until owned resources have stopped.
                  </AlertDescription>
                </Alert>
              ) : !available ? (
                <Alert>
                  <AlertTitle>Supervisor updates unavailable</AlertTitle>
                  <AlertDescription>
                    Return to your workspace and reconnect using the supervisor
                    diagnostics, then reopen Providers. Retained information
                    cannot establish current readiness.
                  </AlertDescription>
                </Alert>
              ) : missing ? (
                <p className="text-sm text-muted-foreground">
                  {target?.selected_executable
                    ? "The remembered Codex location is unavailable. Check it again or locate a replacement."
                    : "We couldn’t find Codex. Install it or locate an existing executable, then check again."}
                </p>
              ) : problem === "incompatible_protocol" ? (
                <Alert variant="destructive">
                  <AlertTitle>Use a compatible Codex runtime</AlertTitle>
                  <AlertDescription>
                    Choose a stable Codex release at or above{" "}
                    {target?.minimum_supported_version ??
                      "the supported minimum"}{" "}
                    with the required runtime capabilities.
                  </AlertDescription>
                </Alert>
              ) : problem === "selection_failed" ? (
                <Alert variant="destructive">
                  <AlertTitle>
                    Codex location could not be remembered
                  </AlertTitle>
                  <AlertDescription>
                    The previous selection is unchanged. Locate Codex again to
                    retry.
                  </AlertDescription>
                </Alert>
              ) : uncertain ? (
                <p className="text-sm text-muted-foreground">
                  Current authentication could not be verified. Check again to
                  retrieve authoritative status.
                </p>
              ) : null}
              <div className="flex flex-wrap gap-2">
                {missing && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onOpenHelp("install")}
                  >
                    Installation guide
                  </Button>
                )}
                {canCheck && !signedOut && (
                  <Button variant="outline" size="sm" onClick={check}>
                    Check again
                  </Button>
                )}
              </div>
            </>
          )}
          {selectionUncertain && (
            <Alert>
              <AlertTitle>Codex location status is unconfirmed</AlertTitle>
              <AlertDescription>
                Retrieve the recorded selection before choosing another
                executable.
              </AlertDescription>
              <Button
                variant="outline"
                size="sm"
                disabled={!available}
                onClick={onReconcileSelection}
              >
                Retrieve location status
              </Button>
            </Alert>
          )}
          {!ready && signIn}
          {!ready &&
            (missing ||
              problem === "incompatible_protocol" ||
              problem === "selection_failed") && (
              <Collapsible className="provider-disclosure">
                <CollapsibleTrigger
                  render={<Button variant="ghost" size="sm" />}
                >
                  <ChevronRight aria-hidden="true" data-icon="inline-start" />
                  Already installed?
                </CollapsibleTrigger>
                <CollapsibleContent className="provider-disclosure-content">
                  <p>Choose the Codex executable on this Mac.</p>
                  <Button
                    variant="outline"
                    size="sm"
                    className="self-start"
                    disabled={!canCheck || locating || !onLocate}
                    onClick={() => {
                      focusOrigin.current =
                        document.activeElement instanceof HTMLElement
                          ? document.activeElement
                          : undefined;
                      void onLocate?.().finally(() => {
                        const origin = focusOrigin.current;
                        if (
                          origin?.isConnected &&
                          document.activeElement === document.body
                        )
                          origin.focus({ preventScroll: true });
                      });
                    }}
                  >
                    Locate Codex…
                  </Button>
                </CollapsibleContent>
              </Collapsible>
            )}
          {!ready && target && !signingIn && (
            <Collapsible className="provider-disclosure">
              <CollapsibleTrigger render={<Button variant="ghost" size="sm" />}>
                <ChevronRight aria-hidden="true" data-icon="inline-start" />
                Technical details
              </CollapsibleTrigger>
              <CollapsibleContent className="provider-disclosure-content">
                <dl className="provider-facts">
                  <dt>Executable</dt>
                  <dd>{observation?.target.executable ?? "Not found"}</dd>
                  <dt>Remembered location</dt>
                  <dd>
                    {observation?.target.selected_executable ??
                      "Automatic discovery"}
                  </dd>
                  <dt>Minimum runtime</dt>
                  <dd>{target.minimum_supported_version}</dd>
                  <dt>Observed runtime</dt>
                  <dd>{evidence?.version ?? "Unknown"}</dd>
                  <dt>Diagnostic</dt>
                  <dd>
                    {observation?.diagnostic ??
                      error?.message ??
                      "No diagnostic available"}
                  </dd>
                </dl>
              </CollapsibleContent>
            </Collapsible>
          )}
          {!ready && helpError && (
            <p role="alert" className="text-sm text-destructive">
              {helpError}
            </p>
          )}
          {!ready && locateError && (
            <p role="alert" className="text-sm text-destructive">
              {locateError}
            </p>
          )}
        </>
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
