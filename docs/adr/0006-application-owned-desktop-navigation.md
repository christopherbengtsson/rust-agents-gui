# Application-owned desktop navigation

Use a small typed application navigation module for the accepted desktop MVP,
without a router: it coordinates workbench/Settings transitions, return/focus and
late-result selection eligibility while existing repository/workspace hooks retain
canonical selections. The user chose workspace selection and document tabs over
general navigation history; React Router and TanStack Router remain credible
alternatives, but neither removes the retained-workbench state or supervisor
ownership the MVP already requires.

Keep workspace tab/file/panel/reading state with workspace presentation and
continuing observations independent of screen visibility. The sidebar is one
caller of navigation rules, alongside Settings Back; this decision adds no generic
view registry or selection rewrite. Reconsider a router when meaningful Back/Forward,
copyable internal destinations, a browser client or independently addressable
sections become requirements. Native deep links require separate target validation
with either choice. The [research](../research/settings-navigation.md) records
primary sources and tradeoffs; the [accepted ticket 06 plan](../../.scratch/milestone-01/codex-setup-plan.md)
records implementation and qualification scope.
