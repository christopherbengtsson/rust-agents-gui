import { useRef, type Ref } from "react";
import { ArrowLeft, Bot } from "lucide-react";
import { Button } from "@/components/ui/button";
import { CodexSettings } from "./codex-settings";
import type { Runtimes } from "@/hooks/use-runtimes";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
} from "@/components/ui/sidebar";

export function SettingsScreen({
  headingRef,
  onBack,
  runtimes,
}: {
  headingRef: Ref<HTMLHeadingElement>;
  onBack: () => void;
  runtimes: Runtimes;
}) {
  const categoryHeading = useRef<HTMLHeadingElement>(null);
  return (
    <div className="settings-screen">
      <header className="settings-header">
        <Button variant="ghost" size="sm" onClick={onBack}>
          <ArrowLeft data-icon="inline-start" aria-hidden="true" />
          Back
        </Button>
      </header>
      <div className="settings-body">
        <Sidebar
          collapsible="none"
          className="repository-sidebar shrink-0 border-r"
        >
          <SidebarHeader className="h-(--shell-header-height) shrink-0 justify-center border-b px-4 py-0 [@media(max-width:700px)]:px-3">
            <h1 ref={headingRef} tabIndex={-1} className="app-identity">
              Settings
            </h1>
          </SidebarHeader>
          <SidebarContent>
            <SidebarGroup>
              <nav aria-label="Settings categories">
                <SidebarGroupContent>
                  <SidebarMenu>
                    <SidebarMenuItem>
                      <SidebarMenuButton
                        type="button"
                        size="sm"
                        isActive
                        aria-current="page"
                        onClick={() => categoryHeading.current?.focus()}
                      >
                        <Bot aria-hidden="true" />
                        Agent runtimes
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                  </SidebarMenu>
                </SidebarGroupContent>
              </nav>
            </SidebarGroup>
          </SidebarContent>
        </Sidebar>
        <main id="settings" tabIndex={-1} className="settings-content">
          <section aria-labelledby="agent-runtimes">
            <h2 ref={categoryHeading} id="agent-runtimes" tabIndex={-1}>
              Agent runtimes
            </h2>
            <CodexSettings runtimes={runtimes} />
          </section>
        </main>
      </div>
    </div>
  );
}
