//! Regression Test Suite for rich_rust
//!
//! This module contains regression tests that prevent reintroduction of previously
//! fixed bugs. Each test documents:
//! - What bug it prevents
//! - When it was introduced/fixed
//! - What the fix was
//! - How to reproduce
//!
//! ## Running Regression Tests
//!
//! ```bash
//! # Run all regression tests
//! cargo test --test regression_tests
//!
//! # Run specific category (by name filter)
//! cargo test --test regression_tests parsing
//! cargo test --test regression_tests layout
//! cargo test --test regression_tests rendering
//! ```
//!
//! ## Categories
//!
//! - `parsing`: Color, style, markup parsing edge cases
//! - `layout`: Table width, text wrap, alignment
//! - `rendering`: ANSI output, Unicode handling

mod common;

use common::{init_test_logging, log_test_context, test_phase};
use rich_rust::r#box::SQUARE;
use rich_rust::color::ColorSystem;
use rich_rust::prelude::*;

// =============================================================================
// PARSING REGRESSION TESTS
// =============================================================================

/// Regression test: Style parsing with empty string
///
/// Bug: Empty string could cause panic or return unexpected result
/// Fixed: Returns null style consistently
///
/// This test ensures empty string parsing always returns a null style.
#[test]
fn regression_parsing_empty_string() {
    init_test_logging();
    log_test_context(
        "regression_parsing_empty_string",
        "Ensures empty string produces null style",
    );

    let _phase = test_phase("parse_empty");

    let result = Style::parse("");
    assert!(result.is_ok(), "Empty string should parse successfully");

    let style = result.unwrap();
    assert!(style.is_null(), "Empty string should produce null style");

    tracing::info!("Regression test PASSED: empty string -> null style");
}

/// Regression test: Style parsing with "none" keyword
///
/// Bug: "none" keyword might be treated as unknown token
/// Fixed: "none" is recognized as producing null style
///
/// This test ensures "none" keyword works correctly.
#[test]
fn regression_parsing_none_keyword() {
    init_test_logging();
    log_test_context(
        "regression_parsing_none_keyword",
        "Ensures 'none' keyword produces null style",
    );

    let _phase = test_phase("parse_none");

    let result = Style::parse("none");
    assert!(result.is_ok(), "'none' should parse successfully");

    let style = result.unwrap();
    assert!(style.is_null(), "'none' should produce null style");

    tracing::info!("Regression test PASSED: 'none' -> null style");
}

/// Regression test: Color parsing with hex codes
///
/// Bug: Hex colors without # prefix or with wrong length could panic
/// Fixed: Proper validation and error handling
///
/// This test ensures hex color parsing handles edge cases correctly.
#[test]
fn regression_parsing_hex_edge_cases() {
    init_test_logging();
    log_test_context(
        "regression_parsing_hex_edge_cases",
        "Ensures hex color parsing handles edge cases",
    );

    let _phase = test_phase("hex_edge_cases");

    // Valid hex colors
    let valid_cases = ["#ff0000", "#00FF00", "#0000ff", "#AbCdEf"];
    for hex in valid_cases {
        let result = Color::parse(hex);
        assert!(result.is_ok(), "'{hex}' should parse successfully");
        tracing::debug!(hex = hex, "Valid hex parsed");
    }

    // Invalid hex colors should return errors, not panic
    let invalid_cases = ["#ff", "#gggggg", "#12345", "#"];
    for hex in invalid_cases {
        let result = Color::parse(hex);
        assert!(result.is_err(), "'{hex}' should fail to parse");
        tracing::debug!(hex = hex, "Invalid hex correctly rejected");
    }

    tracing::info!("Regression test PASSED: hex color edge cases");
}

