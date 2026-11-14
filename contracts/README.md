# Application observation corpus

`observations-v6.json` is a small synthetic application-owned protocol-6 corpus,
introduced by F02 against application version 0.1.0. It is not captured provider
traffic and establishes no native runtime behavior. Native request framing and
Tauri's owner/result/error boundary are checked separately in connection units.

Pure Rust tests construct typed requests, snapshots, replies and failures and compare
their serialization with these literals. TypeScript tests consume the same literals
through production decoders, with separate malformed/target/authority scenarios.
Keep semantic guards when changing or generating declarations in the future.


F04 adds `observations-v7.json`: the protocol-7 corpus is synthetic and application-owned,
with owner-bound resync/hint batches plus the retained request/result cases. The Rust
serializer and TypeScript decoders select v7; v6 is retained as F02 compatibility evidence.
No fixture establishes native provider behavior. Protocol 7 rejects older peers before
domain dispatch; matching desktop/supervisor builds are required. Status revision and
accepted invalidation sequence are independent cursors. Collection hints cover related
details and pending actions; resync markers mean all retained observation interests are dirty.
