import { createFileRoute } from "@tanstack/react-router";
import { Empty, EmptyHeader, EmptyDescription } from "@/components/ui/empty";
export const Route = createFileRoute("/settings/general")({
  component: GeneralPlaceholder,
});
function GeneralPlaceholder() {
  return (
    <div className="workspace-content">
      <h2 id="settings-general-heading" className="font-medium">
        General
      </h2>
      <Empty>
        <EmptyHeader>
          <EmptyDescription>
            General settings will be available here.
          </EmptyDescription>
        </EmptyHeader>
      </Empty>
    </div>
  );
}
