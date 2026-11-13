---
title: "Keep provider content as data at the rendering boundary"
impact: HIGH
impactDescription: "Prevents transcript text from acquiring DOM or application authority"
tags: content, dom, tauri
---

# Keep provider content as data at the rendering boundary

Apply to transcript/tool output, paths, diffs, Markdown, and attachment metadata.
Render ordinary content as text. If a feature requires HTML/Markdown, select a
renderer with an explicit raw-HTML and URL policy; no renderer is chosen merely
by adding this rule.

**Incorrect (provider text is treated as trusted HTML):**

```tsx
<div dangerouslySetInnerHTML={{ __html: providerText }} />
```

**Correct (plain transcript content remains text):**

```tsx
<pre className="whitespace-pre-wrap">{providerText}</pre>
```

For richer content, review supported link schemes, image sources and native-open
callbacks against Tauri capabilities/CSP. Displayed links, file paths, or suggested
tool operations cannot authorize native commands or approval. Preserve exact
application-owned targets in explicit intent paths. This rule adds no sanitizer,
Markdown library, native permission, or remote-resource requirement.

Sources: [React raw HTML caveats](https://react.dev/reference/react-dom/components/common#dangerously-setting-the-inner-html),
[Tauri capabilities](https://v2.tauri.app/security/capabilities/).
