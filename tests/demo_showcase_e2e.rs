//! End-to-end tests for demo_showcase binary.
//!
//! These tests spawn the actual binary and verify its behavior as a black box.
//! All tests use the harness module for consistent timeout handling and logging.

mod common;
mod demo_showcase_harness;

use demo_showcase_harness::{DemoRunner, assertions::*};
use std::time::Duration;

// ============================================================================
// Help and basic CLI tests
// ============================================================================

#[test]
fn test_help_flag_shows_usage() {
    common::init_test_logging();

    let result = DemoRunner::new()
        .arg("--help")
        .timeout_secs(5)
        .run()
        .expect("should run");

    assert_success(&result);
    assert_stdout_contains(&result, "USAGE:");
    assert_stdout_contains(&result, "--list-scenes");
    assert_stdout_contains(&result, "--scene");
}

#[test]
fn test_short_help_flag() {
    common::init_test_logging();

    let result = DemoRunner::new()
        .arg("-h")
        .timeout_secs(5)
        .run()
        .expect("should run");

    assert_success(&result);
    assert_stdout_contains(&result, "USAGE:");
}

// ============================================================================
// Scene listing tests
// ============================================================================

#[test]
fn test_list_scenes_shows_all_scenes() {
    common::init_test_logging();

    let result = DemoRunner::new()
        .arg("--list-scenes")
        .timeout_secs(10)
        .run()
        .expect("should run");

    assert_success(&result);
    assert_no_timeout(&result);

    // All storyboard scenes should be listed
    assert_stdout_contains(&result, "hero");
    assert_stdout_contains(&result, "dashboard");
    assert_stdout_contains(&result, "deep_dive_markdown");
    assert_stdout_contains(&result, "deep_dive_syntax");
    assert_stdout_contains(&result, "deep_dive_json");
    assert_stdout_contains(&result, "debug_tools");
    assert_stdout_contains(&result, "export");
    assert_stdout_contains(&result, "outro");

    // Should show table formatting
    assert_stdout_contains(&result, "Available Scenes");
}

// ============================================================================
// Single scene execution tests
// ============================================================================

#[test]
fn test_run_single_scene_hero() {
    common::init_test_logging();

    let result = DemoRunner::quick()
        .arg("--scene")
        .arg("hero")
        .no_color()
        .run()
        .expect("should run");

    assert_success(&result);
    assert_no_timeout(&result);

    // Placeholder scene should print message
    assert_stdout_contains(&result, "hero");
}

#[test]
fn test_run_single_scene_dashboard() {
    common::init_test_logging();

    let result = DemoRunner::quick()
        .arg("--scene")
        .arg("dashboard")
        .no_color()
        .run()
        .expect("should run");

    assert_success(&result);
    assert_no_timeout(&result);
}

#[test]
fn test_unknown_scene_fails() {
    common::init_test_logging();

    let result = DemoRunner::new()
        .arg("--scene")
        .arg("nonexistent_scene")
        .timeout_secs(5)
        .run()
        .expect("should run");

    assert_failure(&result);
    assert_stderr_contains(&result, "Unknown scene");
}

// ============================================================================
// Full demo run tests
// ============================================================================

#[test]
fn test_full_demo_run_completes() {
    common::init_test_logging();

    let result = DemoRunner::quick()
        .non_interactive()
        .no_color()
        .run()
        .expect("should run");

    assert_success(&result);
    assert_no_timeout(&result);

    // Should show header
    assert_stdout_contains(&result, "Nebula Deploy");

    // Should mention all scenes ran
    assert_stdout_contains(&result, "scenes completed");
}

#[test]
fn test_full_demo_with_seed() {
    common::init_test_logging();

    let result = DemoRunner::quick()
        .arg("--seed")
        .arg("12345")
        .non_interactive()
        .no_color()
        .run()
        .expect("should run");

    assert_success(&result);
    assert_no_timeout(&result);
}

// ============================================================================
// Timing and performance tests
// ============================================================================

#[test]
fn test_quick_mode_is_fast() {
    common::init_test_logging();

    let result = DemoRunner::quick()
        .non_interactive()
        .no_color()
        .run()
        .expect("should run");

    assert_success(&result);
    assert_no_timeout(&result);

    // Quick mode should complete in under 5 seconds
    assert_elapsed_under(&result, Duration::from_secs(5));
}

#[test]
fn test_list_scenes_is_fast() {
    common::init_test_logging();

    let result = DemoRunner::new()
        .arg("--list-scenes")
        .timeout_secs(10)
        .run()
        .expect("should run");

    assert_success(&result);

    // Listing scenes should be very fast
    assert_elapsed_under(&result, Duration::from_secs(2));
}

// ============================================================================
// Error handling tests
// ============================================================================

#[test]
fn test_unknown_flag_fails() {
    common::init_test_logging();

    let result = DemoRunner::new()
        .arg("--unknown-flag")
        .timeout_secs(5)
        .run()
        .expect("should run");

    assert_failure(&result);
    assert_stderr_contains(&result, "Unknown flag");
}

#[test]
fn test_invalid_seed_fails() {
    common::init_test_logging();

    let result = DemoRunner::new()
        .arg("--seed")
        .arg("not_a_number")
        .timeout_secs(5)
        .run()
        .expect("should run");

    assert_failure(&result);
    assert_stderr_contains(&result, "Invalid --seed");
}

#[test]
fn test_invalid_speed_fails() {
    common::init_test_logging();

    let result = DemoRunner::new()
        .arg("--speed")
        .arg("0")
        .timeout_secs(5)
        .run()
        .expect("should run");

    assert_failure(&result);
    assert_stderr_contains(&result, "> 0");
}

// ============================================================================
// Output format tests
// ============================================================================

#[test]
fn test_no_color_env_disables_ansi() {
    common::init_test_logging();

    let result = DemoRunner::quick()
        .arg("--list-scenes")
        .env("NO_COLOR", "1")
        .run()
        .expect("should run");

    assert_success(&result);

    // Output should not contain ANSI escape codes
    assert!(
        !result.stdout.contains("\x1b["),
        "Output should not contain ANSI codes when NO_COLOR is set"
    );
}

#[test]
fn test_width_override() {
    common::init_test_logging();

    let result = DemoRunner::quick()
        .arg("--width")
        .arg("40")
        .arg("--list-scenes")
        .no_color()
        .run()
        .expect("should run");

    assert_success(&result);

    // With narrow width, output should wrap or be narrower
    // Just verify it runs successfully
}
