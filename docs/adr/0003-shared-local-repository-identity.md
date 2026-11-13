# Shared local repository identity

Register one application repository per canonical shared Git directory, retaining
a separate durable application ID and the first registered checkout location.
Main and linked worktrees share navigation identity, while independent clones
remain distinct even when their remote URLs match. The user chose this over
separate checkout entries and main-checkout-only admission during ticket 03.

Restored registrations retain their identity when unavailable. A changed or
missing location is reported rather than silently attached to another checkout;
repository relocation remains a separate future workflow. Conductor documents
repository/workspace grouping but does not establish its registration deduplication
algorithm; this is our decision. See [research](../research/conductor-repository-identity.md).
