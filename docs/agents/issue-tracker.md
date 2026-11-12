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
