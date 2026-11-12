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
import type { Connection } from "@/hooks/use-supervisor-connection";

export function WorkspacePanel({
  connection,
  onRefreshStatus,
}: {
  connection: Connection;
  onRefreshStatus: () => Promise<void>;
}) {
  return (
    <main id="workspace" tabIndex={-1} className="workspace-region flex-1">
      <header className="workspace-header">
        <h1>Workspace</h1>
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
        <Empty>
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <FolderGit2 aria-hidden="true" />
            </EmptyMedia>
            <EmptyTitle>
              <h2>A place for your next change</h2>
            </EmptyTitle>
            <EmptyDescription>
              Your repositories and workspaces will live here. Repository
              registration is coming next.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      </div>
      <footer className="workspace-footer">
        <span>Local workspace</span>
        <span>No workspace selected</span>
      </footer>
    </main>
  );
}
