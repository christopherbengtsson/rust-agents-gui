---
title: "Integrate Base UI injected elements with the desktop CSP"
impact: HIGH
impactDescription: "Avoids broken controls or blanket policy relaxation"
tags: base-ui, csp, tauri
---

# Integrate Base UI injected elements with the desktop CSP

Apply when adding controls that inject style/script elements, such as relevant
Scroll Area or aligned Select configurations. Inspect the current Tauri CSP and
installed Base UI docs before selecting integration behavior.

**Incorrect (broadly allowing inline code to silence a component issue):**

```text
script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'
```

**Correct (one documented option when equivalent external CSS is supplied):**

```tsx
import { CSPProvider } from "@base-ui/react/csp-provider";
<CSPProvider disableStyleElements>{controls}</CSPProvider>;
```

Disabling injected style elements requires supplying their needed behavior in an
allowed stylesheet, then checking the actual controls. A matching nonce strategy
is another option only with a properly integrated CSP; inventing a fixed nonce is
not sufficient. `CSPProvider` governs injected elements, not ordinary client-side
JavaScript style assignments. The current app is client-rendered, so SSR style
attribute caveats must not be copied as a blanket prohibition. This rule authorizes
no automatic CSP, capability, or component changes.

Sources: [Base UI CSP Provider](https://base-ui.com/react/utils/csp-provider),
[Tauri CSP](https://v2.tauri.app/security/csp/).
