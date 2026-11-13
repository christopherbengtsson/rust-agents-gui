# Ticket 03 component research

Researched 2026-11-13 against official shadcn CLI 4.23.0, the project's base-nova
registry, installed Base UI 1.9.0, and repository component policy.

| Component   | Decision and primary source                                                                                                                                                                                                                |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Field       | Add label, description, error and grouping composition. It is a presentation group; explicitly associate help/error IDs with its input. [Field](https://ui.shadcn.com/docs/components/base/field)                                          |
| Input Group | Add an editable path with inline-end Browse button. Put the addon after the input in DOM order. [Input Group](https://ui.shadcn.com/docs/components/base/input-group)                                                                      |
| Spinner     | Add pending feedback with meaningful visible text and one announcement. [Spinner](https://ui.shadcn.com/docs/components/base/spinner)                                                                                                      |
| Collapsible | Add independent repository detail disclosure, initially open. Selection remains outside panel lifetime. [Collapsible](https://ui.shadcn.com/docs/components/base/collapsible), [Base UI](https://base-ui.com/react/components/collapsible) |
| Sidebar     | Reuse installed menu rows, actions, skeletons and scrolling. Active styling does not supply selected-navigation semantics. [Sidebar](https://ui.shadcn.com/docs/components/base/sidebar)                                                   |
| Pagination  | Reuse native Buttons for cursor actions; upstream pagination uses anchors. [Pagination](https://ui.shadcn.com/docs/components/base/pagination)                                                                                             |

Reuse installed Dialog, Button, Alert, Empty, Badge and Skeleton. Item and Scroll
Area add no required behavior to this navigation; Combobox adds a searchable
collection rather than arbitrary path entry. Keep actionable failures in persistent
FieldError/Alert instead of adding Toast infrastructure.

The file Input example is an HTML file-upload control. Standard file inputs hide
the absolute filesystem path. `webkitdirectory` selects contained files and
provides relative paths. Use Tauri's native directory picker for the repository
location. [Input](https://ui.shadcn.com/docs/components/base/input#file),
[HTML standard](<https://html.spec.whatwg.org/multipage/input.html#file-upload-state-(type=file)>),
[directory entries](https://wicg.github.io/entries-api/#dom-htmlinputelement-webkitdirectory),
[Tauri dialog](https://v2.tauri.app/plugin/dialog/)

`info`, `docs`, `search`, `view`, `add --dry-run`, and per-file `--diff` were used
before installation. Four direct additions generate six files: Field, Label,
Input Group, Textarea, Spinner and Collapsible. Existing Button/Input/Separator
were retained by answering no to each overwrite prompt. No Base UI or preset
upgrade was performed. The registry's `cn` dependency was already installed.

Generated Label forwards `htmlFor` from consumers, and InputGroupAddon only adds
mouse focus assistance to its keyboard-operable input/buttons. Narrow documented
lint exceptions preserve those upstream contracts. [Composition](https://base-ui.com/react/handbook/composition)
