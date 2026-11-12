# Issue tracker: Local Markdown

- Each feature lives under `.scratch/<feature-slug>/`.
- Its spec is `spec.md`.
- Implementation tickets are separate files under
  `issues/<NN>-<slug>.md`, numbered from 01 in dependency order.
- Record triage state as `Status: <label>` near the top.
- Record blocking tickets in `Blocked by:`.
- Append discussion under `## Comments`.

Publishing means writing the spec or ticket to this location.
Fetching means reading the referenced file.
Use the labels defined in `triage-labels.md`.

## Completion

`Status` records triage; the five labels include no completed state. Complete a
ticket by checking its verified acceptance items and appending a dated comment
linking evidence, checks, and remaining qualification limits. Leave failed or
unperformed acceptance items unchecked.

For a scoped batch, complete only the requested tickets. Keep the parent spec and
other tickets open until their own acceptance and qualification gates pass.
