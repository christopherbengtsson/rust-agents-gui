# 09: Native deep-link follow-up

Status: needs-triage

Blocked by: [F08](08-qualify-frontend-foundation.md)

## Preparation required

This is a deferred feature, not part of F01–F08 acceptance. Read the
[external-link scope](../spec.md#external-deep-link-follow-up) and completed F08
route/reload evidence before specifying implementation. Agree the native scheme,
external URL contract and allowed destinations with the user first.

The eventual ticket must cover cold/warm macOS delivery, window activation,
validation, startup races, missing destinations and translation into existing
internal routes. Opening a link retrieves observations; it never starts/resumes a
conversation or submits a provider action. Qualify native delivery separately from
memory-history units. Reuse the established app-lifetime observation owners.

## Comments

2026-11-14: Deferred by agreement. External contract decisions and acceptance are
not yet ready for implementation; do not infer a scheme from internal route paths.
