# Numeric boundaries

Preserve values when converting identifiers, lengths, offsets, limits, or counters.
Use `From` for supported infallible conversions and `TryFrom` when range or sign
can differ. An `as` cast needs a proven domain bound; narrowing or changing sign
can otherwise silently change which work or bytes a value identifies.

Choose overflow behavior where inputs or accumulated state can exceed the type's
domain. Checked operations expose failure for admission or recovery; saturating
operations suit a deliberately capped measurement; wrapping suits a deliberately
modular representation. Preserve unique action identities and ordered history
cursors by treating exhaustion explicitly rather than aliasing an earlier value.
Ordinary arithmetic remains appropriate when established bounds prove it safe.

Apply these decisions before allocation or indexing, including combined frame
lengths, attachment sizes, resource budgets, paged-history cursors, and diff offsets.
Retain units in types or names where a conversion can confuse bytes, characters,
lines, or time. Debug/release overflow defaults are not an application decision.

## Sources

- [TryFrom](https://doc.rust-lang.org/std/convert/trait.TryFrom.html) and [From's contract](https://doc.rust-lang.org/std/convert/trait.From.html#when-to-implement-from).
- [Checked addition](https://doc.rust-lang.org/std/primitive.u64.html#method.checked_add) and [Cargo overflow checks](https://doc.rust-lang.org/cargo/reference/profiles.html#overflow-checks).

Curated from upstream `num-cast-try-from` and `num-overflow-explicit` at the
revision pinned in the skill entry point. Choose APIs supported by the workspace's
toolchain rather than copying the current documentation's entire API surface.