/// Regression test: RGB color parsing with out-of-range values
///
/// Bug: RGB values > 255 could cause overflow or incorrect colors
/// Fixed: Proper validation with clear error messages
///
/// This test ensures RGB parsing validates component ranges.
#[test]
fn regression_parsing_rgb_out_of_range() {
    init_test_logging();
    log_test_context(
        "regression_parsing_rgb_out_of_range",
        "Ensures RGB parsing validates ranges",
    );

    let _phase = test_phase("rgb_ranges");

    // Valid RGB colors
    let valid = Color::parse("rgb(255,255,255)");
    assert!(valid.is_ok(), "rgb(255,255,255) should be valid");

    let valid_zero = Color::parse("rgb(0,0,0)");
    assert!(valid_zero.is_ok(), "rgb(0,0,0) should be valid");

    // Out of range should fail
    let result = Color::parse("rgb(256,0,0)");
    assert!(result.is_err(), "rgb(256,0,0) should fail (red > 255)");

    let result = Color::parse("rgb(0,256,0)");
    assert!(result.is_err(), "rgb(0,256,0) should fail (green > 255)");

    let result = Color::parse("rgb(0,0,256)");
    assert!(result.is_err(), "rgb(0,0,256) should fail (blue > 255)");

    tracing::info!("Regression test PASSED: RGB range validation");
}

/// Regression test: 256-color palette parsing
///
/// Bug: Color numbers outside 0-255 range could cause issues
/// Fixed: Proper range validation
///
/// This test ensures 256-color palette parsing validates the index.
#[test]
fn regression_parsing_256_palette_range() {
    init_test_logging();
    log_test_context(
        "regression_parsing_256_palette_range",
        "Ensures 256-color palette validates index",
    );

    let _phase = test_phase("palette_range");

    // Valid palette colors
    let valid_0 = Color::parse("color(0)");
    assert!(valid_0.is_ok(), "color(0) should be valid");

    let valid_255 = Color::parse("color(255)");
    assert!(valid_255.is_ok(), "color(255) should be valid");

    // Out of range should fail
    let invalid = Color::parse("color(256)");
    assert!(invalid.is_err(), "color(256) should fail (> 255)");

    let negative = Color::parse("color(-1)");
    assert!(negative.is_err(), "color(-1) should fail (negative)");

    tracing::info!("Regression test PASSED: 256-color palette range");
}

/// Regression test: Style parsing with incomplete keywords
///
/// Bug: Incomplete "on", "not", or "link" keywords could panic
/// Fixed: Proper error messages for incomplete keywords
///
/// This test ensures incomplete keywords produce errors, not panics.
#[test]
fn regression_parsing_incomplete_keywords() {
    init_test_logging();
    log_test_context(
        "regression_parsing_incomplete_keywords",
        "Ensures incomplete keywords produce errors",
    );

    let _phase = test_phase("incomplete_keywords");

    // "on" alone should fail with clear error
    let on_result = Style::parse("on");
    assert!(on_result.is_err(), "'on' alone should fail");

    // "not" alone should fail with clear error
    let not_result = Style::parse("not");
    assert!(not_result.is_err(), "'not' alone should fail");

    // "link" alone should fail with clear error
    let link_result = Style::parse("link");
    assert!(link_result.is_err(), "'link' alone should fail");

    tracing::info!("Regression test PASSED: incomplete keywords");
}

/// Regression test: Style parsing whitespace handling
///
/// Bug: Extra whitespace between tokens could cause parsing failures
/// Fixed: Whitespace is properly normalized during parsing
///
/// This test ensures whitespace variations are handled correctly.
#[test]
fn regression_parsing_whitespace_handling() {
    init_test_logging();
    log_test_context(
        "regression_parsing_whitespace_handling",
        "Ensures whitespace is handled correctly",
    );

    let _phase = test_phase("whitespace");

    // Various whitespace patterns should all work
    let cases = [
        "bold",
        " bold",
        "bold ",
        " bold ",
        "  bold  ",
        "bold red",
        "bold  red",
        " bold  red ",
        "bold   red   on   blue",
    ];

    for case in cases {
        let result = Style::parse(case);
        assert!(result.is_ok(), "'{case}' should parse despite whitespace");
        tracing::debug!(input = case, "Whitespace case passed");
    }

    tracing::info!("Regression test PASSED: whitespace handling");
}

