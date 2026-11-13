import {
  ChevronDown,
  FolderGit2,
  Layers2,
  Plus,
  GitBranch,
  Settings,
} from "lucide-react";
import type { Ref } from "react";
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
  SidebarMenuSub,
  SidebarMenuSubItem,
  SidebarMenuSubButton,
} from "@/components/ui/sidebar";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { Workspaces } from "@/hooks/use-workspaces";
import { Spinner } from "@/components/ui/spinner";
import { workspaceStatus } from "@/lib/workspaces";
import type { Repositories } from "@/hooks/use-repositories";
import { RepositoryRegistration } from "./repository-registration";
import type { Connection } from "@/hooks/use-supervisor-connection";
import { SupervisorConnection } from "./supervisor-connection";

export function RepositorySidebar({
  connection,
  repositories,
  workspaces,
  onSettings,
  settingsRef,
  disclosureKey,
  canRestoreFocus,
}: {
  connection: Connection;
  repositories: Repositories;
  workspaces: Workspaces;
  onSettings: () => void;
  settingsRef: Ref<HTMLButtonElement>;
  disclosureKey: number;
  canRestoreFocus: () => boolean;
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
                // oxlint-disable-next-line jsx-a11y/heading-has-content
                <h2 id="repositories-heading" />
              }
              className="h-auto px-0"
            >
              Repositories
            </SidebarGroupLabel>
            <SidebarGroupContent>
              {repositories.loading &&
              !repositories.page &&
              connection.state !== "error" ? (
                <div aria-label="Loading repositories">
                  <SidebarMenuSkeleton showIcon />
                  <SidebarMenuSkeleton showIcon />
                </div>
              ) : !repositories.page?.repositories.length ? (
                <p className="navigation-empty">
                  {repositories.error || connection.state === "error"
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
                          className="pr-14!"
                          isActive={
                            repositories.selected?.id === repository.id &&
                            !workspaces.selected
                          }
                          aria-current={
                            repositories.selected?.id === repository.id &&
                            !workspaces.selected
                              ? "true"
                              : undefined
                          }
                          onClick={() => {
                            workspaces.selectRepository();
                            repositories.select(repository);
                          }}
                          title={repository.path}
                        >
                          <FolderGit2 aria-hidden="true" />
                          <span>{repository.name}</span>
                        </SidebarMenuButton>
                        <CollapsibleTrigger
                          render={<SidebarMenuAction className="right-8" />}
                          aria-label={`Toggle details for ${repository.name}`}
                        >
                          <ChevronDown
                            className="transition-transform in-data-panel-open:rotate-180"
                            aria-hidden="true"
                          />
                        </CollapsibleTrigger>
                        <SidebarMenuAction
                          aria-label={`Create workspace in ${repository.name}`}
                          disabled={
                            connection.state !== "connected" ||
                            workspaces.submitting ||
                            !!workspaces.pending ||
                            repository.availability.state !== "available"
                          }
                          onClick={() => {
                            repositories.select(repository);
                            void workspaces.create(repository.id);
                          }}
                        >
                          <Plus aria-hidden="true" />
                        </SidebarMenuAction>
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
                          {!!workspaces.pages[repository.id]?.workspaces
                            .length && (
                            <SidebarMenuSub>
                              {workspaces.pages[repository.id].workspaces.map(
                                (workspace) => (
                                  <SidebarMenuSubItem key={workspace.id}>
                                    <SidebarMenuSubButton
                                      render={<button type="button" />}
                                      isActive={
                                        workspaces.selected?.id === workspace.id
                                      }
                                      aria-current={
                                        workspaces.selected?.id === workspace.id
                                          ? "true"
                                          : undefined
                                      }
                                      title={workspaceStatus(workspace.status)}
                                      onClick={() => {
                                        repositories.select(repository);
                                        workspaces.select(workspace);
                                      }}
                                    >
                                      {workspace.status.state === "queued" ||
                                      workspace.status.state === "running" ? (
                                        <Spinner aria-hidden="true" />
                                      ) : (
                                        <GitBranch aria-hidden="true" />
                                      )}
                                      <span>
                                        {workspace.name ?? "New workspace"}
                                      </span>
                                    </SidebarMenuSubButton>
                                  </SidebarMenuSubItem>
                                ),
                              )}
                            </SidebarMenuSub>
                          )}
                          {(workspaces.hasPrevious(repository.id) ||
                            workspaces.pages[repository.id]?.next) && (
                            <div
                              className="flex gap-1 px-2 pb-2"
                              aria-label={`Workspace pages for ${repository.name}`}
                            >
                              <Button
                                size="xs"
                                variant="ghost"
                                disabled={
                                  !workspaces.hasPrevious(repository.id) ||
                                  workspaces.loading
                                }
                                onClick={() => workspaces.back(repository.id)}
                              >
                                Previous
                              </Button>
                              <Button
                                size="xs"
                                variant="ghost"
                                disabled={
                                  !workspaces.pages[repository.id]?.next ||
                                  workspaces.loading
                                }
                                onClick={() => workspaces.next(repository.id)}
                              >
                                Next
                              </Button>
                            </div>
                          )}
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
          key={`registration-${disclosureKey}`}
          canRestoreFocus={canRestoreFocus}
          repositories={repositories}
          connected={connection.state === "connected"}
        />
        <Button
          ref={settingsRef}
          variant="ghost"
          size="sm"
          className="w-full justify-start"
          onClick={onSettings}
        >
          <Settings data-icon="inline-start" aria-hidden="true" />
          Settings
        </Button>
        <SupervisorConnection
          key={`connection-${disclosureKey}`}
          connection={connection}
          canRestoreFocus={canRestoreFocus}
        />
      </SidebarFooter>
    </Sidebar>
  );
}
