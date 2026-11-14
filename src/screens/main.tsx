import { Button } from "@/components/ui/button";
import { WorkspacePanel } from "@/components/blocks/workspace-panel";
import { useAppRuntime } from "@/app-runtime";
export function MainScreen() {
  const {
    app,
    navigation,
    target,
    status,
    repositories,
    workspaces,
    connection,
    refresh,
  } = useAppRuntime();
  if (
    status === "missing" ||
    (status === "pending" && !repositories.selected) ||
    (status === "unavailable" &&
      target?.kind !== "home" &&
      !repositories.selected)
  )
    return (
      <main id="workspace" tabIndex={-1} className="workspace-region flex-1">
        <header className="workspace-header">
          <h1>
            {status === "pending"
              ? "Loading destination…"
              : "Destination unavailable"}
          </h1>
        </header>
        <div className="workspace-content">
          <p>
            {status === "missing"
              ? "This repository or workspace does not exist, or the workspace belongs to another repository."
              : "The exact destination has not been confirmed. Retry its observations when the supervisor is available."}
          </p>
          <Button variant="outline" onClick={() => void refresh()}>
            Retry destination
          </Button>
        </div>
      </main>
    );
  return (
    <WorkspacePanel
      changeReader={app.inspection}
      connection={connection}
      workspaces={workspaces}
      repositories={repositories}
      onRefreshStatus={refresh}
      view={target?.kind === "workspace" ? target.view : "workspace"}
      onViewChange={(view) => {
        if (target?.kind === "workspace") navigation.go({ ...target, view });
      }}
    />
  );
}
