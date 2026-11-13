---
title: "Bound stream processing and rendered history"
impact: HIGH
impactDescription: "Keeps growing observations from overwhelming the WebView"
tags: streams, history, ordering
---

# Bound stream processing and rendered history

Apply to streamed text/tool events, long history, and large diffs. Bound retained
data and rendered DOM, localize frequent updates, and budget in-flight work. Choose
paging/windowing/virtualization with the feature's accessibility and scroll needs.

**Incorrect (every event retains and renders the entire session forever):**

```tsx
setRows(previous => [...previous, incoming]);
return rows.map(row => <HistoryRow key={row.id} row={row} />);
```

**Correct (illustrative paged presentation exposes a bounded window):**

```tsx
const { page, loadOlder, hasOlder } = useHistoryPage(conversationId);
return <>
  {hasOlder && <button onClick={loadOlder}>Load earlier messages</button>}
  {page.map(row => <HistoryRow key={row.id} row={row} />)}
</>;
```

The example assumes a bounded adapter, not a new storage contract. Authoritative
history and snapshot/cursor recovery remain Rust-owned. Async event callbacks can
finish out of order: preserve sequence/target identity and use the supported
ordered transport when required. Ordered delivery alone does not serialize async
processing; choose explicit processing order or sequence checks. CSS content visibility is optional progressive
enhancement, does not bound DOM/data, and cannot be relied on at Safari 16.4.

Sources: [Tauri channels/events](https://v2.tauri.app/develop/calling-frontend/),
[WebKit content visibility](https://webkit.org/blog/15865/webkit-features-in-safari-18-0/),
[MVP stream bounds](../../../../docs/MVP.md#starting-architecture).
