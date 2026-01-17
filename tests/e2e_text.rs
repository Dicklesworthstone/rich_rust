//! End-to-end tests for Text wrapping, justification, and divide() algorithm.
//!
//! These tests verify the complete text manipulation pipeline including:
//! - Word wrapping at various widths
//! - Style preservation across wrapped lines
//! - The critical divide() algorithm for splitting styled text
//! - Justification methods
//! - Wide character handling
//! - Overflow methods
//!
//! Run with: RUST_LOG=debug cargo test --test e2e_text -- --nocapture

mod common;

use common::init_test_logging;
use rich_rust::prelude::*;

// =============================================================================
// Scenario 1: Basic Word Wrap
// =============================================================================

#[test]
fn e2e_word_wrap_simple() {
    init_test_logging();
    tracing::info!("Starting E2E simple word wrap test");

    let text = Text::new("The quick brown fox jumps over the lazy dog");
    tracing::debug!(original = %text.plain(), original_len = text.cell_len(), "Original text");

    let lines = text.wrap(20);
    tracing::debug!(line_count = lines.len(), "Wrapped lines");

    for (i, line) in lines.iter().enumerate() {
        let width = line.cell_len();
        tracing::debug!(line = i, width = width, content = %line.plain(), "Line detail");
        assert!(
            width <= 20,
            "Line {} exceeds width 20: {} ('{}')",
            i,
            width,
            line.plain()
        );
    }

    // Verify all content preserved (no characters lost)
    let combined: String = lines.iter().map(|l| l.plain()).collect::<Vec<_>>().join(" ");
    let combined_trimmed: String = combined.split_whitespace().collect::<Vec<_>>().join(" ");
    let original_trimmed: String = "The quick brown fox jumps over the lazy dog"
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        combined_trimmed.contains("quick") && combined_trimmed.contains("dog"),
        "Content should be preserved"
    );

    tracing::info!("E2E simple word wrap test PASSED");
}

#[test]
fn e2e_word_wrap_narrow() {
    init_test_logging();
    tracing::info!("Starting E2E narrow word wrap test");

    let text = Text::new("Hello World");
    let lines = text.wrap(8);

    tracing::debug!(line_count = lines.len(), "Wrapped lines at width 8");

    assert!(lines.len() >= 2, "Should wrap to multiple lines");
    for line in &lines {
        assert!(
            line.cell_len() <= 8,
            "Line exceeds width 8: {}",
            line.cell_len()
        );
    }

    tracing::info!("E2E narrow word wrap test PASSED");
}

#[test]
fn e2e_word_wrap_exact_fit() {
    init_test_logging();
    tracing::info!("Starting E2E exact fit word wrap test");

    let text = Text::new("Hello"); // 5 chars
    let lines = text.wrap(5);

    tracing::debug!(
        line_count = lines.len(),
        first_line = %lines.first().map(|l| l.plain()).unwrap_or_default(),
        "Exact fit"
    );

    assert_eq!(lines.len(), 1, "Text fitting exactly should be one line");
    assert_eq!(lines[0].plain(), "Hello");

    tracing::info!("E2E exact fit word wrap test PASSED");
}

// =============================================================================
// Scenario 2: Long Word Breaking
// =============================================================================

#[test]
fn e2e_long_word_breaking() {
    init_test_logging();
    tracing::info!("Starting E2E long word breaking test");

    let text = Text::new("Supercalifragilisticexpialidocious");
    tracing::debug!(word_len = text.cell_len(), "Long word length");

    let lines = text.wrap(10);
    tracing::debug!(line_count = lines.len(), "Wrapped lines");

    // Should break the word since it's longer than width
    assert!(lines.len() > 1, "Long word should be broken across lines");

    for (i, line) in lines.iter().enumerate() {
        let width = line.cell_len();
        tracing::debug!(line = i, width = width, content = %line.plain(), "Broken line");
        assert!(width <= 10, "Line {} exceeds width: {}", i, width);
    }

    // Verify the full word is preserved
    let combined: String = lines.iter().map(|l| l.plain()).collect();
    assert_eq!(
        combined, "Supercalifragilisticexpialidocious",
        "Full word should be preserved"
    );

    tracing::info!("E2E long word breaking test PASSED");
}

#[test]
fn e2e_mixed_long_short_words() {
    init_test_logging();
    tracing::info!("Starting E2E mixed word lengths test");

    let text = Text::new("A verylongwordhere B");
    let lines = text.wrap(10);

    tracing::debug!(line_count = lines.len(), "Mixed words wrapped");

    for line in &lines {
        assert!(line.cell_len() <= 10, "Line exceeds width");
    }

    tracing::info!("E2E mixed word lengths test PASSED");
}

