# Immediate workspaces with durable onboarding

The user chose Conductor-style immediate creation: adding a new repository creates
its first workspace, and repository + creates another, with generated names and
managed folders. Keep registration and creation as distinct durable operations,
recording the initial-creation continuation atomically with successful new
registration so a lost reply or reopening cannot invent another workspace.

Milestone 01 defers the composer and base picker. Automatically pin usable local
origin/HEAD, otherwise exactly one usable local main/master; ambiguity or absence
requires an actionable failure. This avoids a creation setup step while retaining
an identified base, explicit recovery and the original checkout; full-MVP
configured-base fetching remains later scope. See the accepted
[research and trade-offs](../research/ticket-04-conductor-workspace-flow.md).
