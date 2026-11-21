# Dependency Upgrade Log

## 2026-11-21 — v0.2.4 Security & Dependency Maintenance

**Date:** 2026-11-21  |  **Project:** rich_rust  |  **Language:** Rust

### Summary
- **Direct dependencies updated:** 14 (`pulldown-cmark`, `fancy-regex`, `lru`, `serial_test`, `time`, `backtrace`, `bitflags`, `insta`, `log`, `proptest`, `regex`, `serde_json`, `smallvec`, `unicode-width`)
- **Notable transitive security update:** `rustix` `1.1.4 → 1.1.6` (resolves `RUSTSEC-2026-0486`)
- **Cargo audit:** 0 vulnerabilities, 0 unsound warnings (`cargo outdated --root-deps-only` reports all root dependencies up to date; 2 upstream unmaintained warnings remain under latest `syntect 5.3.0`: `bincode 1.3.3` / `RUSTSEC-2025-0141`, `yaml-rust 0.4.5` / `RUSTSEC-2024-0320`)
- **Verification:** `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features --no-fail-fast` (3,895 passed, 0 failed), `cargo doc --no-deps --all-features`, `cargo check --no-default-features --all-targets`

### Key Upgrades

| Dependency | Old | New | Type | Notes |
|------------|-----|-----|------|-------|
| pulldown-cmark | 0.13.4 | 0.13.6 | patch (security) | Resolves `RUSTSEC-2026-0433` and `RUSTSEC-2026-0414`; manifest minimum bumped to `0.13.6` |
| rustix (transitive) | 1.1.4 | 1.1.6 | patch (security) | Resolves `RUSTSEC-2026-0486` |
| fancy-regex | 0.18.0 | 0.20.2 | minor (0.x) | Look-around highlighters and Python conformance suite verified |
| lru | 0.18.2 | 0.19.1 | minor (0.x) | Removes `allocator-api2`, keeps single `hashbrown 0.17.1` |
| serial_test (dev) | 3.5.0 | 4.0.1 | major | Compatible with existing `#[serial]` usage across env-var tests |
| time | 0.3.54 | 0.3.55 | patch | Verified clean against coherence check (`--all-targets`) |
| backtrace | 0.3.76 | 0.3.77 | patch | Compatible update |
| bitflags | 2.13.1 | 2.13.2 | patch | Compatible update |
| log | 0.4.33 | 0.4.35 | patch | Compatible update |
| regex | 1.13.1 | 1.13.2 | patch | Compatible update |
| serde_json | 1.0.151 | 1.0.154 | patch | Compatible update |
| smallvec | 1.15.2 | 1.16.6 | minor | Compatible update |
| unicode-width | 0.2.2 | 0.2.3 | patch | Compatible update |
| insta (dev) | 1.48.0 | 1.49.0 | minor | Compatible update |
| proptest (dev) | 1.11.0 | 1.12.0 | minor | Compatible update |

---

## 2026-01-18 — Initial Dependency Sweep

**Date:** 2026-01-18  |  **Project:** rich_rust  |  **Language:** Rust

## Summary
- **Updated:** 17  |  **Skipped:** 0  |  **Failed:** 0  |  **Needs attention:** 0

### Needs Attention
- (None)

## Dependencies to Update

| Dependency | Current | Latest | Type |
|------------|---------|--------|------|
| bitflags | 2 | 2.10.0 | minor |
| regex | 1 | 1.12.2 | minor |
| crossterm | 0.28 | 0.29.0 | minor |
| unicode-width | 0.2 | 0.2.2 | patch |
| lru | 0.12 | 0.16.3 | minor |
| num-rational | 0.4 | 0.4.2 | patch |
| once_cell | 1.19 | 1.21.3 | minor |
| syntect | 5 | 5.3.0 | minor |
| pulldown-cmark | 0.12 | 0.13.0 | minor |
| serde_json | 1.0 | 1.0.149 | patch |
| criterion | 0.5 | 0.8.1 | minor |
| insta | 1.40 | 1.46.1 | minor |
| tracing | 0.1 | 0.1.44 | patch |
| tracing-subscriber | 0.3 | 0.3.22 | patch |
| tracing-test | 0.2 | 0.2.5 | patch |
| test-log | 0.2 | 0.2.19 | patch |

## Updates

### crossterm: 0.28 → 0.29
- **Breaking changes researched:** Rustix now default (was libc), FileDesc lifetime, KeyEventState serialization
- **Impact on codebase:** None - only basic terminal ops used
- **Tests:** ✓ All passed

### lru: 0.12 → 0.16
- **Breaking changes researched:** `promote`/`demote` return bool (v0.15), MSRV raised to 1.70 (v0.14)
- **Impact on codebase:** None - no promote/demote usage, Rust version compatible
- **Tests:** ✓ All passed

### pulldown-cmark: 0.12 → 0.13
- **Breaking changes researched:** New Tag variants (Superscript, Subscript, DefinitionList*), WikiLinks feature
- **Impact on codebase:** None - wildcard matches handle new variants, specific Options used
- **Tests:** ✓ All passed

### criterion: 0.5 → 0.8
- **Breaking changes researched:** MSRV 1.80, `real_blackbox` removed, `criterion::black_box` deprecated
- **Impact on codebase:** Deprecation warnings - `black_box` should migrate to `std::hint::black_box()`
- **Tests:** ✓ All passed, benchmarks compile with warnings

### insta: 1.40 → 1.46
- **Breaking changes researched:** Minor updates, new assertion macros
- **Impact on codebase:** None
- **Tests:** ✓ All passed

### Batch Update (minor/patch)
- **bitflags:** 2 → 2.10
- **regex:** 1 → 1.12
- **unicode-width:** 0.2 → 0.2.2
- **num-rational:** 0.4 → 0.4.2
- **once_cell:** 1.19 → 1.21
- **syntect:** 5 → 5.3
- **tracing:** 0.1 → 0.1.44
- **tracing-subscriber:** 0.3 → 0.3.22
- **tracing-test:** 0.2 → 0.2.5
- **test-log:** 0.2 → 0.2.19
- **Tests:** ✓ All 424 lib tests passed