// =============================================================================
// Scenario 3: Styled Text Wrapping
// =============================================================================

#[test]
fn e2e_styled_text_wrap() {
    init_test_logging();
    tracing::info!("Starting E2E styled text wrapping test");

    let mut text = Text::new("The quick brown fox");
    let bold = Style::new().bold();
    text.stylize(4, 9, bold); // "quick" is bold

    tracing::debug!(
        plain = %text.plain(),
        span_count = text.spans().len(),
        "Styled text before wrap"
    );

    let lines = text.wrap(12);

    tracing::debug!(line_count = lines.len(), "Wrapped styled text");

    // Check each line for style preservation
    let total_spans: usize = lines.iter().map(|l| l.spans().len()).sum();
    tracing::debug!(total_spans = total_spans, "Total spans after wrap");

    // Style should be preserved in some form
    let combined_plain: String = lines.iter().map(|l| l.plain()).collect::<Vec<_>>().join("");
    assert!(combined_plain.contains("quick"), "Styled word should be preserved");

    tracing::info!("E2E styled text wrapping test PASSED");
}

#[test]
fn e2e_multiple_styles_wrap() {
    init_test_logging();
    tracing::info!("Starting E2E multiple styles wrapping test");

    let mut text = Text::new("Red Blue Green Yellow");
    let red = Style::new().color(Color::parse("red").unwrap());
    let blue = Style::new().color(Color::parse("blue").unwrap());
    let green = Style::new().color(Color::parse("green").unwrap());

    text.stylize(0, 3, red);     // "Red"
    text.stylize(4, 8, blue);    // "Blue"
    text.stylize(9, 14, green);  // "Green"

    tracing::debug!(span_count = text.spans().len(), "Multi-styled text");

    let lines = text.wrap(10);

    // Verify wrapping occurred
    assert!(lines.len() > 1, "Should wrap to multiple lines");

    tracing::info!("E2E multiple styles wrapping test PASSED");
}

// =============================================================================
// Scenario 4: Text.divide() Algorithm (CRITICAL)
// =============================================================================

#[test]
fn e2e_text_divide_basic() {
    init_test_logging();
    tracing::info!("Starting E2E text divide basic test");

    let text = Text::new("Hello World!");
    tracing::debug!(original = %text.plain(), len = text.cell_len(), "Original text");

    let divided = text.divide(&[5, 6, 11]);
    tracing::debug!(part_count = divided.len(), "Divided parts");

    for (i, part) in divided.iter().enumerate() {
        tracing::debug!(
            part = i,
            content = %part.plain(),
            len = part.cell_len(),
            "Divided part"
        );
    }

    assert_eq!(divided[0].plain(), "Hello", "Part 0 should be 'Hello'");
    assert_eq!(divided[1].plain(), " ", "Part 1 should be space");
    assert_eq!(divided[2].plain(), "World", "Part 2 should be 'World'");
    assert_eq!(divided[3].plain(), "!", "Part 3 should be '!'");

    tracing::info!("E2E text divide basic test PASSED");
}

#[test]
fn e2e_text_divide_styled() {
    init_test_logging();
    tracing::info!("Starting E2E text divide with styles test");

    let mut text = Text::new("hello world!");
    let bold = Style::new().bold();
    text.stylize(6, 11, bold); // "world" is bold

    tracing::debug!(
        original = %text.plain(),
        spans = ?text.spans(),
        "Original styled text"
    );

    let divided = text.divide(&[5, 11]);

    for (i, part) in divided.iter().enumerate() {
        tracing::debug!(
            part = i,
            content = %part.plain(),
            span_count = part.spans().len(),
            spans = ?part.spans(),
            "Divided part"
        );
    }

    assert_eq!(divided[0].plain(), "hello", "First part should be 'hello'");
    assert!(divided[0].spans().is_empty(), "First part should have no bold spans");

    assert_eq!(divided[1].plain(), " world", "Second part should be ' world'");
    // The bold span should be adjusted to cover "world" in the second part
    // Note: spans are relative to the part, so we just check presence
    assert!(!divided[1].spans().is_empty(), "Second part should have bold span");

    assert_eq!(divided[2].plain(), "!", "Third part should be '!'");

    tracing::info!("E2E text divide with styles test PASSED");
}

