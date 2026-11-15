import { Alert, AlertTitle, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import type {
  ProviderObservation,
  LoginPhase,
} from "@/generated/contracts/types";
import type { useProviderLogin } from "@/hooks/use-provider-progress";
import { useEffect, useRef } from "react";

const labels: Record<LoginPhase, string> = {
  starting: "Starting browser sign-in…",
  pending: "Complete sign-in in your browser",
  cancelling: "Cancelling sign-in…",
  verifying: "Checking authentication after sign-in…",
  completed: "Browser sign-in completed",
  failed: "Browser sign-in failed",
  cancelled: "Browser sign-in cancelled",
  cleanup_uncertain: "Sign-in cleanup unconfirmed",
};
export function ProviderSignIn({
  observation,
  available,
  login,
}: {
  observation: ProviderObservation | undefined;
  available: boolean;
  login: ReturnType<typeof useProviderLogin>;
}) {
  const attempt = observation?.login;
  const focusOrigin = useRef<HTMLButtonElement | undefined>(undefined);
  const progress = useRef<HTMLParagraphElement>(null);
  useEffect(() => {
    // Recover only focus lost by this view's explicit action removing its own
    // control. Navigation/unmount and focus moved elsewhere retain authority.
    if (focusOrigin.current && document.activeElement === document.body) {
      if (!focusOrigin.current.isConnected) {
        progress.current?.focus({ preventScroll: true });
        focusOrigin.current = undefined;
      } else if (!login.pending) {
        focusOrigin.current.focus({ preventScroll: true });
        focusOrigin.current = undefined;
      }
    }
  }, [attempt?.phase, login.pending]);
  const active =
    !!attempt && !["completed", "failed", "cancelled"].includes(attempt.phase);
  const canStart =
    available &&
    !!observation &&
    !login.pending &&
    !login.uncertain &&
    !active &&
    observation.phase !== "checking" &&
    observation.phase !== "cleanup_uncertain" &&
    observation.problem === "authentication_unavailable";
  return (
    <section aria-label="ChatGPT sign-in" className="flex flex-col gap-3">
      {!active && observation?.problem === "authentication_unavailable" && (
        <p className="text-sm text-muted-foreground">
          Sign in with ChatGPT in your browser. Codex manages your credentials.
        </p>
      )}
      {attempt && (
        <div className="flex flex-col gap-2">
          <p
            ref={progress}
            tabIndex={-1}
            role="status"
            aria-live="polite"
            aria-atomic="true"
          >
            {labels[attempt.phase]}
          </p>
          {attempt.diagnostic && (
            <p className="text-sm text-muted-foreground">
              {attempt.diagnostic}
            </p>
          )}
          {attempt.phase === "completed" && observation.problem && (
            <p className="text-sm">
              The browser reported completion. Current authentication still
              needs verification; see setup status above.
            </p>
          )}
          {attempt.browser_open_failed && attempt.phase === "pending" && (
            <Alert>
              <AlertTitle>Browser could not be opened</AlertTitle>
              <AlertDescription>
                Reopen this sign-in attempt to continue. Another login has not
                been started.
              </AlertDescription>
            </Alert>
          )}
        </div>
      )}
      {login.message && !login.uncertain && (
        <p role="alert" className="text-sm text-destructive">
          {login.message}
        </p>
      )}
      {login.uncertain && (
        <Alert>
          <AlertTitle>Sign-in status is unconfirmed</AlertTitle>
          <AlertDescription>
            Retrieve the recorded attempt before taking another action.
          </AlertDescription>
          <Button
            variant="outline"
            size="sm"
            disabled={!available || login.pending}
            onClick={login.reconcile}
          >
            Retrieve sign-in status
          </Button>
        </Alert>
      )}
      <div className="flex flex-wrap gap-2">
        {!active && observation?.problem === "authentication_unavailable" && (
          <Button
            disabled={!canStart}
            onClick={(event) => {
              focusOrigin.current = event.currentTarget;
              login.start();
            }}
          >
            Sign in
          </Button>
        )}
        {attempt?.phase === "pending" && (
          <Button
            variant="outline"
            disabled={!available || login.pending || login.uncertain}
            onClick={(event) => {
              focusOrigin.current = event.currentTarget;
              login.reopen();
            }}
          >
            Reopen browser
          </Button>
        )}
        {attempt &&
          ["starting", "pending", "cancelling"].includes(attempt.phase) && (
            <Button
              variant="outline"
              disabled={
                !available ||
                login.pending ||
                login.uncertain ||
                attempt.phase === "cancelling"
              }
              onClick={(event) => {
                focusOrigin.current = event.currentTarget;
                login.cancel();
              }}
            >
              Cancel sign-in
            </Button>
          )}
        {(login.pending ||
          (active &&
            attempt?.phase !== "pending" &&
            attempt?.phase !== "cleanup_uncertain")) && (
          <Spinner aria-label="Updating sign-in" />
        )}
      </div>
    </section>
  );
}
