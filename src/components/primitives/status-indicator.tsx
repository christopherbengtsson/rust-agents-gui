import { cn } from "cn";

/** A status light and its accessible meaning; the caller interprets evidence. */
export function StatusIndicator({
  tone,
  label,
  className,
}: {
  tone: "success" | "neutral" | "warning" | "error";
  label: string;
  className?: string;
}) {
  return (
    <div
      role="status"
      aria-live="polite"
      aria-atomic="true"
      className={cn("status-indicator", className)}
      data-tone={tone}
    >
      <span className="status-indicator-light" aria-hidden="true" />
      <span>{label}</span>
    </div>
  );
}
