import { RepositorySidebar } from "@/components/blocks/repository-sidebar";
import { WorkspacePanel } from "@/components/blocks/workspace-panel";
import { SidebarProvider } from "@/components/ui/sidebar";
import { useSupervisorConnection } from "@/hooks/use-supervisor-connection";

export function App() {
  const { connection, refreshStatus } = useSupervisorConnection();
  return (
    <SidebarProvider className="h-dvh min-h-0">
      <a className="skip-link" href="#workspace">
        Skip to workspace
      </a>
      <RepositorySidebar connection={connection} />
      <WorkspacePanel connection={connection} onRefreshStatus={refreshStatus} />
    </SidebarProvider>
  );
}
