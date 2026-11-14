import { useEffect, useRef, type ReactNode } from "react";
import { Link } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";
import { Button, buttonVariants } from "@/components/ui/button";

/** Presentation only: the route supplies history behavior, never IPC policy. */
export function SettingsLayout({
  onBack,
  children,
}: {
  onBack: () => void;
  children: ReactNode;
}) {
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    heading.current?.focus();
  }, []);
  useEffect(() => {
    // Base UI overlays consume Escape at the element/document first. Listening
    // on window in the bubble phase lets them dismiss without leaving Settings.
    function escape(event: KeyboardEvent) {
      if (
        event.key !== "Escape" ||
        event.defaultPrevented ||
        event.isComposing ||
        event.repeat
      )
        return;
      event.preventDefault();
      onBack();
    }
    window.addEventListener("keydown", escape);
    return () => window.removeEventListener("keydown", escape);
  }, [onBack]);
  return (
    <div className="settings-layout flex min-h-0 min-w-0 flex-1 flex-col">
      <header className="workspace-header">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="sm" onClick={onBack}>
            <ArrowLeft aria-hidden="true" data-icon="inline-start" />
            Back
          </Button>
          <h1 ref={heading} tabIndex={-1}>
            Settings
          </h1>
        </div>
      </header>
      <div className="flex min-h-0 min-w-0 flex-1">
        <nav
          aria-label="Settings"
          className="settings-navigation flex shrink-0 flex-col gap-2 border-r p-3"
        >
          <Link
            to="/settings/general"
            className={buttonVariants({
              variant: "ghost",
              size: "sm",
              className: "justify-start",
            })}
            activeProps={{
              "aria-current": "page",
              className: "bg-accent text-accent-foreground",
            }}
          >
            General
          </Link>
        </nav>
        <main
          id="settings-main"
          tabIndex={-1}
          aria-labelledby="settings-general-heading"
          className="workspace-region flex-1"
        >
          {children}
        </main>
      </div>
    </div>
  );
}
