# Application observation corpus

`observations-v6.json` is a small synthetic application-owned protocol-6 corpus,
introduced by F02 against application version 0.1.0. It is not captured provider
traffic and establishes no native runtime behavior. Native request framing and
Tauri's owner/result/error boundary are checked separately in connection units.

Pure Rust tests construct typed requests, snapshots, replies and failures and compare
their serialization with these literals. TypeScript tests consume the same literals
through production decoders, with separate malformed/target/authority scenarios.
Keep semantic guards when changing or generating declarations in the future.
