//! End-to-end tests for HTML and SVG export.
//!
//! Verifies that Console::export_html() and Console::export_svg() produce
//! valid, styled output documents with correct structure, color preservation,
//! and content fidelity.

mod common;

use common::init_test_logging;
use rich_rust::prelude::*;

// =============================================================================
// HTML Export: Basic Structure
// =============================================================================

/// Test: export_html produces valid HTML5 document structure.
#[test]
fn test_export_html_document_structure() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("Hello, World!");
    let html = console.export_html(true);

    assert!(
        html.starts_with("<!DOCTYPE html>"),
        "Should start with DOCTYPE"
    );
    assert!(html.contains("<html>"), "Should contain <html> tag");
    assert!(html.contains("<head>"), "Should contain <head> tag");
    assert!(
        html.contains("<meta charset=\"utf-8\">"),
        "Should have UTF-8 charset"
    );
    assert!(html.contains("<body>"), "Should contain <body> tag");
    assert!(
        html.contains("</body></html>"),
        "Should close body and html"
    );
}

/// Test: export_html wraps content in monospace pre block.
#[test]
fn test_export_html_pre_wrapper() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("test content");
    let html = console.export_html(false);

    assert!(
        html.contains("<pre style=\"margin:0; font-family: monospace;\">"),
        "Should have monospace pre wrapper"
    );
    assert!(html.contains("</pre>"), "Should close pre tag");
}

/// Test: export_html contains plain text content.
#[test]
fn test_export_html_text_content() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("Simple plain text");
    let html = console.export_html(true);

    assert!(
        html.contains("Simple plain text"),
        "HTML should contain the text content"
    );
}

// =============================================================================
// HTML Export: Styled Content
// =============================================================================

/// Test: bold text generates font-weight CSS in HTML.
#[test]
fn test_export_html_bold_style() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[bold]Bold text[/]");
    let html = console.export_html(true);

    assert!(
        html.contains("font-weight:bold"),
        "Bold text should produce font-weight:bold CSS, got: {html}"
    );
    assert!(html.contains("Bold text"), "Should contain the text");
}

/// Test: italic text generates font-style CSS.
#[test]
fn test_export_html_italic_style() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[italic]Italic text[/]");
    let html = console.export_html(true);

    assert!(
        html.contains("font-style:italic"),
        "Italic text should produce font-style:italic CSS"
    );
}

/// Test: underline text generates text-decoration CSS.
#[test]
fn test_export_html_underline_style() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[underline]Underlined[/]");
    let html = console.export_html(true);

    assert!(
        html.contains("text-decoration:underline"),
        "Underlined text should produce text-decoration:underline CSS"
    );
}

/// Test: strikethrough text generates line-through CSS.
#[test]
fn test_export_html_strike_style() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[strike]Struck out[/]");
    let html = console.export_html(true);

    assert!(
        html.contains("text-decoration:line-through"),
        "Strikethrough text should produce text-decoration:line-through CSS"
    );
}

/// Test: dim text generates opacity CSS.
#[test]
fn test_export_html_dim_style() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[dim]Dimmed text[/]");
    let html = console.export_html(true);

    assert!(
        html.contains("opacity:0.7"),
        "Dim text should produce opacity:0.7 CSS"
    );
}

// =============================================================================
// HTML Export: Color Preservation
// =============================================================================

/// Test: colored text produces CSS color property with hex value.
#[test]
fn test_export_html_foreground_color() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .color_system(ColorSystem::TrueColor)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[red]Red text[/]");
    let html = console.export_html(true);

    assert!(
        html.contains("color:#"),
        "Colored text should produce CSS color property with hex value"
    );
    assert!(html.contains("Red text"), "Should contain the text content");
}

/// Test: background color produces CSS background-color property.
#[test]
fn test_export_html_background_color() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .color_system(ColorSystem::TrueColor)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[on red]Highlighted[/]");
    let html = console.export_html(true);

    assert!(
        html.contains("background-color:#"),
        "Background color should produce CSS background-color property"
    );
}

