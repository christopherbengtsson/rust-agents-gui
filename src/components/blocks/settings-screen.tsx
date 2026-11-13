import { useRef, type Ref } from "react";
import { ArrowLeft, Bot } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
} from "@/components/ui/sidebar";

export function SettingsScreen({
  headingRef,
  onBack,
}: {
  headingRef: Ref<HTMLHeadingElement>;
  onBack: () => void;
}) {
  const categoryHeading = useRef<HTMLHeadingElement>(null);
  return (
    <div className="settings-screen">
      <header className="settings-header">
        <h1 ref={headingRef} tabIndex={-1}>
          Settings
        </h1>
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
            <section
              aria-labelledby="codex-settings"
              className="runtime-settings"
            >
              <h3 id="codex-settings">Codex</h3>
              <p className="muted-copy">No setup diagnostic has run.</p>
            </section>
          </section>
        </main>
      </div>
    </div>
  );
}