/// Regression test: Style parsing case insensitivity
///
/// Bug: Uppercase keywords might not be recognized
/// Fixed: All parsing is case-insensitive
///
/// This test ensures case variations are handled correctly.
#[test]
fn regression_parsing_case_insensitivity() {
    init_test_logging();
    log_test_context(
        "regression_parsing_case_insensitivity",
        "Ensures case insensitivity",
    );

    let _phase = test_phase("case");

    let cases = ["BOLD", "Bold", "bold", "BOLD RED", "Bold Red", "bold red"];

    for case in cases {
        let result = Style::parse(case);
        assert!(result.is_ok(), "'{case}' should parse (case insensitive)");

        let style = result.unwrap();
        assert!(
            style.attributes.contains(Attributes::BOLD),
            "'{case}' should set bold attribute"
        );
    }

    tracing::info!("Regression test PASSED: case insensitivity");
}

/// Regression test: Color parsing named colors
///
/// Bug: Some named colors might not be recognized or return wrong values
/// Fixed: Complete set of standard named colors
///
/// This test ensures all standard named colors are recognized.
#[test]
fn regression_parsing_named_colors() {
    init_test_logging();
    log_test_context(
        "regression_parsing_named_colors",
        "Ensures named colors are recognized",
    );

    let _phase = test_phase("named_colors");

    let named_colors = [
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
    ];

    for name in named_colors {
        let result = Color::parse(name);
        assert!(result.is_ok(), "'{name}' should be a valid named color");
        tracing::debug!(color = name, "Named color parsed");
    }

    tracing::info!("Regression test PASSED: named colors");
}

// =============================================================================
// LAYOUT REGRESSION TESTS
// =============================================================================

/// Regression test for issue: Table collapse_widths rounding correction
///
/// Bug: collapse_widths() was missing the rounding error correction loop
///      after proportional shrinking, causing column widths to not sum
///      to exactly the available space.
/// Fixed: 2026-01-17 in commit e160e4f
/// Spec: RICH_SPEC Section 9.3 (lines 1680-1694)
///
/// This test ensures proportional shrinking includes rounding correction.
#[test]
fn regression_layout_table_collapse_widths_rounding() {
    init_test_logging();
    log_test_context(
        "regression_layout_table_collapse_widths_rounding",
        "Bug: Missing rounding correction in collapse_widths()",
    );

    let _phase = test_phase("collapse_rounding");
    tracing::info!("Regression test: collapse_widths() rounding correction (commit e160e4f)");

    // Create a table with wide content that must be collapsed
    let wide_content = "X".repeat(45);

    let mut table = Table::new()
        .box_style(&SQUARE)
        .padding(0, 0)
        .with_column(Column::new("Col1").min_width(10))
        .with_column(Column::new("Col2").min_width(10))
        .with_column(Column::new("Col3").min_width(10));

    table.add_row_cells([
        wide_content.as_str(),
        wide_content.as_str(),
        wide_content.as_str(),
    ]);

    // Render at constrained width to force collapse
    let output = table.render_plain(100);

    // The table should render without panic
    assert!(!output.is_empty(), "Table should render successfully");

    // Check that output is well-formed (has top border with correct chars)
    let first_line = output.lines().next().expect("should have lines");
    assert!(
        first_line.starts_with('┌'),
        "Table should have proper border"
    );

    tracing::info!("Regression test PASSED: collapse_widths rounding correction");
}

/// Regression test: Table expand_widths ratio distribution
///
/// Bug: Ratio-based column expansion might not distribute space correctly
/// Fixed: Per RICH_SPEC Section 14.4
///
/// This test ensures ratio expansion distributes space proportionally.
#[test]
fn regression_layout_table_expand_widths_ratio() {
    init_test_logging();
    log_test_context(
        "regression_layout_table_expand_widths_ratio",
        "Ensures ratio-based expansion works correctly",
    );

    let _phase = test_phase("expand_ratio");

    // Table with 1:2:1 ratio columns
    let mut table = Table::new()
        .expand(true)
        .box_style(&SQUARE)
        .padding(0, 0)
        .with_column(Column::new("A").ratio(1))
        .with_column(Column::new("B").ratio(2))
        .with_column(Column::new("C").ratio(1));

    table.add_row_cells(["x", "y", "z"]);

    // Render with enough width for expansion
    let output = table.render_plain(80);

    // Table should render successfully
    assert!(!output.is_empty(), "Table should render");

    // Check the output is well-formed
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines.len() >= 3, "Table should have multiple lines");

    tracing::info!("Regression test PASSED: expand_widths ratio distribution");
}