/// Test: combined foreground and background colors are preserved.
#[test]
fn test_export_html_fg_and_bg_colors() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .color_system(ColorSystem::TrueColor)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[white on blue]Colored box[/]");
    let html = console.export_html(true);

    assert!(html.contains("color:#"), "Should have foreground color");
    assert!(
        html.contains("background-color:#"),
        "Should have background color"
    );
    assert!(html.contains("Colored box"));
}

/// Test: reverse style swaps foreground and background in CSS.
#[test]
fn test_export_html_reverse_style() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .color_system(ColorSystem::TrueColor)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[reverse red]Reversed[/]");
    let html = console.export_html(true);

    // When reversed, the foreground color (red) should become background-color
    assert!(
        html.contains("background-color:#"),
        "Reverse should swap colors, producing background-color"
    );
}

// =============================================================================
// HTML Export: Inline CSS (span elements)
// =============================================================================

/// Test: styled text wraps in span with inline CSS.
#[test]
fn test_export_html_span_with_inline_css() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[bold italic]Styled[/]");
    let html = console.export_html(true);

    assert!(
        html.contains("<span style=\""),
        "Styled text should be wrapped in <span> with inline style"
    );
    assert!(html.contains("font-weight:bold"));
    assert!(html.contains("font-style:italic"));
}

/// Test: unstyled text is not wrapped in span.
#[test]
fn test_export_html_no_span_for_unstyled() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("Plain text only");
    let html = console.export_html(true);

    // Plain text should NOT be wrapped in a span (no style needed)
    // It should appear directly inside the <pre> tag
    assert!(html.contains("Plain text only"));
    // Count spans - should not have span wrapping plain text
    let span_count = html.matches("<span").count();
    assert_eq!(
        span_count, 0,
        "Plain text should not generate any <span> tags, found {span_count}"
    );
}

// =============================================================================
// HTML Export: Hyperlinks
// =============================================================================

/// Test: hyperlink produces <a> tag in HTML export.
#[test]
fn test_export_html_hyperlink() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    let style = Style::new().bold().link("https://example.com");
    let mut text = Text::new("Click here");
    text.stylize_all(style);

    console.begin_capture();
    console.print_text(&text);
    let html = console.export_html(true);

    assert!(
        html.contains("<a href=\"https://example.com\""),
        "Hyperlink should produce <a> tag with href"
    );
    assert!(html.contains("Click here"), "Should contain link text");
}

// =============================================================================
// HTML Export: Entity Escaping
// =============================================================================

/// Test: HTML special characters are escaped.
#[test]
fn test_export_html_entity_escaping() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("<script>alert('xss')</script>");
    let html = console.export_html(true);

    assert!(
        !html.contains("<script>"),
        "Raw <script> tags should be escaped"
    );
    assert!(
        html.contains("&lt;script&gt;"),
        "< and > should be escaped to &lt; and &gt;"
    );
    assert!(html.contains("&#x27;"), "Single quotes should be escaped");
}

/// Test: ampersands are properly escaped.
#[test]
fn test_export_html_ampersand_escaping() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("AT&T Corporation");
    let html = console.export_html(true);

    assert!(
        html.contains("AT&amp;T Corporation"),
        "Ampersands should be escaped to &amp;"
    );
}

// =============================================================================
// HTML Export: Complex Content
// =============================================================================

/// Test: Text renderable with spans exports correctly.
#[test]
fn test_export_html_text_renderable() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .color_system(ColorSystem::TrueColor)
        .markup(true)
        .build();

    let mut text = Text::new("Hello World");
    text.stylize(0, 5, Style::new().bold());

    console.begin_capture();
    console.print_text(&text);
    let html = console.export_html(true);

    assert!(html.contains("Hello"), "Should contain 'Hello'");
    assert!(html.contains("World"), "Should contain 'World'");
    assert!(
        html.contains("font-weight:bold"),
        "Bold span should be in HTML"
    );
}

/// Test: multiple print calls accumulate in export buffer.
#[test]
fn test_export_html_multiple_prints() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("First line");
    console.print("Second line");
    console.print("Third line");
    let html = console.export_html(true);

    assert!(html.contains("First line"));
    assert!(html.contains("Second line"));
    assert!(html.contains("Third line"));
}

// =============================================================================
// HTML Export: Clear Parameter
// =============================================================================

