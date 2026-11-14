import { useObservationFeed } from "@/hooks/use-observation-feed";
import { useState } from "react";
import { AppObservations } from "@/lib/app-observations";
import { QueryClientProvider } from "@tanstack/react-query";
import { RepositorySidebar } from "@/components/blocks/repository-sidebar";
import { WorkspacePanel } from "@/components/blocks/workspace-panel";
import { SidebarProvider } from "@/components/ui/sidebar";
import { useSupervisorConnection } from "@/hooks/use-supervisor-connection";
import { useWorkspaces } from "@/hooks/use-workspaces";
import { useRepositories } from "@/hooks/use-repositories";

export function App() {
  const [observations] = useState(() => new AppObservations());
  const { connection, refreshStatus } = useSupervisorConnection();
  const feed = useObservationFeed(
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined,
    observations,
  );
  const observedConnection = { ...connection, feed };
  const repositories = useRepositories(
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined,
    observations,
  );
  const workspaces = useWorkspaces(
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined,
    repositories.selected?.id,
    repositories.page?.repositories.map((r) => r.id).join(",") ?? "",
    repositories.initialWorkspace,
    observations,
  );
  return (
    <QueryClientProvider client={observations.client}>
      <SidebarProvider className="h-dvh min-h-0">
        <a className="skip-link" href="#workspace">
          Skip to workspace
        </a>
        <RepositorySidebar
          connection={observedConnection}
          repositories={repositories}
          workspaces={workspaces}
        />
        <WorkspacePanel
          changeReader={observations.inspection}
          connection={observedConnection}
          workspaces={workspaces}
          repositories={repositories}
          onRefreshStatus={async () => {
            await refreshStatus();
            await repositories.refresh();
            await workspaces.refresh();
          }}
        />
      </SidebarProvider>
    </QueryClientProvider>
  );
}
