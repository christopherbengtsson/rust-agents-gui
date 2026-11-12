# React presentation with TypeScript and Vite

Use React with TypeScript and Vite for the Tauri desktop presentation. The user accepted this choice after comparing React, Svelte, Solid and Rust/WASM options: existing React experience and component fit favor it, while the research establishes no measured performance advantage for an alternative in this application.

Keep execution and recovery policy in the supervisor's Rust core under the [MVP contract](../MVP.md). Responsiveness depends on bounded data, localized view updates and actual desktop qualification; selecting React does not establish that performance targets pass. Component-library and styling choices remain separate decisions.

Evidence: [framework performance research](../research/presentation-framework-performance.md), [Tauri-specific fit](../research/tauri-frontend-framework-fit.md).