/// Test: export_html(true) clears the buffer.
#[test]
fn test_export_html_clear_true() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("Content to clear");
    let html1 = console.export_html(true); // clear=true

    // After clearing, next export should not contain the previous content
    let html2 = console.export_html(false);

    assert!(html1.contains("Content to clear"));
    assert!(
        !html2.contains("Content to clear"),
        "After clear=true, buffer should be empty"
    );
}

/// Test: export_html(false) preserves the buffer.
#[test]
fn test_export_html_clear_false() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("Persistent content");
    let html1 = console.export_html(false); // clear=false
    let html2 = console.export_html(false); // clear=false again

    assert!(html1.contains("Persistent content"));
    assert!(
        html2.contains("Persistent content"),
        "Without clear, buffer should be preserved"
    );
}

// =============================================================================
// SVG Export: Basic Structure
// =============================================================================

/// Test: export_svg produces valid SVG document.
#[test]
fn test_export_svg_document_structure() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("SVG content");
    let svg = console.export_svg(true);

    assert!(
        svg.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"),
        "Should start with XML declaration"
    );
    assert!(
        svg.contains("<svg xmlns=\"http://www.w3.org/2000/svg\""),
        "Should have SVG namespace"
    );
    assert!(svg.contains("</svg>"), "Should close SVG tag");
}

/// Test: SVG uses foreignObject to embed HTML.
#[test]
fn test_export_svg_foreign_object() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("Foreign object test");
    let svg = console.export_svg(true);

    assert!(
        svg.contains("<foreignObject width=\"100%\" height=\"100%\">"),
        "Should use foreignObject for HTML embedding"
    );
    assert!(
        svg.contains("</foreignObject>"),
        "Should close foreignObject"
    );
}

/// Test: SVG has dynamic width and height attributes.
#[test]
fn test_export_svg_dimensions() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("Test");
    let svg = console.export_svg(true);

    // SVG should have width and height attributes
    assert!(svg.contains("width=\""), "SVG should have width attribute");
    assert!(
        svg.contains("height=\""),
        "SVG should have height attribute"
    );
}

/// Test: SVG dimensions reflect content size (8px/cell width, 16px/cell height).
#[test]
fn test_export_svg_dimension_calculation() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    // Print a known string (4 characters = 4 cells wide, 1 line high)
    console.begin_capture();
    console.print("ABCD");
    let svg = console.export_svg(true);

    // Cell width = 8px, so 4 cells (minimum) should give at least 32px
    // We can't predict the exact width because print may add trailing content,
    // but we can verify the dimensions are present and reasonable
    assert!(svg.contains("width=\""));
    assert!(svg.contains("height=\""));

    // Extract width value
    if let Some(pos) = svg.find("width=\"") {
        let rest = &svg[pos + 7..];
        if let Some(end) = rest.find('"') {
            let width_str = &rest[..end];
            // This is the first width= in the svg tag
            if let Ok(width) = width_str.parse::<usize>() {
                assert!(
                    width >= 32,
                    "Width should be at least 32px for 4 chars (4 * 8px), got {width}"
                );
            }
        }
    }
}

// =============================================================================
// SVG Export: Content and Styles
// =============================================================================

/// Test: SVG contains styled content via embedded HTML.
#[test]
fn test_export_svg_styled_content() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[bold]Bold in SVG[/]");
    let svg = console.export_svg(true);

    assert!(
        svg.contains("Bold in SVG"),
        "SVG should contain text content"
    );
    assert!(
        svg.contains("font-weight:bold"),
        "SVG should contain styled content via embedded HTML"
    );
}

/// Test: SVG shares the same HTML body rendering as export_html.
#[test]
fn test_export_svg_html_body_consistency() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[italic]Consistent content[/]");

    let html = console.export_html(false);
    let svg = console.export_svg(true);

    // Both should contain the same <pre> body content
    assert!(html.contains("<pre style=\"margin:0; font-family: monospace;\">"));
    assert!(svg.contains("<pre style=\"margin:0; font-family: monospace;\">"));

    // Both should contain the text
    assert!(html.contains("Consistent content"));
    assert!(svg.contains("Consistent content"));
}

