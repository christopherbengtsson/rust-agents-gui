import { RepositorySidebar } from "@/components/blocks/repository-sidebar";
import { WorkspacePanel } from "@/components/blocks/workspace-panel";
import { SidebarProvider } from "@/components/ui/sidebar";
import { useSupervisorConnection } from "@/hooks/use-supervisor-connection";
import { useWorkspaces } from "@/hooks/use-workspaces";
import { useRepositories } from "@/hooks/use-repositories";

export function App() {
  const { connection, refreshStatus } = useSupervisorConnection();
  const repositories = useRepositories(
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined,
  );
  const workspaces = useWorkspaces(
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined,
    repositories.selected?.id,
    repositories.page?.repositories.map((r) => r.id).join(",") ?? "",
    repositories.initialWorkspace,
  );
  return (
    <SidebarProvider className="h-dvh min-h-0">
      <a className="skip-link" href="#workspace">
        Skip to workspace
      </a>
      <RepositorySidebar
        connection={connection}
        repositories={repositories}
        workspaces={workspaces}
      />
      <WorkspacePanel
        connection={connection}
        workspaces={workspaces}
        repositories={repositories}
        onRefreshStatus={async () => {
          await refreshStatus();
          await repositories.refresh();
          await workspaces.refresh();
        }}
      />
    </SidebarProvider>
  );
}