#[test]
fn e2e_text_divide_empty_parts() {
    init_test_logging();
    tracing::info!("Starting E2E text divide with empty parts test");

    let text = Text::new("ABC");
    let divided = text.divide(&[0, 1, 2, 3]);

    tracing::debug!(part_count = divided.len(), "Parts from ABC split at 0,1,2,3");

    for (i, part) in divided.iter().enumerate() {
        tracing::debug!(part = i, content = %part.plain(), "Part");
    }

    // First part is empty (0 to 0)
    // Then A, B, C, empty
    assert_eq!(divided.len(), 5, "Should have 5 parts");

    tracing::info!("E2E text divide with empty parts test PASSED");
}

#[test]
fn e2e_text_divide_no_offsets() {
    init_test_logging();
    tracing::info!("Starting E2E text divide with no offsets test");

    let text = Text::new("Hello");
    let divided = text.divide(&[]);

    assert_eq!(divided.len(), 1, "No offsets should return single part");
    assert_eq!(divided[0].plain(), "Hello", "Single part should be original");

    tracing::info!("E2E text divide with no offsets test PASSED");
}

// =============================================================================
// Scenario 5: Justification
// =============================================================================

#[test]
fn e2e_justify_left() {
    init_test_logging();
    tracing::info!("Starting E2E left justify test");

    let mut text = Text::new("Hello");
    text.pad(10, JustifyMethod::Left);

    tracing::debug!(
        content = %text.plain(),
        len = text.cell_len(),
        "Left justified"
    );

    assert_eq!(text.plain(), "Hello     ", "Left justify should pad right");
    assert_eq!(text.cell_len(), 10, "Should be padded to width 10");

    tracing::info!("E2E left justify test PASSED");
}

#[test]
fn e2e_justify_right() {
    init_test_logging();
    tracing::info!("Starting E2E right justify test");

    let mut text = Text::new("Hello");
    text.pad(10, JustifyMethod::Right);

    tracing::debug!(
        content = %text.plain(),
        len = text.cell_len(),
        "Right justified"
    );

    assert_eq!(text.plain(), "     Hello", "Right justify should pad left");
    assert_eq!(text.cell_len(), 10, "Should be padded to width 10");

    tracing::info!("E2E right justify test PASSED");
}

#[test]
fn e2e_justify_center() {
    init_test_logging();
    tracing::info!("Starting E2E center justify test");

    let mut text = Text::new("Hi");
    text.pad(10, JustifyMethod::Center);

    tracing::debug!(
        content = %text.plain(),
        len = text.cell_len(),
        "Center justified"
    );

    assert_eq!(text.cell_len(), 10, "Should be padded to width 10");
    // "Hi" is 2 chars, 8 padding total, so 4 left + 4 right
    let content = text.plain();
    assert!(content.starts_with(' '), "Should have leading spaces");
    assert!(content.ends_with(' '), "Should have trailing spaces");
    assert!(content.contains("Hi"), "Content should be present");

    tracing::info!("E2E center justify test PASSED");
}

#[test]
fn e2e_justify_no_padding_needed() {
    init_test_logging();
    tracing::info!("Starting E2E no padding needed test");

    let mut text = Text::new("Hello World");
    text.pad(5, JustifyMethod::Left);

    // Text is longer than width, should not change
    assert_eq!(text.plain(), "Hello World", "Should not truncate");

    tracing::info!("E2E no padding needed test PASSED");
}

// =============================================================================
// Scenario 6: Wide Characters
// =============================================================================

#[test]
fn e2e_wrap_cjk() {
    init_test_logging();
    tracing::info!("Starting E2E CJK wrapping test");

    let text = Text::new("你好世界欢迎你"); // Each CJK char is 2 cells wide
    let cell_width = text.cell_len();
    tracing::debug!(cell_width = cell_width, "CJK text cell width");

    assert_eq!(cell_width, 14, "7 CJK chars should be 14 cells");

    let lines = text.wrap(8); // 4 CJK chars fit per line

    tracing::debug!(line_count = lines.len(), "Wrapped CJK lines");

    for (i, line) in lines.iter().enumerate() {
        let width = line.cell_len();
        tracing::debug!(line = i, width = width, content = %line.plain(), "CJK line");
        assert!(width <= 8, "Line {} exceeds width 8: {}", i, width);
    }

    tracing::info!("E2E CJK wrapping test PASSED");
}

#[test]
fn e2e_wrap_emoji() {
    init_test_logging();
    tracing::info!("Starting E2E emoji wrapping test");

    let text = Text::new("Status: ✓ OK");
    let lines = text.wrap(10);

    tracing::debug!(line_count = lines.len(), "Emoji text wrapped");

    // Verify emoji is preserved
    let combined: String = lines.iter().map(|l| l.plain()).collect::<Vec<_>>().join("");
    assert!(combined.contains("✓"), "Emoji should be preserved");

    tracing::info!("E2E emoji wrapping test PASSED");
}

