import { Layers2 } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
} from "@/components/ui/sidebar";
import type { Connection } from "@/hooks/use-supervisor-connection";
import { SupervisorConnection } from "./supervisor-connection";

export function RepositorySidebar({ connection }: { connection: Connection }) {
  return (
    <Sidebar
      collapsible="none"
      role="complementary"
      aria-label="Repository sidebar"
      className="repository-sidebar min-w-0 shrink-0 border-r"
    >
      <SidebarHeader className="h-(--shell-header-height) shrink-0 justify-center border-b px-4 py-0 [@media(max-width:700px)]:px-3">
        <div className="app-identity flex items-center gap-2">
          <Layers2 aria-hidden="true" />
          <span>Agent Workspace</span>
        </div>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup className="px-4 py-[22px]">
          <nav aria-labelledby="repositories-heading">
            <SidebarGroupLabel
              render={
                // Base UI supplies the visible children through render composition.
                // eslint-disable-next-line jsx-a11y/heading-has-content
                <h2 id="repositories-heading" />
              }
              className="h-auto px-0"
            >
              Repositories
            </SidebarGroupLabel>
            <SidebarGroupContent>
              <p className="navigation-empty">No repositories yet</p>
            </SidebarGroupContent>
          </nav>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="connection-footer shrink-0">
        <SupervisorConnection connection={connection} />
      </SidebarFooter>
    </Sidebar>
  );
}
