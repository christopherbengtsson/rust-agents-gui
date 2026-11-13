import { CircleAlert, FolderGit2, RefreshCw } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { WorkspaceDetails } from "./workspace-details";
import type { Workspaces } from "@/hooks/use-workspaces";
import type { Connection } from "@/hooks/use-supervisor-connection";
import type { Repositories } from "@/hooks/use-repositories";

export function WorkspacePanel({
  connection,
  onRefreshStatus,
  repositories,
  workspaces,
}: {
  connection: Connection;
  onRefreshStatus: () => Promise<void>;
  repositories: Repositories;
  workspaces: Workspaces;
}) {
  return (
    <main id="workspace" tabIndex={-1} className="workspace-region flex-1">
      <header className="workspace-header">
        <h1 className="min-w-0 truncate">
          {workspaces.selected
            ? (workspaces.selected.name ?? "New workspace")
            : (repositories.selected?.name ?? "Workspace")}
        </h1>
        <Button
          variant="outline"
          size="sm"
          disabled={connection.state === "checking"}
          onClick={() => void onRefreshStatus()}
        >
          <RefreshCw data-icon="inline-start" />
          {connection.state === "checking"
            ? "Checking…"
            : connection.state === "error"
              ? "Retry connection"
              : "Refresh status"}
        </Button>
      </header>
      <div className="workspace-content">
        {repositories.pending && (
          <p role="status">
            {repositories.actionError
              ? "Registration needs attention."
              : "Registering repository…"}
          </p>
        )}
        {(repositories.error || repositories.actionError) && (
          <Alert variant="destructive">
            <CircleAlert aria-hidden="true" />
            <AlertTitle>Repository action needs attention</AlertTitle>
            <AlertDescription>
              <p className="error-detail">
                {repositories.error?.message ??
                  repositories.actionError?.message}
              </p>
              <p>
                Refresh to retrieve authoritative repository and action status.
              </p>
            </AlertDescription>
          </Alert>
        )}
        {repositories.preferenceError && (
          <Alert>
            <AlertTitle>Desktop preference unavailable</AlertTitle>
            <AlertDescription>{repositories.preferenceError}</AlertDescription>
          </Alert>
        )}
        {repositories.selected && (
          <section
            aria-label="Selected repository"
            className="flex flex-col gap-2"
          >
            <p className="repository-path">{repositories.selected.path}</p>
            <p>
              {repositories.selected.availability.state === "checking"
                ? "Checking repository availability…"
                : repositories.selected.availability.state === "available"
                  ? "Repository available"
                  : "Repository unavailable"}
            </p>
            {repositories.selected.availability.state === "unavailable" && (
              <Alert variant="destructive">
                <AlertTitle>Restore access to this repository</AlertTitle>
                <AlertDescription>
                  {repositories.selected.availability.error.message}
                </AlertDescription>
              </Alert>
            )}
            {(connection.state !== "connected" || repositories.error) && (
              <p>
                Repository details are retained from the last observation and
                may be stale.
              </p>
            )}
          </section>
        )}
        {connection.state === "error" && (
          <Alert variant="destructive" className="connection-error">
            <CircleAlert aria-hidden="true" />
            <AlertTitle>Cannot reach the supervisor</AlertTitle>
            <AlertDescription>
              <p className="error-detail">{connection.error}</p>
              <p>
                After addressing the error, retry the connection to retrieve its
                status.
              </p>
            </AlertDescription>
          </Alert>
        )}
        {workspaces.error && (
          <Alert variant="destructive">
            <AlertTitle>Workspace action needs attention</AlertTitle>
            <AlertDescription>
              <p className="error-detail">{workspaces.error.message}</p>
              {workspaces.pending && (
                <Button
                  size="sm"
                  variant="outline"
                  disabled={
                    workspaces.submitting || connection.state !== "connected"
                  }
                  onClick={() => void workspaces.retry()}
                >
                  Retry same submission
                </Button>
              )}
            </AlertDescription>
          </Alert>
        )}
        {workspaces.preferenceError && (
          <Alert>
            <AlertTitle>Desktop preference unavailable</AlertTitle>
            <AlertDescription>{workspaces.preferenceError}</AlertDescription>
          </Alert>
        )}
        {workspaces.pending && (
          <p role="status">Retrieving workspace creation status…</p>
        )}
        {workspaces.selected ? (
          <WorkspaceDetails
            workspace={workspaces.selected}
            connected={connection.state === "connected"}
            stale={connection.state !== "connected" || !!workspaces.error}
            pending={workspaces.submitting || !!workspaces.pending}
            onReconcile={() => void workspaces.reconcile(workspaces.selected!)}
          />
        ) : (
          <Empty>
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <FolderGit2 aria-hidden="true" />
              </EmptyMedia>
              <EmptyTitle>
                <h2>
                  {repositories.selected
                    ? "Repository registered"
                    : "A place for your next change"}
                </h2>
              </EmptyTitle>
              <EmptyDescription>
                {repositories.selected
                  ? "Create an isolated workspace from the local default branch. Your original checkout stays untouched."
                  : "Add an existing Git repository from the sidebar to get started."}
              </EmptyDescription>
            </EmptyHeader>
            {repositories.selected && (
              <Button
                size="sm"
                disabled={
                  connection.state !== "connected" ||
                  repositories.selected.availability.state !== "available" ||
                  workspaces.submitting ||
                  !!workspaces.pending
                }
                onClick={() =>
                  void workspaces.create(repositories.selected!.id)
                }
              >
                Create workspace
              </Button>
            )}
          </Empty>
        )}
      </div>
      <footer className="workspace-footer">
        <span className="shrink-0">Local workspace</span>
        <span className="min-w-0 truncate">
          {repositories.selected
            ? repositories.selected.name
            : "No repository selected"}
        </span>
      </footer>
    </main>
  );
}
