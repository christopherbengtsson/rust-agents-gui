import { useState, type MouseEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Spinner } from "@/components/ui/spinner";
import { phaseLabel, terminal } from "@/lib/runtimes";
import type { Runtimes } from "@/hooks/use-runtimes";

const documents = {
  install: "https://developers.openai.com/codex/cli/",
  authentication: "https://developers.openai.com/codex/auth/",
};

export function CodexSettings({ runtimes }: { runtimes: Runtimes }) {
  const [draft, setDraft] = useState<string | null>(null);
  const [externalError, setExternalError] = useState<string>();
  const [copied, setCopied] = useState(false);
  const observed = runtimes.observation;
  const configuration = observed?.configuration;
  const value = draft ?? configuration?.executable ?? "";
  const dirty = value !== (configuration?.executable ?? "");
  const invalid =
    value !== "" &&
    (!value.startsWith("/") ||
      new TextEncoder().encode(value).length > 4096 ||
      /[\0\r\n]/.test(value));
  const active = observed?.active;
  const pending = runtimes.sending || !!runtimes.pending || !!observed?.busy;
  const diagnostic = observed?.diagnostic;
  const problem =
    runtimes.actionError ??
    observed?.blocked ??
    active?.error ??
    diagnostic?.error;
  const open = async (
    event: MouseEvent<HTMLAnchorElement>,
    document: keyof typeof documents,
  ) => {
    event.preventDefault();
    setExternalError(undefined);
    try {
      await invoke("open_runtime_document", { document });
    } catch {
      setExternalError(
        "Could not open the official instructions. Copy the displayed address and open it in your browser.",
      );
    }
  };
  const copy = async () => {
    setCopied(false);
    setExternalError(undefined);
    try {
      await navigator.clipboard.writeText("codex --version");
      setCopied(true);
    } catch {
      setExternalError(
        "Could not copy the command. Select and copy the displayed command manually.",
      );
    }
  };
  return (
    <section aria-labelledby="codex-settings" className="runtime-settings">
      <h3 id="codex-settings">Codex</h3>
      <p className="muted-copy">
        Use your installed official Codex CLI and its existing authentication.
        Supported version: 0.167.2.
      </p>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          if (!invalid && dirty && runtimes.canAct)
            void runtimes.save(value || null);
        }}
      >
        <FieldGroup>
          <Field data-invalid={invalid || undefined}>
            <FieldLabel htmlFor="codex-executable">Codex executable</FieldLabel>
            <Input
              id="codex-executable"
              value={value}
              onChange={(event) => setDraft(event.target.value)}
              aria-invalid={invalid || undefined}
              aria-describedby={`codex-path-help${invalid ? " codex-path-error" : ""}`}
              placeholder="Automatic discovery"
            />
            <FieldDescription id="codex-path-help">
              Leave empty for PATH and standard locations, or enter an absolute
              executable path. Save starts no Codex process. Unsaved edits are
              discarded when you leave Settings.
            </FieldDescription>
            {invalid && (
              <FieldError id="codex-path-error">
                Enter an absolute executable path of at most 4096 bytes.
              </FieldError>
            )}
          </Field>
          <div className="runtime-actions">
            <Button
              type="submit"
              size="sm"
              disabled={!runtimes.canAct || !dirty || invalid}
            >
              Save
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={!runtimes.canAct || configuration?.executable === null}
              onClick={() => {
                setDraft("");
                void runtimes.save(null);
              }}
            >
              Reset saved path
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={!runtimes.canAct || dirty || invalid}
              onClick={() => void runtimes.check()}
            >
              Check setup
            </Button>
          </div>
        </FieldGroup>
      </form>
      {dirty && (
        <p className="muted-copy">
          Unsaved executable changes. Save or discard them before checking
          setup.
        </p>
      )}
      <div role="status" aria-live="polite" className="runtime-progress">
        {pending && <Spinner aria-hidden="true" />}
        <span>
          {active && !terminal(active.phase)
            ? phaseLabel[active.phase]
            : pending
              ? "Retrieving recorded setup status…"
              : diagnostic
                ? diagnostic.passed
                  ? "Setup check passed"
                  : "Setup check needs attention"
                : "No setup diagnostic has run."}
        </span>
        {runtimes.stale && <Badge variant="outline">Stale observations</Badge>}
      </div>
      {diagnostic && (
        <dl className="runtime-evidence">
          <dt>Checked</dt>
          <dd>
            <time dateTime={new Date(diagnostic.timestamp_ms).toISOString()}>
              {new Date(diagnostic.timestamp_ms).toLocaleString()}
            </time>
          </dd>
          <dt>Executable</dt>
          <dd>{diagnostic.executable ?? "Unknown"}</dd>
          <dt>Version</dt>
          <dd>{diagnostic.version ?? "Unknown"}</dd>
          <dt>Authentication</dt>
          <dd>{diagnostic.authentication}</dd>
          <dt>Provider</dt>
          <dd>{diagnostic.provider ?? "Unknown"}</dd>
          <dt>Billing route</dt>
          <dd>{diagnostic.billing}</dd>
          <dt>Owned cleanup</dt>
          <dd>{diagnostic.cleanup_confirmed ? "Confirmed" : "Unconfirmed"}</dd>
        </dl>
      )}
      <p className="muted-copy">
        This optional check reads application-wide metadata, then closes its
        owned process. It does not verify project overrides or model execution.
      </p>
      {problem && (
        <Alert variant="destructive">
          <AlertTitle>Setup needs attention</AlertTitle>
          <AlertDescription>{problem.message}</AlertDescription>
        </Alert>
      )}
      {runtimes.error && (
        <Alert variant="destructive">
          <AlertTitle>Runtime observations unavailable</AlertTitle>
          <AlertDescription>
            {runtimes.error.message} Previous evidence is retained as stale.
          </AlertDescription>
        </Alert>
      )}
      {runtimes.preferenceError && (
        <Alert variant="destructive">
          <AlertDescription>{runtimes.preferenceError}</AlertDescription>
        </Alert>
      )}
      <Button
        type="button"
        variant="ghost"
        size="sm"
        disabled={runtimes.loading || runtimes.sending}
        onClick={() => void runtimes.refresh()}
      >
        Refresh saved status
      </Button>
      <div className="runtime-recovery">
        <p>
          <a
            href={documents.install}
            onClick={(event) => void open(event, "install")}
          >
            Official installation instructions
          </a>
        </p>
        <p>
          <a
            href={documents.authentication}
            onClick={(event) => void open(event, "authentication")}
          >
            Official authentication instructions
          </a>
        </p>
        <div className="runtime-actions">
          <code>codex --version</code>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => void copy()}
          >
            {copied ? "Copied command" : "Copy command"}
          </Button>
        </div>
        {externalError && (
          <Alert variant="destructive">
            <AlertDescription>{externalError}</AlertDescription>
          </Alert>
        )}
      </div>
    </section>
  );
}