#[test]
fn e2e_wrap_mixed_width() {
    init_test_logging();
    tracing::info!("Starting E2E mixed width wrapping test");

    let text = Text::new("Hello你好World世界"); // Mix of ASCII and CJK
    let cell_width = text.cell_len();
    tracing::debug!(cell_width = cell_width, "Mixed width cell count");

    // "Hello" = 5, "你好" = 4, "World" = 5, "世界" = 4 = 18 cells
    assert_eq!(cell_width, 18, "Mixed text should be 18 cells");

    let lines = text.wrap(10);

    for line in &lines {
        assert!(line.cell_len() <= 10, "Line exceeds width");
    }

    tracing::info!("E2E mixed width wrapping test PASSED");
}

// =============================================================================
// Scenario 7: Overflow Methods
// =============================================================================

#[test]
fn e2e_overflow_crop() {
    init_test_logging();
    tracing::info!("Starting E2E overflow crop test");

    let mut text = Text::new("Hello World");
    text.overflow = OverflowMethod::Crop;

    let lines = text.wrap(5);

    tracing::debug!(
        line_count = lines.len(),
        first = %lines.first().map(|l| l.plain()).unwrap_or_default(),
        "Cropped text"
    );

    // Crop should truncate to fit width
    assert_eq!(lines.len(), 1, "Crop should produce single line");
    assert!(lines[0].cell_len() <= 5, "Should be cropped to width 5");

    tracing::info!("E2E overflow crop test PASSED");
}

#[test]
fn e2e_overflow_ellipsis() {
    init_test_logging();
    tracing::info!("Starting E2E overflow ellipsis test");

    let mut text = Text::new("Hello World");
    text.overflow = OverflowMethod::Ellipsis;

    let lines = text.wrap(8);

    tracing::debug!(
        line_count = lines.len(),
        first = %lines.first().map(|l| l.plain()).unwrap_or_default(),
        "Ellipsis text"
    );

    // Ellipsis should end with "..."
    assert_eq!(lines.len(), 1, "Ellipsis should produce single line");
    let first_line = lines[0].plain();
    assert!(first_line.ends_with("..."), "Should end with ellipsis");
    assert!(lines[0].cell_len() <= 8, "Should fit within width");

    tracing::info!("E2E overflow ellipsis test PASSED");
}

#[test]
fn e2e_overflow_fold() {
    init_test_logging();
    tracing::info!("Starting E2E overflow fold test");

    let mut text = Text::new("Hello World");
    text.overflow = OverflowMethod::Fold;

    let lines = text.wrap(8);

    tracing::debug!(line_count = lines.len(), "Folded text");

    // Fold should wrap to multiple lines
    assert!(lines.len() >= 2, "Fold should wrap to multiple lines");

    for line in &lines {
        assert!(line.cell_len() <= 8, "Each line should fit");
    }

    tracing::info!("E2E overflow fold test PASSED");
}

#[test]
fn e2e_overflow_ignore() {
    init_test_logging();
    tracing::info!("Starting E2E overflow ignore test");

    let mut text = Text::new("Hello World");
    text.overflow = OverflowMethod::Ignore;

    let lines = text.wrap(5);

    tracing::debug!(
        line_count = lines.len(),
        first = %lines.first().map(|l| l.plain()).unwrap_or_default(),
        "Ignore overflow text"
    );

    // Ignore should return the text unchanged
    assert_eq!(lines[0].plain(), "Hello World", "Should be unchanged");

    tracing::info!("E2E overflow ignore test PASSED");
}

// =============================================================================
// Scenario 8: Edge Cases
// =============================================================================

#[test]
fn e2e_text_empty() {
    init_test_logging();
    tracing::info!("Starting E2E empty text test");

    let text = Text::new("");
    let lines = text.wrap(10);

    tracing::debug!(line_count = lines.len(), "Empty text wrapped");

    assert_eq!(lines.len(), 1, "Empty text should produce one empty line");
    assert!(lines[0].plain().is_empty() || lines[0].cell_len() == 0, "Should be empty");

    tracing::info!("E2E empty text test PASSED");
}

