import { createFileRoute, Link } from "@tanstack/react-router";
export const Route = createFileRoute("/settings/general")({
  component: GeneralPlaceholder,
});
function GeneralPlaceholder() {
  return (
    <main id="workspace" tabIndex={-1} className="workspace-region flex-1">
      <header className="workspace-header">
        <h1>Settings</h1>
      </header>
      <div className="workspace-content">
        <h2>General</h2>
        <p>General settings will be available here.</p>
        <Link to="/">Return home</Link>
      </div>
    </main>
  );
}
