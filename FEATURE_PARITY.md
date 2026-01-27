# Feature Parity (Python Rich -> rich_rust)

> **Purpose:** Authoritative, self-contained parity matrix for the Rust port.
> **Scope:** Compare against core Python Rich behavior and document what is implemented, partial, missing, or explicitly out of scope.
> **Status categories:**
> - **Implemented**: feature exists in rich_rust and is covered by tests.
> - **Partial**: feature exists but is incomplete or has known gaps.
> - **Missing (planned)**: not implemented yet, but in scope.
> - **Out of scope**: intentionally not planned for this project.

This document is the source of truth. If code or docs change, update this file, `RICH_SPEC.md`, README, and `tests/conformance/README.md`.

---

## Status Legend

- **Implemented**: shipping in current codebase.
- **Partial**: some functionality present, but parity gaps remain.
- **Missing (planned)**: explicitly intended but not implemented yet.
- **Out of scope**: excluded by design; not planned.

---

## Related Docs

- `README.md` (public-facing feature overview + examples)
- `RICH_SPEC.md` (detailed behavioral spec)
- `tests/conformance/README.md` (fixture schema + update workflow)

---

## Core Systems

| Area | Python Rich Feature | Status | Rust Location / Flags | Evidence / Notes |
|---|---|---|---|---|
| Console | Console print/render pipeline | Implemented | `src/console.rs` | Extensive unit/e2e tests in `tests/` |
| Console | Capture output segments | Implemented | `Console::begin_capture/end_capture` | `src/console.rs` tests |
| Console | Plain text export | Implemented | `Console::export_text*` | `src/console.rs` tests |
| Style | Style attributes, combine | Implemented | `src/style.rs` | Unit tests + regression tests |
| Color | Named/ANSI/8-bit/truecolor | Implemented | `src/color.rs` | Unit tests + property tests |
| Color | Auto downgrade by terminal | Implemented | `src/color.rs`, `src/terminal.rs` | Regression tests |
| Markup | Rich markup parsing | Implemented | `src/markup/mod.rs` | E2E + regression tests |
| Emoji | Emoji code replacement (`:smile:`) | Implemented | `src/emoji.rs`, `src/console.rs` | Unit tests + Python fixture conformance |
| Text | Styled spans, wrapping | Implemented | `src/text.rs` | E2E + property tests |
| Measurement | Min/max width protocol | Implemented | `src/measure.rs` | Unit + property tests |
| Unicode width | CJK/emoji width | Implemented | `src/cells.rs` | Unit + regression tests |
| Terminal detection | Color + width detection | Implemented | `src/terminal.rs` | Regression tests |

---

## Renderables (Structured Output)

| Renderable | Python Rich Feature | Status | Rust Location / Flags | Evidence / Notes |
|---|---|---|---|---|
| Table | Auto-sizing tables | Implemented | `src/renderables/table.rs` | E2E + golden tests |
| Panel | Panels with titles/subtitles | Implemented | `src/renderables/panel.rs` | E2E + golden tests |
| Rule | Horizontal rules | Implemented | `src/renderables/rule.rs` | E2E + golden tests |
| Tree | Hierarchical trees | Implemented | `src/renderables/tree.rs` | E2E + golden tests |
| Progress | Progress bars + spinners | Implemented | `src/renderables/progress.rs` | E2E + golden tests |
| Columns | Multi-column layout | Implemented | `src/renderables/columns.rs` | Golden tests |
| Emoji | Emoji renderable | Implemented | `src/renderables/emoji.rs` | Unit tests |
| Padding | Padding | Implemented | `src/renderables/padding.rs` | Golden tests |
| Align | Alignment | Implemented | `src/renderables/align.rs` | Golden tests |
| Syntax | Syntax highlighting | Implemented | `src/renderables/syntax.rs` (feature `syntax`) | Tests present; parity vs Python Rich not yet fixture-based |
| Markdown | Markdown rendering | Implemented | `src/renderables/markdown.rs` (feature `markdown`) | Tests present; parity vs Python Rich not yet fixture-based |
| JSON | JSON pretty-print | Implemented | `src/renderables/json.rs` (feature `json`) | Tests present; parity vs Python Rich not yet fixture-based |

---

## Output / Export

| Area | Python Rich Feature | Status | Rust Location / Flags | Evidence / Notes |
|---|---|---|---|---|
| ANSI output | Styled ANSI rendering | Implemented | `Style::render_ansi` + `Console::write_segments` | Unit/regression tests |
| HTML export | Console export to HTML | Implemented | `Console::export_html` | Minimal HTML export (pre + inline styles). |
| SVG export | Console export to SVG | Implemented | `Console::export_svg` | SVG uses `<foreignObject>` + HTML body. |

---

## Live / Layout / Logging

| Feature | Python Rich Feature | Status | Rust Location / Flags | Evidence / Notes |
|---|---|---|---|---|
| Live | Dynamic refresh (`Live`) | Implemented | `src/live.rs` | Nested Live supported; stdout/stderr redirection is best-effort (use Live proxy writers). |
| Layout | Layout engine | Implemented | `src/renderables/layout.rs` | Ratio-based row/column splits with named lookups. |
| Logging | Rich logging handler | Implemented | `src/logging.rs` | `RichLogger` implements the `log` crate; optional `RichTracingLayer` via `tracing` feature. |

---

## Conformance Testing Status

| Topic | Status | Notes |
|---|---|---|
| Internal correctness tests | Implemented | Extensive unit/e2e/golden/regression/property tests in `tests/` |
| Python Rich fixture comparison | Implemented | `tests/conformance/python_reference` + fixtures; gated behind `conformance_test` |

---

## Explicit Exclusions (Out of Scope)

These are intentionally not planned for the Rust port:

- Jupyter/IPython integration
- Python `inspect` module features
- Python `traceback` module features
- Theme `.ini` files (code-defined themes only)
- Input widgets (interactive inputs are out of scope for this library)
- Legacy Windows cmd.exe (use VT sequences via modern terminals)

---

## Maintenance Notes

- Keep this file aligned with real code paths and tests, not intended scope.
- If fixtures are updated, record the Python Rich version and generator timestamp in the JSON.
- For any new feature, add at least one test reference or conformance note in the Evidence column.

---

## Update Checklist (Keep Docs in Sync)

1. Update **this file** (`FEATURE_PARITY.md`) status and notes.
2. Update `RICH_SPEC.md` exclusions and phase notes.
3. Update README **Feature Parity** section.
4. If conformance fixtures change, update conformance docs and fixture version notes.