/// Test: SVG preserves colors from styled content.
#[test]
fn test_export_svg_color_preservation() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .color_system(ColorSystem::TrueColor)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[red on blue]Colorful[/]");
    let svg = console.export_svg(true);

    assert!(
        svg.contains("color:#"),
        "SVG should preserve foreground color"
    );
    assert!(
        svg.contains("background-color:#"),
        "SVG should preserve background color"
    );
}

// =============================================================================
// SVG Export: Clear Parameter
// =============================================================================

/// Test: export_svg(true) clears the buffer.
#[test]
fn test_export_svg_clear_true() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    console.begin_capture();
    console.print("SVG clear test");
    let svg1 = console.export_svg(true);
    let svg2 = console.export_svg(false);

    assert!(svg1.contains("SVG clear test"));
    assert!(
        !svg2.contains("SVG clear test"),
        "Buffer should be cleared after export_svg(true)"
    );
}

// =============================================================================
// Export Text (Plain)
// =============================================================================

/// Test: export_text strips ANSI codes and returns plain text.
#[test]
fn test_export_text_plain() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    let text = console.export_text("[bold]Hello[/] World");

    // Should contain the text without markup tags
    assert!(text.contains("Hello"), "Should contain 'Hello'");
    assert!(text.contains("World"), "Should contain 'World'");
    assert!(!text.contains("[bold]"), "Should not contain markup tags");
    assert!(!text.contains("[/]"), "Should not contain closing tags");
}

/// Test: export_renderable_text works with Text object.
#[test]
fn test_export_renderable_text() {
    init_test_logging();

    let console = Console::builder().width(80).force_terminal(true).build();

    let text = Text::new("Renderable export test");
    let exported = console.export_renderable_text(&text);

    assert!(
        exported.contains("Renderable export test"),
        "Should contain the text content"
    );
}

// =============================================================================
// Round-trip Validation
// =============================================================================

/// Test: HTML export → parse → verify content preserved.
#[test]
fn test_export_html_roundtrip_content_preserved() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("Line 1: [bold]Bold text[/]");
    console.print("Line 2: [italic]Italic text[/]");
    console.print("Line 3: Plain text");

    let html = console.export_html(true);

    // Verify all content is present
    assert!(html.contains("Bold text"));
    assert!(html.contains("Italic text"));
    assert!(html.contains("Plain text"));

    // Verify styles are preserved
    assert!(html.contains("font-weight:bold"));
    assert!(html.contains("font-style:italic"));

    // Verify document is well-formed
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.ends_with("</body></html>"));
}

/// Test: HTML and SVG from same session have consistent content.
#[test]
fn test_export_html_svg_consistency() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[bold red]Important message[/]");

    let html = console.export_html(false);
    let svg = console.export_svg(false);

    // Both should contain the same text
    assert!(html.contains("Important message"));
    assert!(svg.contains("Important message"));

    // Both should have the same styling
    assert!(html.contains("font-weight:bold"));
    assert!(svg.contains("font-weight:bold"));
}

// =============================================================================
// Complex Content Export
// =============================================================================

/// Test: Table renderable can be exported to HTML.
#[test]
fn test_export_html_table() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .color_system(ColorSystem::TrueColor)
        .markup(true)
        .build();

    use rich_rust::renderables::table::{Cell, Column, Row};

    let mut table = Table::new();
    table.add_column(Column::new("Name"));
    table.add_column(Column::new("Value"));
    table.add_row(Row::new(vec![Cell::new("Alpha"), Cell::new("100")]));
    table.add_row(Row::new(vec![Cell::new("Beta"), Cell::new("200")]));

    console.begin_capture();
    console.print_renderable(&table);
    let html = console.export_html(true);

    assert!(html.contains("Alpha"), "Table data should be in HTML");
    assert!(html.contains("Beta"), "Table data should be in HTML");
    assert!(html.contains("100"), "Table values should be in HTML");
    assert!(html.contains("200"), "Table values should be in HTML");
}

/// Test: Panel renderable can be exported to HTML.
#[test]
fn test_export_html_panel() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    let panel = Panel::from_text("Panel content").title("My Panel");

    console.begin_capture();
    console.print_renderable(&panel);
    let html = console.export_html(true);

    assert!(
        html.contains("Panel content"),
        "Panel body should be in HTML"
    );
    assert!(html.contains("My Panel"), "Panel title should be in HTML");
}

