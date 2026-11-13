import { RepositorySidebar } from "@/components/blocks/repository-sidebar";
import { Activity } from "react";
import { useAppNavigation } from "./use-app-navigation";
import { SettingsScreen } from "@/components/blocks/settings-screen";
import { WorkspacePanel } from "@/components/blocks/workspace-panel";
import { SidebarProvider } from "@/components/ui/sidebar";
import { useSupervisorConnection } from "@/hooks/use-supervisor-connection";
import { useWorkspaces } from "@/hooks/use-workspaces";
import { useRepositories } from "@/hooks/use-repositories";
import { useRuntimes } from "@/hooks/use-runtimes";

export function App() {
  const navigation = useAppNavigation();
  const { connection, refreshStatus } = useSupervisorConnection();
  const runtimes = useRuntimes(
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined,
    navigation.screen === "settings",
  );
  const repositories = useRepositories(
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined,
    navigation.selection,
  );
  const workspaces = useWorkspaces(
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined,
    repositories.selected?.id,
    repositories.page?.repositories.map((r) => r.id).join(",") ?? "",
    repositories.initialWorkspace,
    navigation.selection,
  );
  return (
    <SidebarProvider className="h-dvh min-h-0">
      <a
        className="skip-link"
        href={navigation.screen === "settings" ? "#settings" : "#workspace"}
      >
        {navigation.screen === "settings"
          ? "Skip to settings"
          : "Skip to workspace"}
      </a>
      <Activity mode={navigation.screen === "workbench" ? "visible" : "hidden"}>
        <RepositorySidebar
          connection={connection}
          repositories={repositories}
          workspaces={workspaces}
          onSettings={navigation.openSettings}
          settingsRef={navigation.settingsTrigger}
          disclosureKey={navigation.visit}
          canRestoreFocus={navigation.canRestoreFocus}
        />
        <WorkspacePanel
          mainRef={navigation.workspaceMain}
          disclosureKey={navigation.visit}
          canRestoreFocus={navigation.canRestoreFocus}
          connection={connection}
          workspaces={workspaces}
          repositories={repositories}
          onRefreshStatus={async () => {
            await refreshStatus();
            await repositories.refresh();
            await workspaces.refresh();
          }}
        />
      </Activity>
      {navigation.screen === "settings" && (
        <SettingsScreen
          headingRef={navigation.settingsHeading}
          onBack={navigation.back}
          runtimes={runtimes}
        />
      )}
    </SidebarProvider>
  );
}
