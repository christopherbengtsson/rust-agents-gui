import {
  useCallback,
  useRef,
  useState,
  useSyncExternalStore,
  type ReactNode,
} from "react";
import { CSPProvider } from "@base-ui/react/csp-provider";
import type { PanelImperativeHandle } from "react-resizable-panels";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import {
  Empty,
  EmptyHeader,
  EmptyTitle,
  EmptyDescription,
} from "@/components/ui/empty";
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from "@/components/ui/resizable";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
  SheetTrigger,
} from "@/components/ui/sheet";
import type { WorkspaceChangesReader } from "@/lib/workspace-changes-reader";
import { UnifiedDiff } from "@/components/primitives/unified-diff";
import {
  useWorkspaceChanges,
  type WorkspaceChanges,
} from "@/hooks/use-workspace-changes";
import {
  pendingInspection,
  unsupportedLabel,
  type ChangedFile,
} from "@/lib/changes";

const narrowQuery = "(max-width: 879px)";
function subscribeNarrow(changed: () => void) {
  const query = window.matchMedia(narrowQuery);
  query.addEventListener("change", changed);
  return () => query.removeEventListener("change", changed);
}
const readNarrow = () => window.matchMedia(narrowQuery).matches;

export function WorkspaceInspection({
  owner,
  workspaceId,
  reader,
  children,
}: {
  owner: string | undefined;
  workspaceId: string;
  reader: WorkspaceChangesReader;
  children: ReactNode;
}) {
  const [tab, setTab] = useState("workspace");
  const [collapsed, setCollapsed] = useState(false);
  const [sheet, setSheet] = useState(false);
  const narrow = useSyncExternalStore(subscribeNarrow, readNarrow);
  const changes = useWorkspaceChanges(
    owner,
    workspaceId,
    tab === "changes",
    tab === "changes" || (narrow ? sheet : !collapsed),
    reader,
  );
  const panel = useRef<PanelImperativeHandle>(null);
  const position = useRef({ top: 0, left: 0 });
  const savePosition = useCallback((value: { top: number; left: number }) => {
    position.current = value;
  }, []);
  const select = (file: ChangedFile) => {
    if (changes.selected?.id !== file.id)
      position.current = { top: 0, left: 0 };
    changes.select(file);
    setTab("changes");
    setSheet(false);
  };
  const list = <ChangedFiles changes={changes} onSelect={select} />;
  return (
    <CSPProvider disableStyleElements>
      <div className="workspace-inspection">
        <div className="changes-toolbar">
          <span>Uncommitted changes</span>
          <Button
            size="sm"
            variant="outline"
            disabled={
              !owner ||
              changes.loading ||
              (pendingInspection(changes.observation?.phase) && !changes.error)
            }
            onClick={() => void changes.refresh()}
          >
            {changes.loading ||
            (pendingInspection(changes.observation?.phase) && !changes.error)
              ? "Refreshing…"
              : "Refresh changes"}
          </Button>
          {narrow ? (
            <Sheet open={sheet} onOpenChange={setSheet}>
              <SheetTrigger render={<Button size="sm" variant="outline" />}>
                Changed files
              </SheetTrigger>
              <SheetContent>
                <SheetHeader>
                  <SheetTitle>Uncommitted changes</SheetTitle>
                  <SheetDescription>
                    Select a file to read its diff.
                  </SheetDescription>
                </SheetHeader>
                {list}
              </SheetContent>
            </Sheet>
          ) : (
            <Button
              size="sm"
              variant="ghost"
              aria-expanded={!collapsed}
              onClick={() => {
                if (collapsed) panel.current?.expand();
                else panel.current?.collapse();
              }}
            >
              {collapsed ? "Show changed files" : "Hide changed files"}
            </Button>
          )}
        </div>
        <InspectionStatus changes={changes} connected={!!owner} />
        <ResizablePanelGroup
          orientation="horizontal"
          className="inspection-panels"
        >
          <ResizablePanel id="inspection-main" minSize={180}>
            <Tabs
              value={tab}
              onValueChange={(value) => setTab(String(value))}
              className="inspection-tabs"
            >
              <TabsList aria-label="Workspace views">
                <TabsTrigger value="workspace">Workspace</TabsTrigger>
                <TabsTrigger value="changes">Changes</TabsTrigger>
              </TabsList>
              <TabsContent
                value="workspace"
                keepMounted={false}
                className="workspace-tab"
              >
                {children}
              </TabsContent>
              <TabsContent
                value="changes"
                keepMounted={false}
                className="changes-tab"
              >
                <SelectedDiff
                  changes={changes}
                  position={position}
                  onPosition={savePosition}
                />
              </TabsContent>
            </Tabs>
          </ResizablePanel>
          {!narrow && (
            <>
              <ResizableHandle aria-label="Resize changed files" withHandle />
              <ResizablePanel
                id="inspection-files"
                panelRef={panel}
                defaultSize="32%"
                minSize={170}
                maxSize="55%"
                collapsible
                collapsedSize={0}
                onResize={(size) => setCollapsed(size.asPercentage === 0)}
              >
                {!collapsed && list}
              </ResizablePanel>
            </>
          )}
        </ResizablePanelGroup>
      </div>
    </CSPProvider>
  );
}

