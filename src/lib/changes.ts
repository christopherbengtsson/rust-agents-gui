import { identity, object, type RepositoryError } from "./repositories";

export type InspectionPhase =
  | "idle"
  | "queued"
  | "running"
  | "ready"
  | "failed";
export type UnsupportedContent =
  | "binary"
  | "non_utf8"
  | "symlink"
  | "submodule"
  | "conflict"
  | "special_file"
  | "oversized"
  | "preview_limit";
export type ChangedFile = {
  id: string;
  path: string;
  index_status: string;
  worktree_status: string;
  unsupported: UnsupportedContent | null;
};
export type ChangePage = {
  comparison_id: string;
  head: string;
  files: ChangedFile[];
  total: number;
  next: string | null;
};
export type ChangesObservation = {
  workspace_id: string;
  phase: InspectionPhase;
  page: ChangePage | null;
  error: RepositoryError | null;
};
export type PreviewTarget = {
  workspace_id: string;
  comparison_id: string;
  file_id: string;
};
export type DiffRow = {
  kind: "hunk" | "context" | "added" | "removed" | "no_newline";
  old_line: number | null;
  new_line: number | null;
  text: string;
};
export type FilePreview = {
  comparison_id: string;
  head: string;
  rows: DiffRow[];
  limited: string | null;
  unsupported: UnsupportedContent | null;
  note: string | null;
};
export type PreviewObservation = {
  target: PreviewTarget;
  phase: InspectionPhase;
  preview: FilePreview | null;
  error: RepositoryError | null;
};
export type InspectionRequest =
  | {
      operation: "changes";
      workspace_id: string;
      refresh: boolean;
      comparison_id: string | null;
      after: string | null;
    }
  | { operation: "change_preview"; target: PreviewTarget; load: boolean };
export type InspectionReply =
  | { kind: "changes"; observation: ChangesObservation }
  | { kind: "change_preview"; observation: PreviewObservation };

const unsupported = (v: unknown) =>
  v === null ||
  [
    "binary",
    "non_utf8",
    "symlink",
    "submodule",
    "conflict",
    "special_file",
    "oversized",
    "preview_limit",
  ].includes(String(v));
const nullableText = (v: unknown) => v === null || typeof v === "string";
const head = (v: unknown) =>
  typeof v === "string" && /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/i.test(v);
const number = (v: unknown) => Number.isSafeInteger(v) && Number(v) >= 0;
const line = (v: unknown) => v === null || (number(v) && Number(v) > 0);
function status(v: Record<string, unknown>) {
  return (
    ["idle", "queued", "running", "ready", "failed"].includes(
      String(v.phase),
    ) &&
    (v.error === null ||
      (object(v.error) &&
        typeof v.error.code === "string" &&
        typeof v.error.message === "string")) &&
    (v.phase !== "failed" || v.error !== null)
  );
}
function file(v: unknown): v is ChangedFile {
  return (
    object(v) &&
    identity(v.id) &&
    typeof v.path === "string" &&
    v.path.length > 0 &&
    typeof v.index_status === "string" &&
    /^[.MADRCUT?!]$/.test(v.index_status) &&
    typeof v.worktree_status === "string" &&
    /^[.MADRCUT?!]$/.test(v.worktree_status) &&
    unsupported(v.unsupported)
  );
}
function row(v: unknown): v is DiffRow {
  if (
    !object(v) ||
    typeof v.text !== "string" ||
    !line(v.old_line) ||
    !line(v.new_line)
  )
    return false;
  switch (v.kind) {
    case "hunk":
    case "no_newline":
      return v.old_line === null && v.new_line === null;
    case "context":
      return v.old_line !== null && v.new_line !== null;
    case "added":
      return v.old_line === null && v.new_line !== null;
    case "removed":
      return v.old_line !== null && v.new_line === null;
    default:
      return false;
  }
}
export function validInspectionReply(
  request: InspectionRequest,
  value: unknown,
): value is InspectionReply {
  if (
    !object(value) ||
    !object(value.observation) ||
    !status(value.observation) ||
    new TextEncoder().encode(JSON.stringify(value)).byteLength > 48 * 1024
  )
    return false;
  const v = value.observation;
  if (request.operation === "changes") {
    if (value.kind !== "changes" || v.workspace_id !== request.workspace_id)
      return false;
    const p = v.page;
    return (
      (p === null &&
        !request.comparison_id &&
        !request.after &&
        v.phase !== "ready") ||
      (object(p) &&
        identity(p.comparison_id) &&
        (!request.comparison_id || p.comparison_id === request.comparison_id) &&
        head(p.head) &&
        Array.isArray(p.files) &&
        p.files.length <= 32 &&
        p.files.every(file) &&
        new Set(p.files.map((f) => f.id)).size === p.files.length &&
        number(p.total) &&
        Number(p.total) >= p.files.length &&
        (p.next === null ||
          (identity(p.next) &&
            p.files.length > 0 &&
            p.next === p.files.at(-1)?.id)) &&
        (Number(p.total) !== 0 || (p.files.length === 0 && p.next === null)))
    );
  }
  if (
    value.kind !== "change_preview" ||
    !object(v.target) ||
    v.target.workspace_id !== request.target.workspace_id ||
    v.target.comparison_id !== request.target.comparison_id ||
    v.target.file_id !== request.target.file_id
  )
    return false;
  const p = v.preview;
  return (
    (p === null && v.phase !== "ready") ||
    (object(p) &&
      identity(p.comparison_id) &&
      (v.phase !== "ready" ||
        p.comparison_id === request.target.comparison_id) &&
      head(p.head) &&
      Array.isArray(p.rows) &&
      p.rows.length <= 200 &&
      p.rows.every(row) &&
      nullableText(p.limited) &&
      unsupported(p.unsupported) &&
      nullableText(p.note))
  );
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