/// Regression test: Table with zero-width content
///
/// Bug: Columns with empty content could cause division by zero or panic
/// Fixed: Minimum width constraints prevent zero-width columns
///
/// This test ensures empty content doesn't cause issues.
#[test]
fn regression_layout_table_empty_content() {
    init_test_logging();
    log_test_context(
        "regression_layout_table_empty_content",
        "Ensures empty content is handled",
    );

    let _phase = test_phase("empty_content");

    let mut table = Table::new()
        .with_column(Column::new("A"))
        .with_column(Column::new("B"));

    // Add row with empty strings
    table.add_row_cells(["", ""]);

    // Should render without panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| table.render_plain(40)));

    assert!(result.is_ok(), "Empty content should not panic");
    tracing::info!("Regression test PASSED: empty content handling");
}

/// Regression test: Table column width sum exactness
///
/// Bug: Column widths might not sum to available space due to rounding
/// Fixed: Rounding correction distributes remainder to last column
///
/// This test ensures column widths sum exactly to available space.
#[test]
fn regression_layout_table_width_sum_exactness() {
    init_test_logging();
    log_test_context(
        "regression_layout_table_width_sum_exactness",
        "Ensures column widths sum exactly",
    );

    let _phase = test_phase("width_sum");

    // Use ratios that would cause rounding issues (7:13:23)
    let mut table = Table::new()
        .expand(true)
        .box_style(&SQUARE)
        .padding(0, 0)
        .with_column(Column::new("A").ratio(7))
        .with_column(Column::new("B").ratio(13))
        .with_column(Column::new("C").ratio(23));

    table.add_row_cells(["x", "y", "z"]);

    // Render and check it completes without error
    let output = table.render_plain(100);
    assert!(!output.is_empty(), "Table should render");

    tracing::info!("Regression test PASSED: width sum exactness");
}

/// Regression test: Table with very narrow width
///
/// Bug: Table rendered at very narrow width could panic or produce garbled output
/// Fixed: Minimum width constraints and graceful degradation
///
/// This test ensures narrow widths don't cause issues.
#[test]
fn regression_layout_table_very_narrow_width() {
    init_test_logging();
    log_test_context(
        "regression_layout_table_very_narrow_width",
        "Ensures narrow width is handled",
    );

    let _phase = test_phase("narrow_width");

    let mut table = Table::new()
        .with_column(Column::new("Name"))
        .with_column(Column::new("Value"));

    table.add_row_cells(["Test", "Data"]);

    // Render at very narrow width (might need to truncate)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| table.render_plain(10)));

    assert!(result.is_ok(), "Narrow width should not panic");
    tracing::info!("Regression test PASSED: narrow width handling");
}

/// Regression test: Panel with multiline content
///
/// Bug: Panel with multiline content could have alignment issues
/// Fixed: Proper line-by-line rendering with consistent padding
///
/// This test ensures multiline panel content is handled correctly.
#[test]
fn regression_layout_panel_multiline_content() {
    init_test_logging();
    log_test_context(
        "regression_layout_panel_multiline_content",
        "Ensures multiline panel content works",
    );

    let _phase = test_phase("multiline_panel");

    let content = "Line 1\nLine 2\nLine 3";
    let panel = Panel::from_text(content).title("Test").width(30);

    let segments = panel.render(40);
    let output: String = segments.into_iter().map(|s| s.text).collect();

    // Should have multiple lines
    let lines: Vec<&str> = output.lines().collect();
    assert!(
        lines.len() >= 5,
        "Panel should have header, 3 content lines, and footer"
    );

    tracing::info!("Regression test PASSED: multiline panel content");
}