function InspectionStatus({
  changes,
  connected,
}: {
  changes: WorkspaceChanges;
  connected: boolean;
}) {
  const data = changes.observation;
  return (
    <div className="inspection-status">
      {data?.page && (
        <p className="repository-path">
          Compared with HEAD <code>{data.page.head}</code> · {data.page.total}{" "}
          changed files
        </p>
      )}
      {(!connected || changes.error || data?.phase === "failed") && (
        <Alert variant="destructive">
          <AlertTitle>Change inspection needs attention</AlertTitle>
          <AlertDescription>
            <p>
              {changes.error ??
                data?.error?.message ??
                "Reconnect to retrieve workspace observations."}
            </p>
            {data?.page && (
              <p>Showing the last-good comparison; it may be stale.</p>
            )}
          </AlertDescription>
        </Alert>
      )}
      <p role="status">
        {pendingInspection(data?.phase)
          ? data?.page
            ? "Refreshing the retained comparison…"
            : "Loading workspace changes…"
          : data?.phase === "idle"
            ? "Refresh changes to inspect this workspace."
            : connected &&
                !changes.error &&
                data?.phase === "ready" &&
                data.page?.total === 0
              ? "No uncommitted changes."
              : !data
                ? "Retrieving inspection status…"
                : ""}
      </p>
    </div>
  );
}

export function ChangedFiles({
  changes,
  onSelect,
}: {
  changes: WorkspaceChanges;
  onSelect: (file: ChangedFile) => void;
}) {
  const page = changes.observation?.page;
  return (
    <section aria-label="Changed files" className="changed-files">
      <h2>Changed files</h2>
      <ScrollArea className="changed-file-scroll">
        <ul>
          {page?.files.map((file) => (
            <li key={file.id}>
              <Button
                variant="ghost"
                className="changed-file"
                aria-pressed={changes.selected?.id === file.id}
                title={file.path}
                onClick={() => onSelect(file)}
              >
                <span className="min-w-0 truncate">{file.path}</span>
                <Badge
                  variant="secondary"
                  title={`Staged: ${file.index_status}; unstaged: ${file.worktree_status}`}
                >
                  {file.index_status}
                  {file.worktree_status}
                </Badge>
                {file.unsupported && (
                  <span className="sr-only">
                    {unsupportedLabel(file.unsupported)}
                  </span>
                )}
              </Button>
              {file.unsupported && (
                <p className="unsupported-file">
                  {unsupportedLabel(file.unsupported)}
                </p>
              )}
            </li>
          ))}
        </ul>
      </ScrollArea>
      {page && page.total > 0 && (
        <div className="file-paging">
          <Button
            size="sm"
            variant="outline"
            disabled={changes.loading}
            onClick={() => void changes.first()}
          >
            First page
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={!page.next || changes.loading}
            onClick={() => void changes.next()}
          >
            Next files
          </Button>
        </div>
      )}
    </section>
  );
}

function SelectedDiff({
  changes,
  position,
  onPosition,
}: {
  changes: WorkspaceChanges;
  position: React.RefObject<{ top: number; left: number }>;
  onPosition: (position: { top: number; left: number }) => void;
}) {
  const p = changes.preview;
  const preview = p?.preview;
  if (!changes.selected)
    return (
      <Empty>
        <EmptyHeader>
          <EmptyTitle>Select a changed file</EmptyTitle>
          <EmptyDescription>
            Refresh the file list, then select a path to inspect its unified
            diff.
          </EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  return (
    <section className="selected-diff" aria-label="Selected file changes">
      <h2 className="repository-path">{changes.selected.path}</h2>
      {(pendingInspection(p?.phase) || (!p && !changes.error)) && (
        <p role="status">Loading selected-file preview…</p>
      )}
      {p?.error && (
        <Alert variant="destructive">
          <AlertTitle>Preview unavailable</AlertTitle>
          <AlertDescription>{p.error.message}</AlertDescription>
        </Alert>
      )}
      {preview && (
        <>
          {(preview.comparison_id !==
            changes.observation?.page?.comparison_id ||
            changes.observation?.phase === "failed" ||
            p.phase === "failed" ||
            changes.error) && (
            <p>
              Showing the last-good preview against HEAD {preview.head}; it may
              be stale.
            </p>
          )}
          {preview.unsupported && (
            <p>Preview unsupported: {unsupportedLabel(preview.unsupported)}.</p>
          )}
          {preview.limited && <p role="status">{preview.limited}</p>}
          {preview.note && <p>{preview.note}</p>}
          {preview.rows.length > 0 && (
            <UnifiedDiff
              key={changes.selected.id}
              rows={preview.rows}
              position={position}
              onPosition={onPosition}
            />
          )}
        </>
      )}
    </section>
  );
}