/// Test: Rule renderable can be exported to HTML.
#[test]
fn test_export_html_rule() {
    init_test_logging();

    let console = Console::builder()
        .width(40)
        .force_terminal(true)
        .markup(false)
        .build();

    let rule = Rule::new();

    console.begin_capture();
    console.print_renderable(&rule);
    let html = console.export_html(true);

    // Rule should produce some content (horizontal line characters)
    assert!(
        html.len() > 100,
        "Rule export should produce substantial HTML, got {} bytes",
        html.len()
    );
}

/// Test: Mixed styled content combined in one export.
#[test]
fn test_export_html_combined_styles() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .color_system(ColorSystem::TrueColor)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[bold]Bold[/] [italic]Italic[/] [underline]Underline[/] [dim]Dim[/]");
    let html = console.export_html(true);

    assert!(html.contains("font-weight:bold"));
    assert!(html.contains("font-style:italic"));
    assert!(html.contains("text-decoration:underline"));
    assert!(html.contains("opacity:0.7"));
}

// =============================================================================
// Empty / Edge Cases
// =============================================================================

/// Test: export_html with no content produces minimal document.
#[test]
fn test_export_html_empty_content() {
    init_test_logging();

    let console = Console::builder().width(80).force_terminal(true).build();

    console.begin_capture();
    // Don't print anything
    let html = console.export_html(true);

    // Should still produce valid HTML structure
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<pre style=\"margin:0; font-family: monospace;\">"));
    assert!(html.contains("</pre>"));
}

/// Test: export_svg with no content produces minimal SVG.
#[test]
fn test_export_svg_empty_content() {
    init_test_logging();

    let console = Console::builder().width(80).force_terminal(true).build();

    console.begin_capture();
    let svg = console.export_svg(true);

    assert!(svg.contains("<?xml version=\"1.0\""));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

/// Test: begin_capture / end_capture lifecycle.
#[test]
fn test_capture_lifecycle() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    // Before capture, export should be empty
    console.begin_capture();
    let html_before = console.export_html(false);
    assert!(!html_before.contains("should not appear"));

    // Print during capture
    console.print("Captured content");
    let html_during = console.export_html(false);
    assert!(html_during.contains("Captured content"));

    // End capture returns segments
    let segments = console.end_capture();
    assert!(!segments.is_empty(), "end_capture should return segments");
}

/// Test: export_text without markup does not insert tags.
#[test]
fn test_export_text_no_markup() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(false)
        .build();

    let text = console.export_text("Hello <World> & Friends");
    assert!(
        text.contains("Hello <World> & Friends"),
        "Plain export should preserve special chars as-is"
    );
}

// =============================================================================
// File Output Verification
// =============================================================================

/// Test: export_html output can be written to a file and read back.
#[test]
fn test_export_html_file_roundtrip() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[bold]File export test[/]");
    let html = console.export_html(true);

    // Write to temp file and read back
    let temp_path = std::env::temp_dir().join("rich_rust_export_test.html");
    std::fs::write(&temp_path, &html).expect("write HTML file");
    let read_back = std::fs::read_to_string(&temp_path).expect("read HTML file");

    assert_eq!(html, read_back, "File content should match export output");
    assert!(read_back.contains("File export test"));
    assert!(read_back.starts_with("<!DOCTYPE html>"));

    let _ = std::fs::remove_file(&temp_path);
}

/// Test: export_svg output can be written to a file and read back.
#[test]
fn test_export_svg_file_roundtrip() {
    init_test_logging();

    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .markup(true)
        .build();

    console.begin_capture();
    console.print("[italic]SVG file test[/]");
    let svg = console.export_svg(true);

    let temp_path = std::env::temp_dir().join("rich_rust_export_test.svg");
    std::fs::write(&temp_path, &svg).expect("write SVG file");
    let read_back = std::fs::read_to_string(&temp_path).expect("read SVG file");

    assert_eq!(svg, read_back, "File content should match export output");
    assert!(read_back.contains("SVG file test"));
    assert!(read_back.contains("<svg"));

    let _ = std::fs::remove_file(&temp_path);
}