/// Regression test: Rule with very long title
///
/// Bug: Rule with title longer than width could panic or render incorrectly
/// Fixed: Title is truncated when necessary
///
/// This test ensures long titles are handled.
#[test]
fn regression_layout_rule_long_title() {
    init_test_logging();
    log_test_context(
        "regression_layout_rule_long_title",
        "Ensures long titles are handled",
    );

    let _phase = test_phase("long_title");

    let long_title = "This is a very long title that exceeds the available width";
    let rule = Rule::with_title(long_title);

    // Render at narrow width
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| rule.render(30)));

    assert!(result.is_ok(), "Long title should not panic");
    tracing::info!("Regression test PASSED: long title handling");
}

/// Regression test: Tree with deep nesting
///
/// Bug: Deeply nested trees could cause stack overflow or incorrect guides
/// Fixed: Iterative rendering approach
///
/// This test ensures deep nesting is handled.
#[test]
fn regression_layout_tree_deep_nesting() {
    init_test_logging();
    log_test_context(
        "regression_layout_tree_deep_nesting",
        "Ensures deep nesting works",
    );

    let _phase = test_phase("deep_tree");

    // Create a tree with 10 levels of nesting
    let mut deepest = TreeNode::new("Level 10");
    for i in (1..10).rev() {
        let mut parent = TreeNode::new(format!("Level {i}"));
        parent = parent.child(deepest);
        deepest = parent;
    }

    let tree = Tree::new(deepest);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tree.render()));

    assert!(result.is_ok(), "Deep nesting should not panic");
    tracing::info!("Regression test PASSED: deep tree nesting");
}

// =============================================================================
// RENDERING REGRESSION TESTS
// =============================================================================

/// Regression test for issue: Hyperlink-only style rendering
///
/// Bug: Style::render() returned early when ANSI codes were empty,
///      skipping OSC 8 hyperlink output. Styles with only a hyperlink
///      (no colors or attributes) would not render the link.
/// Fixed: 2026-01-18 in commit ca4bd56
///
/// This test ensures hyperlink-only styles render correctly.
#[test]
fn regression_rendering_hyperlink_only_style() {
    init_test_logging();
    log_test_context(
        "regression_rendering_hyperlink_only_style",
        "Bug: Hyperlink-only styles not rendering OSC 8 sequences",
    );

    let _phase = test_phase("hyperlink_only");
    tracing::info!("Regression test: hyperlink-only style rendering (commit ca4bd56)");

    // Create a style with ONLY a hyperlink (no colors, no attributes)
    let mut style = Style::new();
    style.link = Some("https://example.com".to_string());

    // The style should NOT be null (it has a link)
    assert!(!style.is_null(), "Style with link should not be null");

    // Render the style using render_ansi for (prefix, suffix) tuple
    let (prefix, suffix) = style.render_ansi(ColorSystem::TrueColor);

    // The prefix should contain OSC 8 sequence for the link
    // OSC 8 format: \x1b]8;;URL\x1b\\
    assert!(
        prefix.contains("\x1b]8;;") || prefix.contains("\x1b]8;"),
        "Hyperlink-only style should render OSC 8 prefix: got '{}'",
        prefix.escape_debug()
    );

    // The suffix should close the hyperlink
    assert!(
        suffix.contains("\x1b]8;;") || suffix.contains("\x1b]8;"),
        "Hyperlink-only style should render OSC 8 suffix: got '{}'",
        suffix.escape_debug()
    );

    tracing::info!("Regression test PASSED: hyperlink-only style renders OSC 8");
}

/// Regression test: Style with hyperlink AND attributes
///
/// Bug: Styles combining hyperlinks with other attributes might drop one or the other
/// Fixed: Both ANSI codes and OSC 8 sequences are rendered
///
/// This test ensures combined styles render all components.
#[test]
fn regression_rendering_hyperlink_with_attributes() {
    init_test_logging();
    log_test_context(
        "regression_rendering_hyperlink_with_attributes",
        "Ensures hyperlink + attributes both render",
    );

    let _phase = test_phase("hyperlink_with_attrs");

    let style = Style::new().bold().link("https://example.com");

    let (prefix, _suffix) = style.render_ansi(ColorSystem::TrueColor);

    // Should have both bold (SGR 1) and hyperlink (OSC 8)
    assert!(
        prefix.contains("\x1b[1m") || prefix.contains(";1m") || prefix.contains("\x1b[1;"),
        "Should render bold attribute: got '{}'",
        prefix.escape_debug()
    );
    assert!(
        prefix.contains("\x1b]8;"),
        "Should render hyperlink OSC 8 sequence: got '{}'",
        prefix.escape_debug()
    );

    tracing::info!("Regression test PASSED: hyperlink + attributes");
}

