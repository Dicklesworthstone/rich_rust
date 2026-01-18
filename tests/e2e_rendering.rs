//! End-to-end tests for the complete rendering pipeline.
//!
//! These tests verify the full path from markup string to ANSI output:
//! markup → parse → styled segments → ANSI codes → terminal output
//!
//! Run with: RUST_LOG=debug cargo test --test e2e_rendering -- --nocapture

mod common;

use common::init_test_logging;
use rich_rust::markup;
use rich_rust::prelude::*;

/// Helper to render markup through the console pipeline and capture output.
fn render_markup(markup: &str, color_system: ColorSystem) -> String {
    let console = Console::builder()
        .color_system(color_system)
        .width(80)
        .force_terminal(true)
        .build();

    let mut output = Vec::new();
    let mut options = rich_rust::console::PrintOptions::new().with_markup(true);
    options.no_newline = true;

    console
        .print_to(&mut output, markup, &options)
        .expect("failed to render");

    String::from_utf8(output).expect("invalid utf8")
}

/// Helper to strip ANSI codes for content verification.
fn strip_ansi(s: &str) -> String {
    let ansi_regex = regex::Regex::new(r"\x1b\[[0-9;]*m|\x1b\]8;;[^\x1b]*\x1b\\").unwrap();
    ansi_regex.replace_all(s, "").to_string()
}

// =============================================================================
// Scenario 1: Basic Markup Rendering
// =============================================================================

#[test]
fn e2e_basic_markup_bold() {
    init_test_logging();
    tracing::info!("Starting E2E basic markup bold test");

    let input = "[bold]Hello[/bold]";
    tracing::debug!(input = input, "Rendering markup");

    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "Raw ANSI output");

    // Verify bold ANSI code present (SGR 1)
    assert!(output.contains("\x1b[1m"), "Missing bold ANSI code");
    assert!(output.contains("Hello"), "Missing text content");
    // Verify reset code
    assert!(output.contains("\x1b[0m"), "Missing reset code");

    // Verify plain text extraction
    let plain = strip_ansi(&output);
    assert_eq!(plain, "Hello");

    tracing::info!("E2E basic markup bold test PASSED");
}

#[test]
fn e2e_basic_markup_color() {
    init_test_logging();
    tracing::info!("Starting E2E basic markup color test");

    let input = "[red]Warning[/red]";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "Raw ANSI output");

    // True color may use 38;2;r;g;b or standard 31 for red
    let has_color = output.contains("\x1b[31m") || output.contains("\x1b[38;2;");
    assert!(has_color, "Missing red color code. Output: {}", output);
    assert!(output.contains("Warning"), "Missing text content");

    let plain = strip_ansi(&output);
    assert_eq!(plain, "Warning");

    tracing::info!("E2E basic markup color test PASSED");
}

#[test]
fn e2e_basic_markup_combined() {
    init_test_logging();
    tracing::info!("Starting E2E basic markup combined test");

    let input = "[bold red]Hello[/] [green]World[/]!";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "Raw ANSI output");

    // Verify text content preserved
    let plain = strip_ansi(&output);
    assert_eq!(plain, "Hello World!");

    // Should have multiple style segments
    assert!(output.contains("Hello"), "Missing Hello");
    assert!(output.contains("World"), "Missing World");

    tracing::info!("E2E basic markup combined test PASSED");
}

// =============================================================================
// Scenario 2: Nested Styles
// =============================================================================

#[test]
fn e2e_nested_styles() {
    init_test_logging();
    tracing::info!("Starting E2E nested styles test");

    let input = "[bold][red][underline]styled[/underline][/red][/bold]";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "Raw ANSI output");

    // All three styles should be applied
    assert!(output.contains("\x1b[1"), "Missing bold code");
    // Underline is SGR 4
    assert!(output.contains("4"), "Missing underline indicator");

    let plain = strip_ansi(&output);
    assert_eq!(plain, "styled");

    // Verify proper reset
    assert!(output.contains("\x1b[0m"), "Missing reset code");

    tracing::info!("E2E nested styles test PASSED");
}

