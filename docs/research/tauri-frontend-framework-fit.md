# Tauri-specific frontend framework fit

Historical follow-up to [presentation framework performance](presentation-framework-performance.md), based on primary sources and configuration inspection. Decision status is resolved by the accepted [React decision](../adr/0001-react-presentation-with-vite.md) and [component foundation decision](../adr/0002-shadcn-base-ui-presentation-components.md). The findings below retain the evidence considered before acceptance.

## What Tauri's frontend page actually recommends

Tauri is frontend-agnostic and serves built HTML/CSS/JavaScript and optionally WASM to a webview. Its recommended JavaScript path is Vite for SPA frameworks including React, Vue, Svelte and Solid. Server-based SSR is not natively supported. The page's framework-specific links describe configuration needs, not a ranked list of preferred frameworks. Source: [Frontend Configuration](https://v2.tauri.app/start/frontend/). The interpretation of the link list follows from the page's explanation of documented configurations.

For this local desktop, React + Vite is therefore a direct fit. Svelte or Solid + Vite would be equally ordinary integration choices. Adding a server-oriented framework such as Next.js is not needed for the accepted workflow; we have no product requirement for its server features. These are application recommendations, not comparative performance findings.

## macOS rendering environment

Tauri uses macOS's WKWebView/WebKit, with its version tied to OS updates. Source: [Webview Versions](https://v2.tauri.app/reference/webview-versions/).

Consequences for our choice: every candidate must be evaluated against the actual target macOS webview. A Chromium-only browser measurement does not establish desktop behavior. React's JavaScript runtime and a Rust/WASM frontend both ultimately operate within the web frontend arrangement; neither choice automatically substitutes native AppKit widgets. This is an architectural inference from Tauri's documented webview/static-asset model, not a measurement.

## Streaming and application ownership

Tauri documents that its general event system is not intended for low-latency/high-throughput delivery. It recommends channels for streaming, describes them as ordered, and requires cleanup of framework event listeners to prevent leaks/duplicate handlers. Source: [Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/).

For agent output, evaluate channels for the Tauri-to-frontend hop while maintaining bounded traffic and localized presentation updates. A channel does not establish durable delivery across supervisor failure or desktop reconnect. Authoritative history, IDs, action status and reconciliation remain in the supervisor's owning Rust modules; Tauri remains a forwarding/presentation adapter. Source for those ownership requirements: [MVP](../MVP.md). This research proposes no replacement of the separate supervisor transport and no promised exactly-once behavior.

## Rust/WASM is possible, with distinct tradeoffs

Tauri has a Leptos integration guide, including a wasm-bindgen bridge to Tauri APIs. That guide explicitly describes itself as accurate for Leptos 0.6, so its existence is integration evidence rather than a current-version implementation recipe. Source: [Tauri Leptos guide](https://v2.tauri.app/start/frontend/leptos/).

Tauri also documents a CSP requirement for WASM execution. Source: [Tauri CSP](https://v2.tauri.app/security/csp/). Rust in the frontend would add browser/WASM build and interop concerns to this project; sharing the Rust language with the backend does not grant browser code direct access to the native supervisor's processes, SQLite connection or ownership. Those effects remain across the existing product interfaces. The latter is an inference from the chosen process architecture, not a limitation on all possible Leptos applications.

No source inspected establishes that using Rust/WASM would improve this app's UI latency, startup, memory or packaged size. Choose it if its authoring/typing model earns the additional tooling and component adaptation, rather than assuming a speed benefit from the Rust name.

## Current project fit

The existing [Tauri configuration](../../src-tauri/tauri.conf.json) already points development at Vite and production at built frontend assets. The [manifest](../../package.json) uses Vite/TypeScript/pnpm with no frontend framework installed. Moving to React, Svelte or Solid would extend this setup; Rust/WASM would introduce a different frontend build pipeline.

The app also has an explicit CSP. Evaluate any selected library's overlays, dynamic styling, resizing, fonts and optional WASM under that actual policy. This is a compatibility consideration, not an observed defect or permission to loosen policy broadly. Package frontend resources locally where appropriate; preserve the application-specific configuration rather than copying a documentation example wholesale. Source: [Tauri CSP](https://v2.tauri.app/security/csp/).

## Recommendation

React + TypeScript + Vite remains the recommended presentation foundation given the user's experience, the researched React component options, and the current project setup. Svelte + Vite is a credible alternative if the user prefers its granular reactivity/authoring model; Solid is likewise viable. Tauri itself supplies no reason to override the experience and component-fit considerations with an unmeasured framework ranking.

Keep performance work attached to real features: bounded streaming over suitable transport, limited mounted history/diffs, targeted view updates, and actual release-desktop observations of navigation, typing, scrolling and history opening. Existing product targets and the [verification policy](../agents/verification.md) remain authoritative. No new test suite, benchmark, app run or provider allowance is introduced by this recommendation.

## Evidence limits

No candidate was built or benchmarked. No installed dependency compatibility, CSP behavior, end-to-end streaming latency or WKWebView interaction behavior was exercised. Findings distinguish documented Tauri mechanisms from our application recommendations. Only this research note was added.
