import { useLayoutEffect, useRef } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  CodeBlock,
  CodeBlockContent,
  CodeBlockLine,
} from "@/components/ui/code-block";
import type { DiffRow } from "@/lib/changes";

// Git supplies both source columns; Code Block's sequential counter stays off.
export function UnifiedDiff({
  rows,
  initialPosition,
  onPosition,
}: {
  rows: DiffRow[];
  initialPosition: { top: number; left: number };
  onPosition: (position: { top: number; left: number }) => void;
}) {
  const position = useRef(initialPosition);
  const root = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const viewport = root.current?.querySelector<HTMLElement>(
      "[data-slot=scroll-area-viewport]",
    );
    if (!viewport) return;
    viewport.scrollTop = position.current.top;
    viewport.scrollLeft = position.current.left;
    let pending: { top: number; left: number } | undefined;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const flush = () => {
      clearTimeout(timer);
      timer = undefined;
      if (pending) onPosition(pending);
      pending = undefined;
    };
    const save = () => {
      pending = { top: viewport.scrollTop, left: viewport.scrollLeft };
      // Bounded write cadence while scrolling; release flushes the last position.
      if (timer === undefined) timer = setTimeout(flush, 150);
    };
    viewport.addEventListener("scroll", save, { passive: true });
    return () => {
      viewport.removeEventListener("scroll", save);
      flush();
    };
  }, [onPosition]);
  return (
    <ScrollArea ref={root} className="diff-scroll">
      <CodeBlock size="sm" className="diff-code">
        <CodeBlockContent
          className="diff-content"
          aria-label="Unified file diff"
        >
          {rows.map((row, index) => (
            <CodeBlockLine
              key={index}
              variant={
                row.kind === "added"
                  ? "added"
                  : row.kind === "removed"
                    ? "removed"
                    : "default"
              }
              className="diff-row"
              data-kind={row.kind}
            >
              <span className="diff-number" aria-label="Old line">
                {row.old_line ?? ""}
              </span>
              <span className="diff-number" aria-label="New line">
                {row.new_line ?? ""}
              </span>
              <span className="diff-sign" aria-hidden="true">
                {row.kind === "added"
                  ? "+"
                  : row.kind === "removed"
                    ? "−"
                    : " "}
              </span>
              <span>{row.text || "\n"}</span>
            </CodeBlockLine>
          ))}
        </CodeBlockContent>
      </CodeBlock>
    </ScrollArea>
  );
}
