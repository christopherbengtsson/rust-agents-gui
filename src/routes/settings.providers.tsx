import { createFileRoute } from "@tanstack/react-router";
import { useAppRuntime } from "@/app-runtime";
import { ProviderSetup, ClaudeSetup } from "@/components/blocks/provider-setup";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { useProviderSetup } from "@/hooks/use-provider-setup";

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
  const { app, connection } = useAppRuntime();
  const owner =
    connection.state === "connected"
      ? connection.observation.snapshot.cursor.owner
      : undefined;
  const setup = useProviderSetup(app, owner);
  return <ProviderSetup setup={setup} />;
}
function ProvidersScreen() {
  const { provider } = Route.useSearch();
  const navigate = Route.useNavigate();
  return (
    <div className="workspace-content flex flex-col gap-5">
      <h2 id="settings-providers-heading" className="font-medium">
        Providers
      </h2>
      <Tabs
        value={provider}
        onValueChange={(value) => {
          if (value === "codex" || value === "claude")
            void navigate({ search: { provider: value }, replace: true });
        }}
      >
        <TabsList aria-label="Provider">
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
  );
}
