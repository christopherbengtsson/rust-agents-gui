# Ticket 03 sidebar reference

The user supplied Conductor's first-workspace screenshot on 2026-11-13. It shows
repository headings with chevrons, workspace rows with secondary context, and
Add repository at the sidebar bottom. The image establishes appearance, not
Conductor's precise click behavior or internal toolkit.
[Source page](https://www.conductor.build/docs/first-workspace)

Our chosen composition uses a selectable repository-name button and separate
Collapsible trigger per repository. The user chose path and availability as
ticket 03's expanded content. Ticket 04 adds actual workspace children to this
grouping. Keep selection and observation owners outside the panels; collapsing
one does not change the main-panel selection. Add repository stays near the
bottom above supervisor status. [Official Sidebar composition](https://ui.shadcn.com/docs/components/base/sidebar),
[Base UI Collapsible](https://base-ui.com/react/components/collapsible)

This replaces the earlier suggestion to collapse the whole Repositories section.
The fixed sidebar and existing compact dark tokens remain. The reference's chat,
Changes, New workspace, and terminal surfaces belong to their separately scoped
capabilities; no additional layout framework is required for registration.
