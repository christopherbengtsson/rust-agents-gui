# Types and errors

## Identity and state

Give identities distinct newtypes when swapping them would target different work:
for example, `ConversationId`, `TurnId`, and `ActionId`. Distinguish application
identity from provider identity. Represent the identifier's real domain; introducing
a wrapper does not justify narrowing its underlying value or supported cardinality.

Use enums with variant-specific data for mutually exclusive execution states.
Match policy decisions on application-owned states exhaustively so a new state
requires an explicit decision. Keep uncertainty representable instead of converting
missing evidence into success or failure. Persisted, runtime-selected states
usually suit enums; use typestate only where a compile-time protocol simplifies
the actual callers.

When moving state out to perform a transition, identify what remains after each
`?` or early return. Preserve the resource or record the explicit failure/unknown
state needed for recovery; replacing a state with an idle sentinel before fallible
work can discard the evidence needed to continue.

## Input boundaries

Parse raw input into validated types with private fields and fallible constructors
or `TryFrom`. Route deserialization through the same validation, for example with
`#[serde(try_from = "Raw")]`; deriving `Deserialize` on the inner field alone can
bypass a constructor's invariants.

Separate structural validity from current authority. A parsed identifier or
approval payload still needs the owning module's current-state check before an
effect. Follow the provider's supported schema for field naming, optional fields,
unknown fields, and enum representation; a generic Serde recommendation does not
define a provider's compatibility contract.

## Error contracts

Return typed errors where callers make different decisions: incompatible versions,
owner conflicts, rejected approvals, persistence failure, or uncertain receipt.
Keep the original source error and useful operation context through wrapping.
Render a presentation message at the client boundary while retaining the core's
structured evidence. An application crate can need typed errors just as a library
does; choose `thiserror` or an erased error wrapper according to caller needs.

Propagate expected environmental and input failures with `Result`. Use `expect`
only when the established invariant makes failure a programming defect, and state
why it holds. Prefer direct access through a binding when it removes a redundant
check-and-unwrap. Assertions and unwraps in controlled test setup have a different
purpose from handling external failures in production.

Sources: `type-newtype-ids`, `type-enum-states`, `pat-exhaustive-enum`,
`api-parse-dont-validate`, `serde-try-from-validate`, `err-custom-type`,
`err-expect-bugs-only`, and `err-source-chain` in the pinned upstream collection.
Serde's [container attributes](https://serde.rs/container-attrs.html) document the
deserialization mechanism.
