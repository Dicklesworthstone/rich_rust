//! Per-scene smoke tests for demo_showcase.
//!
//! Each test runs a single scene with CI-safe flags and verifies:
//! - Exit code 0 (success)
//! - Completes within timeout
//! - Output contains expected scene marker
//!
//! These tests use the DemoRunner harness for consistent execution.

mod demo_showcase_harness;

use demo_showcase_harness::{DemoRunner, assertions};
use std::time::Duration;

/// CI-safe timeout for individual scene tests.
const SCENE_TIMEOUT: Duration = Duration::from_secs(10);

/// Helper to create a runner configured for smoke testing a specific scene.
fn smoke_runner(scene: &str) -> DemoRunner {
    DemoRunner::quick()
        .non_interactive()
        .arg("--scene")
        .arg(scene)
        .arg("--seed")
        .arg("0")
        .arg("--color-system")
        .arg("none")
        .timeout(SCENE_TIMEOUT)
}

#[test]
fn smoke_hero() {
    let result = smoke_runner("hero").run().expect("should run");
    assertions::assert_success(&result);
    // Hero scene should contain the brand title
    assert!(
        result.stdout_contains("N E B U L A") || result.stdout_contains("Hero"),
        "hero scene should produce recognizable output:\n{}",
        result.diagnostic_output()
    );
}

#[test]
fn smoke_dashboard() {
    let result = smoke_runner("dashboard").run().expect("should run");
    assertions::assert_success(&result);
    // Dashboard is a placeholder, should mention the scene name
    assert!(
        result.stdout_contains("dashboard") || result.stdout_contains("Dashboard"),
        "dashboard scene should produce recognizable output:\n{}",
        result.diagnostic_output()
    );
}

#[test]
fn smoke_markdown() {
    let result = smoke_runner("markdown").run().expect("should run");
    assertions::assert_success(&result);
    assert!(
        result.stdout_contains("markdown") || result.stdout_contains("Markdown"),
        "markdown scene should produce recognizable output:\n{}",
        result.diagnostic_output()
    );
}

#[test]
fn smoke_syntax() {
    let result = smoke_runner("syntax").run().expect("should run");
    assertions::assert_success(&result);
    assert!(
        result.stdout_contains("syntax") || result.stdout_contains("Syntax"),
        "syntax scene should produce recognizable output:\n{}",
        result.diagnostic_output()
    );
}

#[test]
fn smoke_json() {
    let result = smoke_runner("json").run().expect("should run");
    assertions::assert_success(&result);
    assert!(
        result.stdout_contains("json") || result.stdout_contains("JSON"),
        "json scene should produce recognizable output:\n{}",
        result.diagnostic_output()
    );
}

#[test]
fn smoke_table() {
    let result = smoke_runner("table").run().expect("should run");
    assertions::assert_success(&result);
    // Table scene should contain table-related output
    assert!(
        result.stdout_contains("Table") || result.stdout_contains("table"),
        "table scene should produce recognizable output:\n{}",
        result.diagnostic_output()
    );
}

#[test]
fn smoke_debug_tools() {
    let result = smoke_runner("debug_tools").run().expect("should run");
    assertions::assert_success(&result);
    // Debug tools scene shows Pretty/Inspect
    assert!(
        result.stdout_contains("Debug")
            || result.stdout_contains("Pretty")
            || result.stdout_contains("Inspect"),
        "debug_tools scene should produce recognizable output:\n{}",
        result.diagnostic_output()
    );
}

#[test]
fn smoke_traceback() {
    let result = smoke_runner("traceback").run().expect("should run");
    assertions::assert_success(&result);
    // Traceback scene shows error tracing
    assert!(
        result.stdout_contains("Traceback")
            || result.stdout_contains("traceback")
            || result.stdout_contains("Error"),
        "traceback scene should produce recognizable output:\n{}",
        result.diagnostic_output()
    );
}

#[test]
fn smoke_tracing() {
    let result = smoke_runner("tracing").run().expect("should run");
    assertions::assert_success(&result);
    // Tracing scene shows either tracing demo or feature-disabled notice
    assert!(
        result.stdout_contains("Tracing")
            || result.stdout_contains("tracing")
            || result.stdout_contains("Observability"),
        "tracing scene should produce recognizable output:\n{}",
        result.diagnostic_output()
    );
}

#[test]
fn smoke_export() {
    let result = smoke_runner("export").run().expect("should run");
    assertions::assert_success(&result);
    assert!(
        result.stdout_contains("export") || result.stdout_contains("Export"),
        "export scene should produce recognizable output:\n{}",
        result.diagnostic_output()
    );
}

#[test]
fn smoke_outro() {
    let result = smoke_runner("outro").run().expect("should run");
    assertions::assert_success(&result);
    assert!(
        result.stdout_contains("outro") || result.stdout_contains("Outro"),
        "outro scene should produce recognizable output:\n{}",
        result.diagnostic_output()
    );
}

/// Test that --list-scenes works and shows all scenes.
#[test]
fn smoke_list_scenes() {
    let result = DemoRunner::new()
        .non_interactive()
        .arg("--list-scenes")
        .timeout(SCENE_TIMEOUT)
        .run()
        .expect("should run");

    assertions::assert_success(&result);
    // Should list all scene names
    // Note: Some scene names may wrap in narrow terminals (e.g., "debug_tools" -> "debug_tool" + "s")
    assertions::assert_stdout_contains(&result, "hero");
    assertions::assert_stdout_contains(&result, "table");
    assertions::assert_stdout_contains(&result, "debug_tool"); // may wrap as "debug_tool" + "s"
    assertions::assert_stdout_contains(&result, "tracing");
    assertions::assert_stdout_contains(&result, "traceback");
}

/// Test that invalid scene name produces an error.
#[test]
fn smoke_invalid_scene() {
    let result = DemoRunner::new()
        .non_interactive()
        .arg("--scene")
        .arg("nonexistent_scene_xyz")
        .timeout(SCENE_TIMEOUT)
        .run()
        .expect("should run");

    // Should fail with non-zero exit
    assert!(
        !result.success(),
        "invalid scene should fail:\n{}",
        result.diagnostic_output()
    );
}

/// Test that --help works.
#[test]
fn smoke_help() {
    let result = DemoRunner::new()
        .arg("--help")
        .timeout(SCENE_TIMEOUT)
        .run()
        .expect("should run");

    assertions::assert_success(&result);
    assertions::assert_stdout_contains(&result, "demo_showcase");
}
