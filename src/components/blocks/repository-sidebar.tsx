import { ChevronDown, FolderGit2, Layers2 } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarMenuAction,
  SidebarMenuSkeleton,
} from "@/components/ui/sidebar";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { Repositories } from "@/hooks/use-repositories";
import { RepositoryRegistration } from "./repository-registration";
import type { Connection } from "@/hooks/use-supervisor-connection";
import { SupervisorConnection } from "./supervisor-connection";

export function RepositorySidebar({
  connection,
  repositories,
}: {
  connection: Connection;
  repositories: Repositories;
}) {
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
              {repositories.loading && !repositories.page ? (
                <div aria-label="Loading repositories">
                  <SidebarMenuSkeleton showIcon />
                  <SidebarMenuSkeleton showIcon />
                </div>
              ) : !repositories.page?.repositories.length ? (
                <p className="navigation-empty">
                  {repositories.error
                    ? "Repository list unavailable"
                    : "No repositories yet"}
                </p>
              ) : (
                <SidebarMenu className="mt-3">
                  {repositories.page.repositories.map((repository) => (
                    <SidebarMenuItem key={repository.id}>
                      <Collapsible defaultOpen>
                        <SidebarMenuButton
                          size="sm"
                          isActive={repositories.selected?.id === repository.id}
                          aria-current={
                            repositories.selected?.id === repository.id
                              ? "true"
                              : undefined
                          }
                          onClick={() => repositories.select(repository)}
                          title={repository.path}
                        >
                          <FolderGit2 aria-hidden="true" />
                          <span>{repository.name}</span>
                        </SidebarMenuButton>
                        <CollapsibleTrigger
                          render={<SidebarMenuAction />}
                          aria-label={`Toggle details for ${repository.name}`}
                        >
                          <ChevronDown
                            className="transition-transform in-data-panel-open:rotate-180"
                            aria-hidden="true"
                          />
                        </CollapsibleTrigger>
                        <CollapsibleContent>
                          <div className="repository-details flex flex-col items-start gap-1 px-2 pb-3 pt-1">
                            <p className="repository-path">{repository.path}</p>
                            <Badge
                              variant={
                                repository.availability.state === "unavailable"
                                  ? "destructive"
                                  : "secondary"
                              }
                            >
                              {repository.availability.state === "checking"
                                ? "Checking…"
                                : repository.availability.state === "available"
                                  ? "Available"
                                  : "Unavailable"}
                            </Badge>
                            {repository.availability.state ===
                              "unavailable" && (
                              <p className="repository-path">
                                {repository.availability.error.message}
                              </p>
                            )}
                          </div>
                        </CollapsibleContent>
                      </Collapsible>
                    </SidebarMenuItem>
                  ))}
                </SidebarMenu>
              )}
              {(repositories.hasPrevious || repositories.page?.next) && (
                <div className="flex gap-1 pt-2" aria-label="Repository pages">
                  <Button
                    variant="ghost"
                    size="xs"
                    disabled={!repositories.hasPrevious || repositories.loading}
                    onClick={repositories.back}
                  >
                    Previous
                  </Button>
                  <Button
                    variant="ghost"
                    size="xs"
                    disabled={!repositories.page?.next || repositories.loading}
                    onClick={repositories.next}
                  >
                    Next
                  </Button>
                </div>
              )}
            </SidebarGroupContent>
          </nav>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter className="connection-footer shrink-0">
        <RepositoryRegistration
          repositories={repositories}
          connected={connection.state === "connected"}
        />
        <SupervisorConnection connection={connection} />
      </SidebarFooter>
    </Sidebar>
  );
}
