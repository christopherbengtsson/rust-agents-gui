import { RepositorySidebar } from "@/components/blocks/repository-sidebar";
import { WorkspacePanel } from "@/components/blocks/workspace-panel";
import { SidebarProvider } from "@/components/ui/sidebar";
import { useSupervisorConnection } from "@/hooks/use-supervisor-connection";
import { useRepositories } from "@/hooks/use-repositories";

export function App() {
  const { connection, refreshStatus } = useSupervisorConnection();
  const repositories = useRepositories(
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined,
  );
  return (
    <SidebarProvider className="h-dvh min-h-0">
      <a className="skip-link" href="#workspace">
        Skip to workspace
      </a>
      <RepositorySidebar connection={connection} repositories={repositories} />
      <WorkspacePanel
        connection={connection}
        repositories={repositories}
        onRefreshStatus={async () => {
          await refreshStatus();
          await repositories.refresh();
        }}
      />
    </SidebarProvider>
  );
}