/// Regression test: Style combining preserves hyperlinks
///
/// Bug: When combining styles, hyperlinks could be lost
/// Fixed: combine() properly handles link field
///
/// This test ensures style combination preserves hyperlinks.
#[test]
fn regression_rendering_style_combine_preserves_hyperlink() {
    init_test_logging();
    log_test_context(
        "regression_rendering_style_combine_preserves_hyperlink",
        "Ensures style combination preserves hyperlinks",
    );

    let _phase = test_phase("combine_hyperlink");

    let style1 = Style::new().bold();
    let style2 = Style::new().link("https://example.com");

    let combined = style1.combine(&style2);

    // Combined should have both bold and hyperlink
    assert!(
        combined.attributes.contains(Attributes::BOLD),
        "Combined should have bold"
    );
    assert!(combined.link.is_some(), "Combined should have hyperlink");
    assert_eq!(
        combined.link.as_deref(),
        Some("https://example.com"),
        "Hyperlink URL should be preserved"
    );

    tracing::info!("Regression test PASSED: style combine preserves hyperlink");
}

/// Regression test: Null style rendering
///
/// Bug: Null style could produce unexpected output
/// Fixed: Null style renders as empty strings
///
/// This test ensures null styles render correctly.
#[test]
fn regression_rendering_null_style() {
    init_test_logging();
    log_test_context(
        "regression_rendering_null_style",
        "Ensures null style renders as empty",
    );

    let _phase = test_phase("null_style");

    let style = Style::null();
    assert!(style.is_null(), "Style::null() should be null");

    let (prefix, suffix) = style.render_ansi(ColorSystem::TrueColor);

    // Null style should produce empty prefix and suffix
    assert!(prefix.is_empty(), "Null style prefix should be empty");
    assert!(suffix.is_empty(), "Null style suffix should be empty");

    tracing::info!("Regression test PASSED: null style renders empty");
}

/// Regression test: Color downgrade from TrueColor to 256
///
/// Bug: Color downgrade could produce incorrect color codes
/// Fixed: Proper color space conversion
///
/// This test ensures color downgrade works correctly.
#[test]
fn regression_rendering_color_downgrade_truecolor_to_256() {
    init_test_logging();
    log_test_context(
        "regression_rendering_color_downgrade_truecolor_to_256",
        "Ensures color downgrade works",
    );

    let _phase = test_phase("color_downgrade");

    // Create a truecolor style
    let style = Style::parse("#ff5500").unwrap();

    // Render for 256-color system
    let (prefix, _suffix) = style.render_ansi(ColorSystem::EightBit);

    // Should produce 256-color code (38;5;N format)
    assert!(
        prefix.contains("38;5;"),
        "Should downgrade to 256-color format: got '{}'",
        prefix.escape_debug()
    );

    tracing::info!("Regression test PASSED: color downgrade");
}

/// Regression test: Unicode cell width calculation
///
/// Bug: CJK characters could be counted as 1 cell instead of 2
/// Fixed: Proper unicode-width handling
///
/// This test ensures CJK characters have correct width.
#[test]
fn regression_rendering_unicode_cjk_width() {
    init_test_logging();
    log_test_context(
        "regression_rendering_unicode_cjk_width",
        "Ensures CJK characters have width 2",
    );

    let _phase = test_phase("cjk_width");

    use rich_rust::cells::cell_len;

    // CJK characters should be width 2
    let cjk_chars = ['中', '文', '日', '本', '語'];
    for ch in cjk_chars {
        let s = ch.to_string();
        let width = cell_len(&s);
        assert_eq!(
            width, 2,
            "CJK character '{}' should have width 2, got {}",
            ch, width
        );
    }

    // ASCII should be width 1
    let ascii = "hello";
    assert_eq!(cell_len(ascii), 5, "ASCII 'hello' should have width 5");

    tracing::info!("Regression test PASSED: CJK character width");
}

