import { SlidersHorizontal } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import type {
  Connection,
  Observation,
} from "@/hooks/use-supervisor-connection";

export function SupervisorConnection({
  connection,
}: {
  connection: Connection;
}) {
  const label =
    connection.state === "connected"
      ? "Connected"
      : connection.state === "checking"
        ? "Checking connection…"
        : "Connection error";
  return (
    <div className="connection-summary">
      <div
        role="status"
        aria-live="polite"
        className="connection-state"
        data-state={connection.state}
      >
        <span className="status-dot" aria-hidden="true" />
        {label}
      </div>
      <Dialog>
        <DialogTrigger
          render={
            <Button
              variant="ghost"
              size="sm"
              className="w-full justify-start"
            />
          }
        >
          <SlidersHorizontal data-icon="inline-start" />
          Connection details
        </DialogTrigger>
        <DialogContent className="max-h-[calc(100dvh-3rem)] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Supervisor connection</DialogTitle>
            <DialogDescription>
              Local connection and process diagnostics.
            </DialogDescription>
          </DialogHeader>
          <Badge
            variant={connection.state === "error" ? "destructive" : "secondary"}
          >
            {label}
          </Badge>
          {connection.state === "error" && (
            <p className="error-detail">{connection.error}</p>
          )}
          {connection.observation ? (
            <ConnectionDetails
              observation={connection.observation}
              current={connection.state === "connected"}
            />
          ) : (
            <p className="muted-copy">No successful observation yet.</p>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}

function ConnectionDetails({
  observation: { snapshot, desktopPid },
  current,
}: {
  observation: Observation;
  current: boolean;
}) {
  const fields = [
    ["Supervisor process", snapshot.status.supervisor_pid],
    ["Desktop process", desktopPid],
    ["Application version", snapshot.status.application_version],
    ["Owner identity", snapshot.cursor.owner],
    ["Snapshot revision", snapshot.cursor.revision],
  ];
  return (
    <section
      aria-label={
        current ? "Current observation" : "Last successful observation"
      }
    >
      <h3 className="section-label">
        {current
          ? "Current observation"
          : "Last successful observation · may be stale"}
      </h3>
      <dl className="diagnostic-fields">
        {fields.map(([label, value]) => (
          <div key={label}>
            <dt>{label}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
