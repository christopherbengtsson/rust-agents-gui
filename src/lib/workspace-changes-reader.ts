import { workspaceRequest } from "./workspaces";
import {
  pendingInspection,
  type ChangedFile,
  type ChangesObservation,
  type PreviewObservation,
} from "./changes";
import { completeOperation } from "./complete-operation";
import { repositoryError } from "./repositories";

export type ChangesView = {
  observation?: ChangesObservation;
  preview?: PreviewObservation;
  selected?: Pick<ChangedFile, "id" | "path">;
  error?: string;
  loading: boolean;
};

type Context = {
  owner: string | undefined;
  workspaceId: string;
  active: boolean;
  visible: boolean;
};

// Presentation reads only. Rust owns comparison capture, admission and recovery.
// The lane survives view reconfiguration: invalidating a reply does not cancel IPC.
export class WorkspaceChangesReader {
  private context?: Context;
  private mounted = false;
  private generation = 0;
  private reading = false;
  private queued?: "observe" | "refresh";
  private timer?: ReturnType<typeof setTimeout>;
  private loaded?: string;
  private page: { comparison_id: string | null; after: string | null } = {
    comparison_id: null,
    after: null,
  };
  private view: ChangesView = { loading: false };

  private publish?: (view: ChangesView) => void;

  private update(value: Partial<ChangesView>) {
    this.view = { ...this.view, ...value };
    if (this.mounted) this.publish?.(this.view);
  }

  configure(context: Context, publish: (view: ChangesView) => void) {
    this.publish = publish;
    const previous = this.context;
    this.context = context;
    this.mounted = true;
    this.generation++;
    clearTimeout(this.timer);
    if (previous?.workspaceId !== context.workspaceId) {
      this.view = { loading: false };
      this.publish?.(this.view);
    }
    if (
      previous?.workspaceId !== context.workspaceId ||
      previous.owner !== context.owner
    ) {
      this.page = { comparison_id: null, after: null };
      this.loaded = undefined;
      this.queued = undefined;
    }
    if (this.observable()) void this.request(false);
    else this.queued = undefined;
  }

  dispose() {
    this.mounted = false;
    this.publish = undefined;
    this.generation++;
    this.queued = undefined;
    clearTimeout(this.timer);
  }

  private observable() {
    return this.mounted && this.context?.owner && this.context.visible;
  }

  private failed() {
    return (
      this.view.error ||
      this.view.observation?.phase === "failed" ||
      (this.view.observation?.phase === "ready" &&
        this.view.preview?.phase === "failed" &&
        this.view.preview.target.comparison_id ===
          this.view.observation.page?.comparison_id)
    );
  }

  select = (file: ChangedFile) => {
    if (this.view.selected?.id !== file.id) {
      this.generation++;
      this.loaded = undefined;
      this.update({
        selected: { id: file.id, path: file.path },
        preview: undefined,
      });
    }
    void this.request(false);
  };

  refresh = () => {
    this.generation++;
    return this.request(true);
  };

  first = () => {
    this.generation++;
    this.page = { comparison_id: null, after: null };
    return this.request(false);
  };

  next = () => {
    const page = this.view.observation?.page;
    if (!page?.next) return;
    this.generation++;
    this.page = { comparison_id: page.comparison_id, after: page.next };
    return this.request(false);
  };

  private request(refresh: boolean): Promise<void> {
    if (!this.mounted || !this.context?.owner) return Promise.resolve();
    clearTimeout(this.timer);
    // A view observation must not erase a queued explicit refresh.
    if (refresh || !this.queued) this.queued = refresh ? "refresh" : "observe";
    return this.drain();
  }

  private drain(): Promise<void> {
    if (this.reading || !this.queued || !this.mounted || !this.context?.owner)
      return Promise.resolve();
    const refresh = this.queued === "refresh";
    this.queued = undefined;
    this.reading = true;
    const { workspaceId, owner, active } = this.context;
    const generation = this.generation;
    const current = () =>
      this.mounted &&
      this.generation === generation &&
      this.context?.owner === owner &&
      this.context.workspaceId === workspaceId;
    if (refresh) {
      this.page = { comparison_id: null, after: null };
      this.loaded = undefined;
    }
    const page = { ...this.page };
    const selected = this.view.selected;
    this.update({ loading: true });
    return completeOperation(
      async () => {
        const result = await workspaceRequest({
          operation: "changes",
          workspace_id: workspaceId,
          refresh,
          ...page,
        });
        if (!current() || result.kind !== "changes") return;
        const observation = result.observation;
        this.update({ observation, error: undefined });
        if (
          !active ||
          !selected ||
          !observation.page ||
          observation.phase !== "ready"
        )
          return;
        const target = {
          workspace_id: workspaceId,
          comparison_id: observation.page.comparison_id,
          file_id: selected.id,
        };
        const key = `${target.comparison_id}/${target.file_id}`;
        const resultPreview = await workspaceRequest({
          operation: "change_preview",
          target,
          load: this.loaded !== key,
        });
        if (!current() || resultPreview.kind !== "change_preview") return;
        this.loaded = key;
        this.update({ preview: resultPreview.observation });
      },
      (failure) => {
        if (current()) this.update({ error: repositoryError(failure).message });
      },
      () => {
        // Release the actual lane even if its view generation became obsolete.
        this.reading = false;
        if (!this.mounted) return;
        this.update({ loading: false });
        if (this.queued) void this.drain();
        else if (this.observable() && !this.failed()) {
          const pending =
            pendingInspection(this.view.observation?.phase) ||
            (this.context?.active &&
              pendingInspection(this.view.preview?.phase));
          this.timer = setTimeout(
            () => void this.request(!pending),
            pending ? 250 : 2000,
          );
        }
      },
    );
  }
}
