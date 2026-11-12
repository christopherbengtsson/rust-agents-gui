# Unsafe code and diagnostics

## Unsafe boundaries

Use a safe API when it expresses the operation. Otherwise keep the unsafe block
small and explain why the specific call meets its preconditions in a `SAFETY`
comment. For an unsafe function, document caller obligations in `# Safety` as well
as the justification at each unsafe operation.

A safe wrapper must establish every invariant callers cannot be required to
supply. Bounds checks against a caller-provided length do not establish a raw
pointer's allocation, alignment, initialization, or lifetime. Accept a slice when
it carries the needed evidence; otherwise expose and document the unsafe contract.
Check the called API's actual requirements, including whether the chosen integer
or handle conversions preserve its values.

Keep OS ownership, descriptor lifetime, and error handling in the wrapper's
contract alongside memory safety. Choose applicable verification through the
repository policy; a blanket Miri command cannot establish native process or
filesystem behavior.

## Diagnostic context

Attach structured identifiers for the owner, conversation, turn, and action when
they are relevant. Record the operation, outcome, and source chain so a failure
can be followed across the process seam. Keep subscriber installation in the
executable's configuration rather than inside reusable core operations.

Use `#[instrument(skip_all, fields(...))]` or explicit skip lists and safe fields
when arguments can contain prompts, files, provider messages, or credentials.
Treat `Debug`, `Display`, and error formatting as possible disclosure paths;
logging an entire request or error wrapper can bypass field-level redaction.
Keep retained product transcripts under their storage contract, separate from
routine diagnostic logging.

Sources: `unsafe-safety-comment`, `unsafe-minimize-scope`, `obs-structured-fields`,
`obs-library-facade`, and `obs-no-sensitive-data` in the pinned upstream collection.
The raw-pointer wrapper example from `unsafe-minimize-scope` is deliberately
replaced by the complete-invariant guidance above.
