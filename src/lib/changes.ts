import { bounded, domainError } from "./observation-contracts";
import type {
  ChangedFile,
  DiffRow,
  InspectionPhase,
  UnsupportedContent,
  WorkspaceRequest,
  WorkspaceReply,
  ChangesObservation,
  PreviewObservation,
} from "@/generated/contracts/types";
import { validateWorkspaceReply } from "@/generated/contracts/validators.js";
export type {
  InspectionPhase,
  UnsupportedContent,
  ChangedFile,
  ChangePage,
  ChangesObservation,
  PreviewTarget,
  DiffRow,
  FilePreview,
  PreviewObservation,
} from "@/generated/contracts/types";
export type InspectionRequest = Extract<
  WorkspaceRequest,
  { operation: "changes" | "change_preview" }
>;
export type InspectionReply = Extract<
  WorkspaceReply,
  { kind: "changes" | "change_preview" }
>;

const head = (value: string) => /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/i.test(value);
const safeCount = (value: number) => Number.isSafeInteger(value) && value >= 0;
const line = (value: number | null) =>
  value === null || (safeCount(value) && value > 0);
function statusSemantics(
  value: ChangesObservation | PreviewObservation,
): boolean {
  return (
    (value.error === null || domainError(value.error)) &&
    (value.phase !== "failed" || value.error !== null) &&
    // The Rust incarnation retains the previous error while a retry is queued/running.
    ((value.phase !== "idle" && value.phase !== "ready") ||
      value.error === null)
  );
}
function fileSemantics(value: ChangedFile): boolean {
  return (
    value.path.length > 0 &&
    /^[.MADRCUT?!]$/.test(value.index_status) &&
    /^[.MADRCUT?!]$/.test(value.worktree_status)
  );
}
function rowSemantics(value: DiffRow): boolean {
  if (!line(value.old_line) || !line(value.new_line)) return false;
  switch (value.kind) {
    case "hunk":
    case "no_newline":
      return value.old_line === null && value.new_line === null;
    case "context":
      return value.old_line !== null && value.new_line !== null;
    case "added":
      return value.old_line === null && value.new_line !== null;
    case "removed":
      return value.old_line !== null && value.new_line === null;
  }
}
export function validInspectionReply(
  request: InspectionRequest,
  value: unknown,
): value is InspectionReply {
  if (
    !validateWorkspaceReply(value) ||
    (value.kind !== "changes" && value.kind !== "change_preview") ||
    !statusSemantics(value.observation) ||
    !bounded(value, 48 * 1024)
  )
    return false;
  if (request.operation === "changes" && value.kind === "changes") {
    const observation = value.observation;
    if (observation.workspace_id !== request.workspace_id) return false;
    const page = observation.page;
    return page === null
      ? !request.comparison_id &&
          !request.after &&
          observation.phase !== "ready"
      : (!request.comparison_id ||
          page.comparison_id === request.comparison_id) &&
          head(page.head) &&
          page.files.length <= 32 &&
          page.files.every(fileSemantics) &&
          new Set(page.files.map((file) => file.id)).size ===
            page.files.length &&
          safeCount(page.total) &&
          page.total >= page.files.length &&
          (page.next === null ||
            (page.files.length > 0 && page.next === page.files.at(-1)?.id)) &&
          (page.total !== 0 || (page.files.length === 0 && page.next === null));
  }
  if (request.operation !== "change_preview" || value.kind !== "change_preview")
    return false;
  const observation = value.observation;
  if (
    observation.target.workspace_id !== request.target.workspace_id ||
    observation.target.comparison_id !== request.target.comparison_id ||
    observation.target.file_id !== request.target.file_id
  )
    return false;
  const preview = observation.preview;
  return preview === null
    ? observation.phase !== "ready"
    : (observation.phase !== "ready" ||
        preview.comparison_id === request.target.comparison_id) &&
        head(preview.head) &&
        preview.rows.length <= 200 &&
        preview.rows.every(rowSemantics);
}
export const pendingInspection = (phase: InspectionPhase | undefined) =>
  phase === "queued" || phase === "running";
export function unsupportedLabel(reason: UnsupportedContent) {
  return {
    binary: "Binary content",
    non_utf8: "Non-UTF-8 path or content",
    symlink: "Symbolic link",
    submodule: "Submodule",
    conflict: "Unresolved conflict",
    special_file: "Special file",
    oversized: "Content exceeds the read limit",
    preview_limit: "A row exceeds the preview limit",
  }[reason];
}
