# Changelog

All notable changes to rich_rust are documented in this file.

Organized by landed capabilities, not raw diff order.
Each version links to its GitHub Release (where one exists) and includes representative commit links.

---

## [Unreleased]

Work merged to `main` after the v0.2.1 tag.

- Update AGENTS.md with cass (Cross-Agent Session Search) tool reference ([478c1be](https://github.com/Dicklesworthstone/rich_rust/commit/478c1be6bb701d583608057e1f1b882119fbaf56))

---

## [0.2.1] - 2026-02-22

[GitHub Release](https://github.com/Dicklesworthstone/rich_rust/releases/tag/v0.2.1) (Latest) &middot; 9 commits since v0.2.0

Patch release: removes the nightly-only feature gate, upgrades dependencies, updates the license, and fixes syntax highlighting in interactive prompts.

### Compatibility

- Remove nightly-only feature gate so the crate builds on stable Rust edition 2024 ([8785f20](https://github.com/Dicklesworthstone/rich_rust/commit/8785f20fa7794829f534ec017c455be3f4df66fa))

### Fixed

- Enable syntax highlighting in `Prompt` and `Select` prompt rendering ([b155a2d](https://github.com/Dicklesworthstone/rich_rust/commit/b155a2d858a08098a5719cc206cfefd0ca021820))
- Replace `assert!(false, ...)` with `panic!()` in conformance tests ([c109de9](https://github.com/Dicklesworthstone/rich_rust/commit/c109de974faa577e9e41842a2ac78790e79d54bc))

### Dependencies

- Upgrade bitflags, fancy-regex, time, proptest, and others ([9556469](https://github.com/Dicklesworthstone/rich_rust/commit/955646965ee43a060da36954dbc22295d01f9668))

### Chores

- Update license to MIT with OpenAI/Anthropic Rider ([b9e75ef](https://github.com/Dicklesworthstone/rich_rust/commit/b9e75efe648a54207b032fe4b6f91cadb5b4b36a))
- Update README license references ([8af0ed9](https://github.com/Dicklesworthstone/rich_rust/commit/8af0ed93ed8c4fc707cc183243adbd978df3918e))
- Add GitHub social preview image ([e7beb53](https://github.com/Dicklesworthstone/rich_rust/commit/e7beb539cd4bcdad7dc5159a5c46259a1791ec88))
- Remove tracked `__pycache__/*.pyc` files ([2eb80a8](https://github.com/Dicklesworthstone/rich_rust/commit/2eb80a8d7e15e23d5879d128e940eb9fabd96f46))
- Add `.DS_Store` to `.gitignore` ([ecfad41](https://github.com/Dicklesworthstone/rich_rust/commit/ecfad413b47890e105b5212de699a84be50fa01d))

---

## [0.2.0] - 2026-02-15

[GitHub Release](https://github.com/Dicklesworthstone/rich_rust/releases/tag/v0.2.0) &middot; 221 commits since v0.1.1

Major milestone release that substantially closes the gap with Python Rich feature parity. Adds new renderables, a full interactive demo showcase, conformance testing against Python Rich fixtures, and significant performance work.

### New Renderables and Modules

- **Traceback** renderable with syntax-highlighted source context and optional `Traceback::capture` via the `backtrace` feature ([b35a58f](https://github.com/Dicklesworthstone/rich_rust/commit/b35a58f92b04dabfb0fbec14022cd36ba7d73d6f), [6bd5d4b](https://github.com/Dicklesworthstone/rich_rust/commit/6bd5d4ba12a4472b890219051acb7ecbd089ca77), [540595e](https://github.com/Dicklesworthstone/rich_rust/commit/540595ee1e9631a3a0e3ea97e4f84a6794229a1b))
- **Control** renderable and control code helpers for cursor movement, erase, and terminal control (`rich.control` parity) ([88bda62](https://github.com/Dicklesworthstone/rich_rust/commit/88bda62e2a653d01eccee43260fc7e85072ac8f0), [3fa016c](https://github.com/Dicklesworthstone/rich_rust/commit/3fa016c07934bfb1efa522063e3da02be6a88f23))
- **Constrain** renderable (`rich.constrain` parity) ([24126e5](https://github.com/Dicklesworthstone/rich_rust/commit/24126e5c0a8b305979d3607bdf183c517516fa3b))
- **Highlighter** module with full Python Rich highlighter parity (regex, `ReprHighlighter`, etc.) ([e4a9986](https://github.com/Dicklesworthstone/rich_rust/commit/e4a9986075bd325fd7476d22728a2ff497c326ac), [2901aad](https://github.com/Dicklesworthstone/rich_rust/commit/2901aad9e3023df80f70ccd6143c5b023a8490fd))
- **Group** renderable and `Text::join` method for composing multiple renderables ([4a952d3](https://github.com/Dicklesworthstone/rich_rust/commit/4a952d33a95790fe5a53a6b25c3f459f3a76a338))
- **Pretty/Inspect** renderables for `Debug`-based object introspection ([92c1b5e](https://github.com/Dicklesworthstone/rich_rust/commit/92c1b5e2c6734cfbbf2abfbc710518ba2a87d846), [870a893](https://github.com/Dicklesworthstone/rich_rust/commit/870a893a31c81b29221fe9194d4b987759d798ca))
- **Emoji** module ported from Python Rich emoji codes ([6b45c59](https://github.com/Dicklesworthstone/rich_rust/commit/6b45c59fe9a1b0af9afd86b3df0582f7a6519659))
- **Pager** module for graceful long-content paging ([f990963](https://github.com/Dicklesworthstone/rich_rust/commit/f9909637dd05ce3e2372cf588138d730066e908e))
- **File size formatting** and transfer speed display columns for progress bars ([f97606f](https://github.com/Dicklesworthstone/rich_rust/commit/f97606fb7cb25fde5f3e67fa6e3fc9eabe51717f), [a5c9aa8](https://github.com/Dicklesworthstone/rich_rust/commit/a5c9aa8ab2670640a8aa6a3e5c8b122129118088), [79ee7dc](https://github.com/Dicklesworthstone/rich_rust/commit/79ee7dc81e3c0fccac2f94cecf569d97bc7cfda0))

### Interactive and Console

- **Select** prompt for choice selection ([0085b97](https://github.com/Dicklesworthstone/rich_rust/commit/0085b9771c40c0ce116e7e753a25e2b145a154f3))
- Interactive key handling and start wizard for demo_showcase ([da65d3c](https://github.com/Dicklesworthstone/rich_rust/commit/da65d3c2ea649f934ab016540f99e39c8a45bd9d), [d450a82](https://github.com/Dicklesworthstone/rich_rust/commit/d450a825aa93f5b161cdefc9771c0b01704f0870))
- **Theme** system for named styles matching Python Rich ([fe0cb8a](https://github.com/Dicklesworthstone/rich_rust/commit/fe0cb8a5782ddd2b0f9dedef672971aad61dbda8))
- **ANSI decode** parity: `Text::from_ansi` + `AnsiDecoder` ([e14f645](https://github.com/Dicklesworthstone/rich_rust/commit/e14f6457b10f9d3c1f3af51fbfbf1c6afe3599c3))
- `rich.protocol` parity (`rich_cast` + measure) ([770c3ee](https://github.com/Dicklesworthstone/rich_rust/commit/770c3ee72f613ca02f7345fbe7035e3c9092ea26))

### Demo Showcase ("Nebula Deploy")

Complete interactive demo binary (`demo_showcase`) with a scene-based architecture exercising every renderable. 15 scenes including hero, dashboard, table, panel, tree, layout, JSON, markdown, syntax, traceback, emoji/hyperlinks, debug tools, tracing, export, and outro.

- Scene trait, registry, and runner main loop ([bd2ca08](https://github.com/Dicklesworthstone/rich_rust/commit/bd2ca0899d06dfc76cbdaaef3e6251dc2efaabb3))
- CLI scaffold with config, theming, seed, and non-interactive safety ([29327fe](https://github.com/Dicklesworthstone/rich_rust/commit/29327fedbae26eae9b2f44f46abe1907c88abd46))
- Dashboard scene with live pipeline simulation and streaming logs ([9df47c6](https://github.com/Dicklesworthstone/rich_rust/commit/9df47c677c2a6e4dd10f5dfa1a586515e107cfe9), [cec1187](https://github.com/Dicklesworthstone/rich_rust/commit/cec1187e2326688915b65481a8fb7f2bcd7eadb3))
- Snapshot test harness for visual regression of all demo scenes ([8a0367c](https://github.com/Dicklesworthstone/rich_rust/commit/8a0367c56e0dee1f1c9e0f789bd212ef1504e342))
- Non-TTY safety tests and smoke tests ([c252013](https://github.com/Dicklesworthstone/rich_rust/commit/c252013e9374d796c8a87ee5965facbca801012a), [c2e8c88](https://github.com/Dicklesworthstone/rich_rust/commit/c2e8c8848ecc994ab0f9b70d447fd638a761a761))

### Conformance and Parity

- Syntax/Markdown ANSI output aligned with Python Rich fixtures ([4a4ef20](https://github.com/Dicklesworthstone/rich_rust/commit/4a4ef20be3c9f8cfbda492afd93e0090764c3ebd), [7653034](https://github.com/Dicklesworthstone/rich_rust/commit/76530342aea178d4a40f3cc1539391d6ddbdea23))
- Traceback ANSI parity with Python Rich fixtures ([31db427](https://github.com/Dicklesworthstone/rich_rust/commit/31db427a54b05784adb4662d437204d87f265127))
- `rich.control` parity with conformance fixtures ([3fa016c](https://github.com/Dicklesworthstone/rich_rust/commit/3fa016c07934bfb1efa522063e3da02be6a88f23))
- Export parity and Syntax wrap improvements ([c30a3b3](https://github.com/Dicklesworthstone/rich_rust/commit/c30a3b3958cdddecf4376f24eba63ee8f0709e28))
- Live stdio redirect, emoji toggle, traceback improvements ([1a8ca77](https://github.com/Dicklesworthstone/rich_rust/commit/1a8ca779bab6986052568ee2d07ea9741978ee18))

### Bug Fixes

- Prevent wide-char overflow in narrow fixed-width table columns ([69afd89](https://github.com/Dicklesworthstone/rich_rust/commit/69afd89c8c985d260678a17ab183fd4b0c31a4bc))
- Prevent gutter overflow when Columns width is narrower than separator budget ([1419340](https://github.com/Dicklesworthstone/rich_rust/commit/14193406dc18e48ed8923cef68a14172df8568ba))
- Clamp interactive `max_length` zero to one ([45284ef](https://github.com/Dicklesworthstone/rich_rust/commit/45284efb49e3fa3c6cadad123ff87faa4dc3de5c))
- Preserve CRLF content in `LiveWriter` proxy ([f35f502](https://github.com/Dicklesworthstone/rich_rust/commit/f35f50293594d1e5a6bb78f7c07a9fbf78a362eb))
- Correct cursor positioning to match Python Rich's 0-based API ([67cf0d1](https://github.com/Dicklesworthstone/rich_rust/commit/67cf0d1c7eb08a8b43c72531c2ac497389df3932))
- Fix nested list indentation in Markdown rendering ([0d3b315](https://github.com/Dicklesworthstone/rich_rust/commit/0d3b3151d69b28757734fb933ac79260511c7a61))
- Handle `FORCE_COLOR=0` as "unset" per convention ([f4e3e5a](https://github.com/Dicklesworthstone/rich_rust/commit/f4e3e5a37ebe4ceb73512865fe66870296958aac))
- Prevent panic on malformed markup handler syntax ([4bfbaef](https://github.com/Dicklesworthstone/rich_rust/commit/4bfbaefe3db5ffef370a83a349939aa63189a8cd))
- Preserve Tree label spans during newline sanitization ([9a63bc9](https://github.com/Dicklesworthstone/rich_rust/commit/9a63bc9a658e96712b310112182339c0b04d0e7b))
- Properly handle nested structures in Inspect renderable ([870a893](https://github.com/Dicklesworthstone/rich_rust/commit/870a893a31c81b29221fe9194d4b987759d798ca), [1080a6b](https://github.com/Dicklesworthstone/rich_rust/commit/1080a6b6886e0c95769a08e05080557db1f9bb30))
- Preserve source indentation in Traceback and stop filtering "main" as internal frame ([540595e](https://github.com/Dicklesworthstone/rich_rust/commit/540595ee1e9631a3a0e3ea97e4f84a6794229a1b))
- Fix `time::Month` formatting for time 0.3 ([913f1ad](https://github.com/Dicklesworthstone/rich_rust/commit/913f1ad7a03bb706bd766c8aade9d34b4fe221b0))

### Performance

- Use `SmallVec` for hot paths to reduce heap allocations ([6f293d4](https://github.com/Dicklesworthstone/rich_rust/commit/6f293d42c90ceaf2a1a86126c4c66804bb007d29))
- Comprehensive clippy pedantic lint cleanup across codebase ([4c27749](https://github.com/Dicklesworthstone/rich_rust/commit/4c27749dfb17589195a352e988d21c47a6764e0d), [c10af97](https://github.com/Dicklesworthstone/rich_rust/commit/c10af97b803b50c34e545f26c11efaccde692d88))
- Render benchmark updates ([17e1860](https://github.com/Dicklesworthstone/rich_rust/commit/17e18607da59939d78a05edf77408bfeb6bcb17a))

### Testing Infrastructure

- Property-based testing with proptest ([9e52fa2](https://github.com/Dicklesworthstone/rich_rust/commit/9e52fa27506694eefd2f304f5aadcb8737b325ee))
- Flaky test detection and retry infrastructure ([e3114d3](https://github.com/Dicklesworthstone/rich_rust/commit/e3114d3dc20c9e8436467960d7d07003d97238e1))
- Serial test support for env var isolation ([b3f6b07](https://github.com/Dicklesworthstone/rich_rust/commit/b3f6b075d760f51c114481e2d915f04afae002ca))
- E2E test suites for wide terminals, logging, and markdown ([66c5fa9](https://github.com/Dicklesworthstone/rich_rust/commit/66c5fa94c071c79cad0c4c97833801c92d7bfc39), [534b0fb](https://github.com/Dicklesworthstone/rich_rust/commit/534b0fbaa397744e1c697b6e220090193a951d81))
- Coverage tracking and reporting infrastructure ([d0fbe2a](https://github.com/Dicklesworthstone/rich_rust/commit/d0fbe2abbf7267e9f513880f3710a2cf1708098c))
- Test naming standards and lint enforcement ([9514f5f](https://github.com/Dicklesworthstone/rich_rust/commit/9514f5f03c3e45d16b0648f726da2ff597c137ca))
- Platform-specific fixtures and validation utilities ([0a2ccaf](https://github.com/Dicklesworthstone/rich_rust/commit/0a2ccaf0b7d913c9b66b3576a15e21285c88020b))

---

## [0.1.1] - 2026-01-25

[GitHub Release](https://github.com/Dicklesworthstone/rich_rust/releases/tag/v0.1.1) &middot; 33 commits since v0.1.0

Stabilization patch. Fixes CI failures, adds conformance test infrastructure, and implements layout/live display/logging systems.

### Added

- **Layout, Live display, and Logging** systems in a major feature parity push ([399e228](https://github.com/Dicklesworthstone/rich_rust/commit/399e2284cdc1d8093f1f44bfa69e578cbf727c8e))
- Conformance fixtures and layout fixes ([ccad15d](https://github.com/Dicklesworthstone/rich_rust/commit/ccad15d03991e626782887eff9f532775b7979c1))
- Progress tests module added to conformance suite ([1c1b01f](https://github.com/Dicklesworthstone/rich_rust/commit/1c1b01f3c0a80311bd9232d8d3e4962830817464))
- `force_terminal` honored for color detection ([09e1a8c](https://github.com/Dicklesworthstone/rich_rust/commit/09e1a8cc2869d308972003fd383c36bf69a4324b))
- MIT License file added ([4885d89](https://github.com/Dicklesworthstone/rich_rust/commit/4885d8991160b1f82fd0aa0094b1871cb8447f67))

### Fixed

- Fix `log` crate `std` feature and unicode-width compatibility ([5d0c0cc](https://github.com/Dicklesworthstone/rich_rust/commit/5d0c0cc33d3424fef7800ed4e48c6312cfbc1ebc))
- Sanitize newlines in rule titles to prevent layout breakage ([f842e6a](https://github.com/Dicklesworthstone/rich_rust/commit/f842e6a2bbc38a5b8565f4a8004029e2d6e82570))
- Fix markdown list continuation indent ([be7a034](https://github.com/Dicklesworthstone/rich_rust/commit/be7a0344db58acc6ae697187c59df43827b2fd80))
- Make `PrintOptions.soft_wrap` enable text wrapping ([879862b](https://github.com/Dicklesworthstone/rich_rust/commit/879862baf2e9a91b09ebbfb9033348b95f63fafe))
- Resolve multiple CI failures: clippy warnings, test snapshots, branch refs, cargo-audit CVSS, Windows test warnings ([aa98edc](https://github.com/Dicklesworthstone/rich_rust/commit/aa98edcd4570a87d14e7a6d0eef9473c1a729f07), [2fa9e00](https://github.com/Dicklesworthstone/rich_rust/commit/2fa9e002f293ddf2e5dd917b09f0b1371c33a827), [9c15b22](https://github.com/Dicklesworthstone/rich_rust/commit/9c15b228d2dcd87029ed3b3b6de74a18870dc8e1))
- Relax unicode-width version constraint ([f021b9f](https://github.com/Dicklesworthstone/rich_rust/commit/f021b9f6ca61ec9d0bcb4213f701d0dd45de00a8))
- Remove `let_chains` nightly feature flag for broader compatibility ([dd3dcdb](https://github.com/Dicklesworthstone/rich_rust/commit/dd3dcdb80bff2b4f544f88ed3147dd28e8d5e7d7))

### Changed

- Improve text rendering and console rendering ([5dab327](https://github.com/Dicklesworthstone/rich_rust/commit/5dab3271dc61f0d385c2ee05069cf0ee0f12ac42), [d278086](https://github.com/Dicklesworthstone/rich_rust/commit/d278086fd0f99c2a2bb76c2ca8a56a77c2370907))
- Enhance renderables module ([6d912ac](https://github.com/Dicklesworthstone/rich_rust/commit/6d912ac59407db344302e8dc947a3f7b44798ed9))

---

## [0.1.0] - 2026-01-18

[GitHub Release](https://github.com/Dicklesworthstone/rich_rust/releases/tag/v0.1.0) &middot; 142 commits &middot; Initial release

First public release of rich_rust, a Rust port of Python's Rich library for beautiful terminal output. Approximately 11,800 lines of idiomatic, `#![forbid(unsafe_code)]` Rust across 20+ modules.

### Core

- **Console** coordinator with automatic terminal capability detection (4-bit, 8-bit, 24-bit truecolor), configurable output streams, and ANSI SGR generation ([19b5187](https://github.com/Dicklesworthstone/rich_rust/commit/19b518759f81c0a615ea87c16dcde86603f10820), [b05abdb](https://github.com/Dicklesworthstone/rich_rust/commit/b05abdbbe266fe84a12f020e998fde7c82c91e3c))
- **Style** system with bold, italic, underline, strikethrough, dim, reverse, foreground/background colors, hyperlinks (`link_id`), and meta fields ([d2ce02f](https://github.com/Dicklesworthstone/rich_rust/commit/d2ce02f2a040545333248a0061d945980d58beff)), LRU cache for `render_ansi()` ([c67e00a](https://github.com/Dicklesworthstone/rich_rust/commit/c67e00a6554388360a982dc58d8ce8d28dea9c07))
- **Text** with rich markup parsing (`[bold red]text[/]`), styled spans, Unicode cell width handling
- **Segment** as atomic rendering unit using `Cow<'a, str>` for zero-allocation paths
- **Color** system covering standard 16, 256-color palette, and 24-bit RGB/hex ([d3c14c8](https://github.com/Dicklesworthstone/rich_rust/commit/d3c14c80c234a6cd6e973f733b3b408e34ffd639))
- **Terminal** detection for dimensions, color support, Unicode, and legacy Windows

### Renderables

- **Table** with auto-sizing columns, borders, headers/footers, leading support, and rounding correction in `collapse_widths()` ([e160e4f](https://github.com/Dicklesworthstone/rich_rust/commit/e160e4f4080bc806be70daa0a3d3db8dc7988bbf))
- **Panel** with box styles (rounded, square, heavy, double, ASCII), titles, subtitles, padding ([e69f3db](https://github.com/Dicklesworthstone/rich_rust/commit/e69f3dbcc52482f66940b6da34aed64ed07cbc82))
- **Tree** with hierarchical data visualization and guide lines
- **ProgressBar** with customizable progress tracking and spinners
- **Rule** horizontal dividers with optional styled titles
- **Columns** multi-column newspaper-style layout
- **Padding** and **Align** layout helpers

### Feature-gated Modules

- `syntax`: Syntax highlighting via syntect (100+ languages)
- `markdown`: Markdown rendering via pulldown-cmark (CommonMark + GFM)
- `json`: JSON pretty-printing with syntax highlighting

### Infrastructure

- GitHub Actions CI/CD workflows ([46ae771](https://github.com/Dicklesworthstone/rich_rust/commit/46ae771681064f248dadbee27239cab6d27af217))
- Criterion benchmarks for rendering performance ([c67c126](https://github.com/Dicklesworthstone/rich_rust/commit/c67c126d10d1c929a82a4a4f462cf540181f68ad))
- Comprehensive test infrastructure with golden snapshots, E2E tests (Table, Panel, Text wrapping, rendering pipeline), and proptest invariant verification ([5c7fe7d](https://github.com/Dicklesworthstone/rich_rust/commit/5c7fe7d5e718bf77e97b7ed33e42d1ea9756595c), [707740b](https://github.com/Dicklesworthstone/rich_rust/commit/707740b68d6592f9f3e3ccc715422d5dc4e67e41))
- RICH_SPEC.md validation complete for all sections including Color System, Unicode Cell Width, and Padding ([53619cc](https://github.com/Dicklesworthstone/rich_rust/commit/53619cc4e5152362d85429fdbb41dd07c9b31324))
- Thread safety verified: `Console` is `Send` but not `Sync` (intentional) ([04e9087](https://github.com/Dicklesworthstone/rich_rust/commit/04e9087bdc97dbcf6e36ae31a1071bda290e88f8))
- OSC 8 hyperlink support verified ([d0f8523](https://github.com/Dicklesworthstone/rich_rust/commit/d0f85235879b7150bc48af4ab6d3f64ec5ada970))

### Performance

- `Segment` uses `Cow<'a, str>` for zero-allocation rendering
- `Text::render` uses byte-index slicing to avoid intermediate `String` allocations
- `Style::render_ansi` returns `Arc`-cached strings to reduce cloning in the hot path ([c67e00a](https://github.com/Dicklesworthstone/rich_rust/commit/c67e00a6554388360a982dc58d8ce8d28dea9c07))

---

<!-- version comparison links -->
[Unreleased]: https://github.com/Dicklesworthstone/rich_rust/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/Dicklesworthstone/rich_rust/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Dicklesworthstone/rich_rust/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/Dicklesworthstone/rich_rust/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Dicklesworthstone/rich_rust/releases/tag/v0.1.0