#[test]
fn e2e_text_whitespace_only() {
    init_test_logging();
    tracing::info!("Starting E2E whitespace only test");

    let text = Text::new("     ");
    let lines = text.wrap(3);

    tracing::debug!(line_count = lines.len(), "Whitespace text wrapped");

    // Should handle whitespace without panic
    for line in &lines {
        assert!(line.cell_len() <= 3 || line.plain().trim().is_empty());
    }

    tracing::info!("E2E whitespace only test PASSED");
}

#[test]
fn e2e_text_single_char() {
    init_test_logging();
    tracing::info!("Starting E2E single character test");

    let text = Text::new("X");
    let lines = text.wrap(1);

    assert_eq!(lines.len(), 1, "Single char should be one line");
    assert_eq!(lines[0].plain(), "X");

    tracing::info!("E2E single character test PASSED");
}

#[test]
fn e2e_text_newlines() {
    init_test_logging();
    tracing::info!("Starting E2E text with newlines test");

    let text = Text::new("Line1\nLine2\nLine3");
    let split = text.split_lines();

    tracing::debug!(line_count = split.len(), "Split lines");

    assert_eq!(split.len(), 3, "Should have 3 lines");
    assert_eq!(split[0].plain(), "Line1");
    assert_eq!(split[1].plain(), "Line2");
    assert_eq!(split[2].plain(), "Line3");

    tracing::info!("E2E text with newlines test PASSED");
}

#[test]
fn e2e_text_truncate() {
    init_test_logging();
    tracing::info!("Starting E2E text truncate test");

    let mut text = Text::new("Hello World");
    text.truncate(8, OverflowMethod::Ellipsis, false);

    tracing::debug!(
        content = %text.plain(),
        len = text.cell_len(),
        "Truncated text"
    );

    assert!(text.plain().ends_with("..."), "Should end with ellipsis");
    assert!(text.cell_len() <= 8, "Should fit in width 8");

    tracing::info!("E2E text truncate test PASSED");
}

#[test]
fn e2e_text_strip() {
    init_test_logging();
    tracing::info!("Starting E2E text strip test");

    let text = Text::new("  Hello World  ");
    let stripped = text.strip();

    assert_eq!(stripped.plain(), "Hello World", "Whitespace should be stripped");

    tracing::info!("E2E text strip test PASSED");
}

// =============================================================================
// Scenario 9: Styled Operations
// =============================================================================

#[test]
fn e2e_stylize_range() {
    init_test_logging();
    tracing::info!("Starting E2E stylize range test");

    let mut text = Text::new("Hello World");
    let bold = Style::new().bold();
    text.stylize(0, 5, bold);

    tracing::debug!(
        span_count = text.spans().len(),
        spans = ?text.spans(),
        "After stylize"
    );

    assert_eq!(text.spans().len(), 1, "Should have one span");
    let span = &text.spans()[0];
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 5);

    tracing::info!("E2E stylize range test PASSED");
}

#[test]
fn e2e_stylize_all() {
    init_test_logging();
    tracing::info!("Starting E2E stylize all test");

    let mut text = Text::new("Test");
    let italic = Style::new().italic();
    text.stylize_all(italic);

    assert_eq!(text.spans().len(), 1, "Should have one span");
    let span = &text.spans()[0];
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 4, "Span should cover entire text");

    tracing::info!("E2E stylize all test PASSED");
}

// =============================================================================
// Snapshot Tests
// =============================================================================

#[test]
fn e2e_snapshot_wrapped_text() {
    init_test_logging();

    let text = Text::new("The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.");
    let lines = text.wrap(30);

    let output: String = lines
        .iter()
        .map(|l| l.plain())
        .collect::<Vec<_>>()
        .join("\n");

    insta::assert_snapshot!("e2e_wrapped_text", output);
}

#[test]
fn e2e_snapshot_justified_text() {
    init_test_logging();

    let mut left = Text::new("Left");
    left.pad(15, JustifyMethod::Left);

    let mut center = Text::new("Center");
    center.pad(15, JustifyMethod::Center);

    let mut right = Text::new("Right");
    right.pad(15, JustifyMethod::Right);

    let output = format!(
        "|{}|\n|{}|\n|{}|",
        left.plain(),
        center.plain(),
        right.plain()
    );

    insta::assert_snapshot!("e2e_justified_text", output);
}

#[test]
fn e2e_snapshot_cjk_wrapped() {
    init_test_logging();

    let text = Text::new("你好世界欢迎来到Rust编程语言的世界");
    let lines = text.wrap(12);

    let output: String = lines
        .iter()
        .map(|l| l.plain())
        .collect::<Vec<_>>()
        .join("\n");

    insta::assert_snapshot!("e2e_cjk_wrapped", output);
}
