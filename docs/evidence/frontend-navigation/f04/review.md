# F04 independent review

2026-11-14. Fixed point: `b7c9b8907f8f06e7cd43a17a9328c15e0f7ba61b`.
The code-review skill ran separate fresh-context Standards and Spec agents. To honor
review before commit, both inspected the staged diff from this fixed point plus
subsequent working corrections. No ticket implementation was committed before review.
The unrelated `docs/IDEAS.md` edit was excluded. Repository policies and the F04 ticket /
parent plan were supplied explicitly. Reviewers made no edits or verification runs.

## Standards

P2: a rejected retained stop promise prevented all future explicit reconnect attempts
from reaching a new stop/join. A controlled regression failed before correction
([red output](red-stop-recovery.log)). The controller now handles that previous lost
cleanup reply and awaits a fresh idempotent targeted stop before replacement.
The regression passed; the reviewer confirmed resolution.

Low, heuristic: repeated workspace-detail/list hint pairs could drift. The concrete
owning-module `OwnerState::workspace_changed` helper now owns those four publication
sites. Reviewer confirmed resolution; no speculative forwarding modules were added.

Follow-up inspection confirmed that failed subscription setup requests cleanup of its
possibly attached native resource, with a separate red/green regression, and that the
mounted mocked-Tauri bridge check introduced no standards concern. Final disposition:
no outstanding findings.

## Spec

No additional findings. The reviewer confirmed the intended F04 boundary: bounded
journal, typed hints, owner binding, exact acknowledgements, accepted cursor, dirty
recording before acknowledgement and visible stalls. Existing reads/polling remain.
The reviewer also inspected the working stop-retry correction. Actual native/Tauri
desktop qualification remains pending; controlled tests establish no desktop result.

Final totals: Standards one corrected P2 and one resolved optional heuristic; Spec
zero additional findings. Neither axis has an unresolved finding.
