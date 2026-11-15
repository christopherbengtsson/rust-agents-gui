import { useState } from "react";
import { openProviderHelp, type ProviderHelpTopic } from "@/lib/provider-help";
import { createFileRoute } from "@tanstack/react-router";
import { useAppRuntime } from "@/app-runtime";
import { ProviderSetup, ClaudeSetup } from "@/components/blocks/provider-setup";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { useProviderSetup } from "@/hooks/use-provider-setup";
import { useProviderLogin } from "@/hooks/use-provider-progress";
import { ProviderSignIn } from "@/components/blocks/provider-sign-in";

export function providerSearch(search: Record<string, unknown>): {
  provider: "codex" | "claude";
} {
  if (search.provider === undefined) return { provider: "codex" };
  if (search.provider === "codex" || search.provider === "claude")
    return { provider: search.provider };
  throw new Error(
    "Invalid provider. Open Codex or Claude Code in Providers settings.",
  );
}
export const Route = createFileRoute("/settings/providers")({
  validateSearch: providerSearch,
  loaderDeps: ({ search }) => ({ provider: search.provider }),
  loader: ({ context: { app }, deps, abortController }) => {
    const owner = app.freshness.currentOwner();
    if (owner && deps.provider === "codex")
      void app.providers.prepare(
        owner,
        { operation: "target" },
        abortController.signal,
      );
  },
  component: ProvidersScreen,
});
function CodexSetup() {
  const [helpError, setHelpError] = useState<string>();
  const { app, connection } = useAppRuntime();
  const owner =
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined;
  const updatesAvailable =
    connection.feed?.state !== "stopped" && !connection.feed?.message;
  const setup = useProviderSetup(app, owner, updatesAvailable);
  const login = useProviderLogin(app, owner, setup.observation);
  const openHelp = (topic: ProviderHelpTopic) => {
    setHelpError(undefined);
    void openProviderHelp(topic).catch(() =>
      setHelpError(
        "The documentation could not be opened. Use developers.openai.com/codex in your browser.",
      ),
    );
  };
  return (
    <ProviderSetup
      setup={setup}
      onOpenHelp={openHelp}
      helpError={helpError}
      signIn={
        <ProviderSignIn
          observation={setup.observation}
          available={!!owner && updatesAvailable}
          login={login}
        />
      }
    />
  );
}
function ProvidersScreen() {
  const { provider } = Route.useSearch();
  const navigate = Route.useNavigate();
  return (
    <div className="workspace-content flex flex-col gap-5">
      <div className="providers-content">
        <h2 id="settings-providers-heading" className="settings-page-heading">
          Providers
        </h2>
        <Tabs
          value={provider}
          onValueChange={(value) => {
            if (value === "codex" || value === "claude")
              void navigate({ search: { provider: value }, replace: true });
          }}
        >
          <TabsList
            aria-label="Provider"
            variant="line"
            className="provider-tabs"
          >
            <TabsTrigger value="codex">Codex</TabsTrigger>
            <TabsTrigger value="claude">Claude Code</TabsTrigger>
          </TabsList>
          <TabsContent value="codex" className="pt-4">
            {provider === "codex" && <CodexSetup />}
          </TabsContent>
          <TabsContent value="claude" className="pt-4">
            {provider === "claude" && <ClaudeSetup />}
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}