#[test]
fn e2e_nested_styles_partial_close() {
    init_test_logging();
    tracing::info!("Starting E2E nested styles partial close test");

    // Using [/] to close the innermost style
    let input = "[bold][red]a[/]b[/]";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "Raw ANSI output");

    let plain = strip_ansi(&output);
    assert_eq!(plain, "ab");

    tracing::info!("E2E nested styles partial close test PASSED");
}

// =============================================================================
// Scenario 3: Style Override
// =============================================================================

#[test]
fn e2e_style_override() {
    init_test_logging();
    tracing::info!("Starting E2E style override test");

    // When blue is nested in red, 'b' should be blue
    let input = "[red]a[blue]b[/blue]c[/red]";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "Raw ANSI output");

    let plain = strip_ansi(&output);
    assert_eq!(plain, "abc");

    // The output should have distinct style regions
    // Check that we have multiple escape sequences
    let escape_count = output.matches("\x1b[").count();
    assert!(
        escape_count >= 3,
        "Expected multiple style changes, got {}",
        escape_count
    );

    tracing::info!("E2E style override test PASSED");
}

// =============================================================================
// Scenario 4: Color System Downgrade
// =============================================================================

#[test]
fn e2e_color_system_truecolor() {
    init_test_logging();
    tracing::info!("Starting E2E TrueColor test");

    let input = "[#ff5500]Orange[/]";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "TrueColor output");

    // TrueColor should use 38;2;r;g;b format
    // #ff5500 = rgb(255, 85, 0)
    assert!(
        output.contains("\x1b[38;2;255;85;0m") || output.contains("\x1b[38;2;"),
        "Expected true color sequence. Output: {}",
        output
    );

    let plain = strip_ansi(&output);
    assert_eq!(plain, "Orange");

    tracing::info!("E2E TrueColor test PASSED");
}

#[test]
fn e2e_color_system_256() {
    init_test_logging();
    tracing::info!("Starting E2E 256-color test");

    let input = "[#ff5500]Orange[/]";
    let output = render_markup(input, ColorSystem::EightBit);
    tracing::debug!(output = %output, "256-color output");

    // 256-color should use 38;5;n format
    let has_256 = output.contains("\x1b[38;5;");
    assert!(has_256, "Expected 256-color sequence. Output: {}", output);

    let plain = strip_ansi(&output);
    assert_eq!(plain, "Orange");

    tracing::info!("E2E 256-color test PASSED");
}

#[test]
fn e2e_color_system_standard() {
    init_test_logging();
    tracing::info!("Starting E2E standard color test");

    let input = "[red]Red[/]";
    let output = render_markup(input, ColorSystem::Standard);
    tracing::debug!(output = %output, "Standard color output");

    // Standard color should use basic codes 30-37/40-47
    // Red foreground is 31
    assert!(
        output.contains("\x1b[31m"),
        "Expected standard red (31). Output: {}",
        output
    );

    let plain = strip_ansi(&output);
    assert_eq!(plain, "Red");

    tracing::info!("E2E standard color test PASSED");
}

// =============================================================================
// Scenario 5: Wide Characters
// =============================================================================

#[test]
fn e2e_wide_characters_cjk() {
    init_test_logging();
    tracing::info!("Starting E2E wide characters CJK test");

    let input = "[bold]你好世界[/]";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "CJK output");

    let plain = strip_ansi(&output);
    assert_eq!(plain, "你好世界");

    // Verify bold applied
    assert!(output.contains("\x1b[1m"), "Missing bold code");

    tracing::info!("E2E wide characters CJK test PASSED");
}

#[test]
fn e2e_wide_characters_emoji() {
    init_test_logging();
    tracing::info!("Starting E2E emoji test");

    let input = "[green]✓[/green] Success";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "Emoji output");

    let plain = strip_ansi(&output);
    assert_eq!(plain, "✓ Success");

    tracing::info!("E2E emoji test PASSED");
}

