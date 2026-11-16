# Serde wire contracts

Apply when changing serialized DTOs, compatibility behavior or generated contract
inputs. [Types and errors](types-and-errors.md#input-boundaries) owns validated
construction and current authority. [Observation guidance](../../../../docs/agents/observations.md#contract-preparation)
owns this application's generators, schema settings and preparation interfaces.

## Representations and compatibility

Review what Serde actually reads and writes, not just the Rust field types. Choose
names and enum tagging to match the existing protocol. Treat absent, null and empty
values as distinct where callers do; changing between them changes the contract.
Use defaults for intentional optional inputs in the current contract. Require
current persisted fields rather than supplying values for retired application
formats; incompatible changes use fresh state under the MVP policy.

`default` supplies a value during deserialization. `skip_serializing_if` controls
output omission independently. For example, these fields serialize differently:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct ExampleReply {
    // Always present in output; missing input uses zero.
    #[serde(default)]
    count: u32,
    // None is omitted; missing input is None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    next: Option<String>,
}
```

`ExampleReply { count: 0, next: None }` writes `{"count":0}`, not `{}` or
`{"count":0,"next":null}`. This is an illustrative shape, not a product DTO.
Check generated output schemas against serialization and input schemas against
deserialization; a round trip alone can hide a representation mismatch.

## Flattening and custom representations

Flattening changes the containing wire object, including the handling of additional
keys. Serde does not support `deny_unknown_fields` on the outer or inner flattened
struct. Choose unknown-field handling from the actual compatibility contract;
blanket strictness or omission rules from examples do not define that contract.

Custom `with`, `serialize_with`, `deserialize_with` and `try_from` behavior can
make the wire representation or accepted values differ from a field's Rust type.
Check actual serialized examples and invalid inputs when deriving declarations or
schemas. Preserve contextual assertions after structural validation; generated
shape checks cannot decide current owner/target authority.

Keep a tooling-only export feature additive: enabling it adds export capabilities
without changing ordinary wire behavior. Follow the owning preparation interface
rather than introducing generation in test discovery or runtime initialization.

## Sources

Adapted from `serde-default-compat`, `serde-skip-empty`, `serde-flatten`,
`serde-enum-representation`, `serde-deny-unknown-fields`, `serde-custom-with`,
`serde-try-from-validate` and `proj-feature-additive` at the revision pinned in
the skill entry point. Primary semantics:

- [Serde field attributes](https://serde.rs/field-attrs.html).
- [Serde enum representations](https://serde.rs/enum-representations.html).
- [Serde flattening](https://serde.rs/attr-flatten.html).
- [Cargo feature unification](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification).
