import { GitBranch, Folder, Check, CircleAlert } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Alert, AlertTitle, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { workspaceStatus, type Workspace } from "@/lib/workspaces";

export function WorkspaceDetails({
  workspace,
  connected,
  stale,
  pending,
  onReconcile,
}: {
  workspace: Workspace;
  connected: boolean;
  stale: boolean;
  pending: boolean;
  onReconcile: () => void;
}) {
  const busy =
    workspace.status.state === "queued" || workspace.status.state === "running";
  const failure =
    workspace.status.state === "failed" ||
    workspace.status.state === "uncertain";
  return (
    <section
      aria-label="Selected workspace"
      className="workspace-details flex flex-col gap-5 pt-6"
    >
      <div className="flex items-center gap-2">
        <Badge variant={failure ? "destructive" : "secondary"}>
          {workspaceStatus(workspace.status)}
        </Badge>
        <Badge variant="outline">Codex</Badge>
      </div>
      <h2 className="workspace-welcome">
        {workspace.status.state === "ready"
          ? "Your workspace is ready"
          : failure
            ? "Workspace needs attention"
            : "Preparing your workspace"}
      </h2>
      <p className="repository-path">{workspace.name}</p>
      <dl className="workspace-facts flex flex-col gap-4">
        <div>
          <dt>
            <GitBranch aria-hidden="true" />
            Branch
          </dt>
          <dd>{workspace.branch}</dd>
        </div>
        <div>
          <dt>
            <Folder aria-hidden="true" />
            Workspace folder
          </dt>
          <dd>{workspace.path}</dd>
        </div>
        <div>
          <dt>Local base</dt>
          <dd>
            {workspace.base
              ? `${workspace.base.reference} · ${workspace.base.commit}`
              : "Waiting for an identifiable local default branch"}
          </dd>
        </div>
        <div>
          <dt>Branch artifact</dt>
          <dd>
            {
              {
                unknown: "Not yet confirmed",
                absent: "Not created",
                owned: "Created by this action",
                conflict: "Conflicting branch",
              }[workspace.artifacts.branch]
            }
          </dd>
        </div>
        <div>
          <dt>Folder artifact</dt>
          <dd>
            {
              {
                unknown: "Not yet confirmed",
                absent: "Not created",
                reserved: "Reserved for this workspace",
                worktree: "Worktree created",
                partial: "Incomplete worktree; files retained",
                conflict: "Conflicting folder",
              }[workspace.artifacts.folder]
            }
          </dd>
        </div>
      </dl>
      {busy && (
        <p role="status" className="flex items-center gap-2">
          <Spinner aria-hidden="true" />
          {workspaceStatus(workspace.status)}
        </p>
      )}
      {failure && (
        <Alert variant="destructive">
          <CircleAlert aria-hidden="true" />
          <AlertTitle>{workspaceStatus(workspace.status)}</AlertTitle>
          <AlertDescription>
            <p className="error-detail">
              {"error" in workspace.status
                ? workspace.status.error.message
                : ""}
            </p>
            <Button
              variant="outline"
              size="sm"
              disabled={!connected || pending}
              onClick={onReconcile}
            >
              Reconcile workspace
            </Button>
          </AlertDescription>
        </Alert>
      )}
      {workspace.status.state === "ready" && (
        <Alert>
          <Check aria-hidden="true" />
          <AlertTitle>Isolated checkout created</AlertTitle>
          <AlertDescription>
            Your workspace uses the pinned local base above. Your original
            checkout is unchanged. No agent has been started.
          </AlertDescription>
        </Alert>
      )}
      {stale && (
        <p className="muted-copy">
          Workspace details are retained from the last observation and may be
          stale. Refresh to retrieve authoritative status.
        </p>
      )}
    </section>
  );
}
