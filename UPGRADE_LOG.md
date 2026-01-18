# Dependency Upgrade Log

**Date:** 2026-01-17  |  **Project:** rich_rust  |  **Language:** Rust

## Summary
- **Updated:** 0  |  **Skipped:** 0  |  **Failed:** 0  |  **Needs attention:** 0

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