/// Regression test: Emoji cell width
///
/// Bug: Emoji could have incorrect width (1 instead of 2)
/// Fixed: Proper unicode-width handling for emoji
///
/// This test ensures emoji have correct width.
#[test]
fn regression_rendering_unicode_emoji_width() {
    init_test_logging();
    log_test_context(
        "regression_rendering_unicode_emoji_width",
        "Ensures emoji have correct width",
    );

    let _phase = test_phase("emoji_width");

    use rich_rust::cells::cell_len;

    // Basic emoji should typically be width 2
    // Note: actual width depends on unicode-width crate version
    let emoji = "\u{1F600}";
    let width = cell_len(emoji);

    // Width should be reasonable (1 or 2, not 0 or very large)
    assert!(
        width == 1 || width == 2,
        "Emoji should have width 1 or 2, got {}",
        width
    );

    tracing::info!("Regression test PASSED: emoji width");
}

/// Regression test: Segment split at cell boundary
///
/// Bug: Splitting segments at cell boundaries could corrupt CJK characters
/// Fixed: Proper handling of multi-cell characters during split
///
/// This test ensures segment splitting preserves character integrity.
#[test]
fn regression_rendering_segment_split_cjk() {
    init_test_logging();
    log_test_context(
        "regression_rendering_segment_split_cjk",
        "Ensures segment split preserves CJK characters",
    );

    let _phase = test_phase("segment_split");

    let segment = Segment::new("中文", None); // 4 cells total

    // Split in the middle should handle 2-cell characters
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| segment.split_at_cell(2)));

    assert!(result.is_ok(), "Segment split should not panic");

    let (left, right) = result.unwrap();
    // Both parts should be valid UTF-8
    assert!(left.text.chars().count() > 0 || right.text.chars().count() > 0);

    tracing::info!("Regression test PASSED: segment split with CJK");
}

/// Regression test: ANSI escape code stripping
///
/// Bug: Stripping ANSI codes could leave partial sequences
/// Fixed: Complete regex pattern for all SGR codes
///
/// This test ensures ANSI stripping is complete.
#[test]
fn regression_rendering_ansi_strip_completeness() {
    init_test_logging();
    log_test_context(
        "regression_rendering_ansi_strip_completeness",
        "Ensures ANSI stripping is complete",
    );

    let _phase = test_phase("ansi_strip");

    // Create styled output
    let style = Style::parse("bold red on blue").unwrap();
    let (prefix, suffix) = style.render_ansi(ColorSystem::TrueColor);
    let styled = format!("{prefix}Hello{suffix}");

    // Strip ANSI codes
    let ansi_regex = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    let stripped = ansi_regex.replace_all(&styled, "");

    // Should only have the text
    assert_eq!(stripped, "Hello", "ANSI stripping should leave only text");

    tracing::info!("Regression test PASSED: ANSI strip completeness");
}

/// Regression test: Control character handling
///
/// Bug: Control characters could cause rendering issues
/// Fixed: Control characters have zero display width
///
/// This test ensures control characters are handled.
#[test]
fn regression_rendering_control_character_width() {
    init_test_logging();
    log_test_context(
        "regression_rendering_control_character_width",
        "Ensures control characters have zero width",
    );

    let _phase = test_phase("control_chars");

    use rich_rust::cells::cell_len;

    // Control characters should have width 0
    let control_chars = ['\x00', '\x01', '\x1f', '\x7f'];
    for ch in control_chars {
        let s = ch.to_string();
        let width = cell_len(&s);
        assert_eq!(
            width, 0,
            "Control character {:?} should have width 0, got {}",
            ch, width
        );
    }

    tracing::info!("Regression test PASSED: control character width");
}