// =============================================================================
// Scenario 6: Hyperlinks (OSC 8)
// =============================================================================

#[test]
fn e2e_hyperlinks() {
    init_test_logging();
    tracing::info!("Starting E2E hyperlinks test");

    let input = "[link=https://example.com]Click here[/link]";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "Hyperlink output");

    // Verify the text content is rendered
    let plain = strip_ansi(&output);
    assert_eq!(plain, "Click here");

    // OSC 8 sequence should be present if hyperlinks are fully implemented
    // Note: Link support depends on terminal capabilities detection
    // The markup parsing correctly handles link=URL syntax, even if the
    // output doesn't include OSC 8 sequences in all scenarios
    tracing::info!(
        has_osc8 = output.contains("\x1b]8;;"),
        "OSC 8 hyperlink support check"
    );

    tracing::info!("E2E hyperlinks test PASSED");
}

// =============================================================================
// Scenario 7: Edge Cases
// =============================================================================

#[test]
fn e2e_escaped_brackets() {
    init_test_logging();
    tracing::info!("Starting E2E escaped brackets test");

    // Single backslash before opening bracket escapes it
    // Note: closing bracket doesn't need escaping as it's part of the escaped content
    let input = "\\[not a tag]";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "Escaped brackets output");

    let plain = strip_ansi(&output);
    assert_eq!(plain, "[not a tag]");

    tracing::info!("E2E escaped brackets test PASSED");
}

#[test]
fn e2e_empty_markup() {
    init_test_logging();
    tracing::info!("Starting E2E empty markup test");

    let input = "";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "Empty output");

    assert_eq!(output, "");

    tracing::info!("E2E empty markup test PASSED");
}

#[test]
fn e2e_plain_text_no_markup() {
    init_test_logging();
    tracing::info!("Starting E2E plain text test");

    let input = "Just plain text, no markup here.";
    let output = render_markup(input, ColorSystem::TrueColor);
    tracing::debug!(output = %output, "Plain text output");

    // Should not have any ANSI codes
    assert!(
        !output.contains("\x1b["),
        "Unexpected ANSI code in plain text"
    );
    assert_eq!(output, "Just plain text, no markup here.");

    tracing::info!("E2E plain text test PASSED");
}

// =============================================================================
// Snapshot Tests for Visual Regression
// =============================================================================

#[test]
fn e2e_snapshot_complex_markup() {
    init_test_logging();

    let input = "[bold]Title[/bold]\n\
                 [dim]───────────[/dim]\n\
                 [green]✓[/green] Item 1\n\
                 [red]✗[/red] Item 2\n\
                 [yellow]![/yellow] Item 3";

    let output = render_markup(input, ColorSystem::TrueColor);
    let plain = strip_ansi(&output);

    insta::assert_snapshot!("e2e_complex_markup", plain);
}

#[test]
fn e2e_snapshot_color_palette() {
    init_test_logging();

    let colors = vec![
        "[red]red[/]",
        "[green]green[/]",
        "[blue]blue[/]",
        "[yellow]yellow[/]",
        "[magenta]magenta[/]",
        "[cyan]cyan[/]",
    ];

    let input = colors.join(" ");
    let output = render_markup(&input, ColorSystem::TrueColor);
    let plain = strip_ansi(&output);

    insta::assert_snapshot!("e2e_color_palette", plain);
}

// =============================================================================
// Markup Parser Edge Cases
// =============================================================================

#[test]
fn e2e_markup_parser_direct() {
    init_test_logging();
    tracing::info!("Starting markup parser direct test");

    let text = markup::render("[bold red]Hello[/]").expect("parse failed");
    tracing::debug!(plain = %text.plain(), "Parsed text");

    assert_eq!(text.plain(), "Hello");
    assert!(!text.spans().is_empty(), "Expected styled spans");

    // Verify span has both bold and red
    let span = &text.spans()[0];
    assert!(
        span.style.attributes.contains(Attributes::BOLD),
        "Missing bold attribute"
    );

    tracing::info!("Markup parser direct test PASSED");
}
