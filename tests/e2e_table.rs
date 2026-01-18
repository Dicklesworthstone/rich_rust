//! End-to-end tests for Table rendering.
//!
//! Tables are the most complex renderable with many interacting features:
//! column sizing, row rendering, borders, alignment, padding, and more.
//!
//! Run with: RUST_LOG=debug cargo test --test e2e_table -- --nocapture

mod common;

use common::init_test_logging;
use rich_rust::prelude::*;

// =============================================================================
// Scenario 1: Simple Table
// =============================================================================

#[test]
fn e2e_table_simple_2x2() {
    init_test_logging();
    tracing::info!("Starting E2E simple 2x2 table test");

    let mut table = Table::new()
        .with_column(Column::new("Name"))
        .with_column(Column::new("Value"));

    table.add_row_cells(["Alice", "100"]);
    table.add_row_cells(["Bob", "200"]);

    tracing::debug!("Table has {} columns, {} rows", 2, 2);

    let output = table.render_plain(50);
    tracing::debug!(output = %output, "Rendered table");

    // Verify structure
    assert!(output.contains("Name"), "Missing header 'Name'");
    assert!(output.contains("Value"), "Missing header 'Value'");
    assert!(output.contains("Alice"), "Missing cell 'Alice'");
    assert!(output.contains("Bob"), "Missing cell 'Bob'");
    assert!(output.contains("100"), "Missing cell '100'");
    assert!(output.contains("200"), "Missing cell '200'");

    // Verify box characters (default is SQUARE)
    assert!(
        output.contains("┌") || output.contains("├"),
        "Missing box corners"
    );
    assert!(output.contains("─"), "Missing horizontal box line");
    assert!(output.contains("│"), "Missing vertical box line");

    tracing::info!("E2E simple 2x2 table test PASSED");
}

#[test]
fn e2e_table_single_column() {
    init_test_logging();
    tracing::info!("Starting E2E single column table test");

    let mut table = Table::new().with_column(Column::new("Item"));

    table.add_row_cells(["First"]);
    table.add_row_cells(["Second"]);
    table.add_row_cells(["Third"]);

    let output = table.render_plain(30);
    tracing::debug!(output = %output, "Single column table");

    assert!(output.contains("Item"), "Missing header");
    assert!(output.contains("First"), "Missing row 1");
    assert!(output.contains("Second"), "Missing row 2");
    assert!(output.contains("Third"), "Missing row 3");

    tracing::info!("E2E single column table test PASSED");
}

// =============================================================================
// Scenario 2: Table with All Features
// =============================================================================

#[test]
fn e2e_table_with_title_and_caption() {
    init_test_logging();
    tracing::info!("Starting E2E table with title and caption test");

    let mut table = Table::new()
        .title("User Statistics")
        .caption("Q4 2024 Report")
        .with_column(Column::new("Metric"))
        .with_column(Column::new("Value"));

    table.add_row_cells(["Active Users", "1,234"]);
    table.add_row_cells(["Sessions", "5,678"]);

    let output = table.render_plain(50);
    tracing::debug!(output = %output, "Table with title/caption");

    assert!(output.contains("User Statistics"), "Missing title");
    assert!(output.contains("Q4 2024 Report"), "Missing caption");
    assert!(output.contains("Active Users"), "Missing data");

    tracing::info!("E2E table with title and caption test PASSED");
}

#[test]
fn e2e_table_with_footer() {
    init_test_logging();
    tracing::info!("Starting E2E table with footer test");

    let mut table = Table::new()
        .show_footer(true)
        .with_column(Column::new("Product").footer("Total"))
        .with_column(Column::new("Sales").footer("$1,500"));

    table.add_row_cells(["Widget A", "$500"]);
    table.add_row_cells(["Widget B", "$1,000"]);

    let output = table.render_plain(40);
    tracing::debug!(output = %output, "Table with footer");

    assert!(output.contains("Product"), "Missing header");
    assert!(output.contains("Total"), "Missing footer left");
    assert!(output.contains("$1,500"), "Missing footer right");

    tracing::info!("E2E table with footer test PASSED");
}

#[test]
fn e2e_table_ascii_box() {
    init_test_logging();
    tracing::info!("Starting E2E ASCII box table test");

    let mut table = Table::new()
        .ascii()
        .with_column(Column::new("A"))
        .with_column(Column::new("B"));

    table.add_row_cells(["1", "2"]);

    let output = table.render_plain(30);
    tracing::debug!(output = %output, "ASCII box table");

    // ASCII uses +, -, |
    assert!(output.contains("+"), "Missing ASCII corner '+'");
    assert!(output.contains("-"), "Missing ASCII horizontal '-'");
    assert!(output.contains("|"), "Missing ASCII vertical '|'");

    // Should NOT contain Unicode box chars
    assert!(!output.contains("┌"), "Should not have Unicode box chars");
    assert!(!output.contains("─"), "Should not have Unicode horizontal");

    tracing::info!("E2E ASCII box table test PASSED");
}

#[test]
fn e2e_table_no_header() {
    init_test_logging();
    tracing::info!("Starting E2E table without header test");

    let mut table = Table::new()
        .show_header(false)
        .with_column(Column::new("Hidden"))
        .with_column(Column::new("Header"));

    table.add_row_cells(["Data", "Only"]);

    let output = table.render_plain(30);
    tracing::debug!(output = %output, "Table without header");

    assert!(!output.contains("Hidden"), "Header should be hidden");
    assert!(output.contains("Data"), "Data should be visible");

    tracing::info!("E2E table without header test PASSED");
}

#[test]
fn e2e_table_with_row_lines() {
    init_test_logging();
    tracing::info!("Starting E2E table with row separators test");

    let mut table = Table::new()
        .show_lines(true)
        .with_column(Column::new("Item"));

    table.add_row_cells(["One"]);
    table.add_row_cells(["Two"]);
    table.add_row_cells(["Three"]);

    let output = table.render_plain(30);
    tracing::debug!(output = %output, "Table with row lines");

    // Count horizontal separators - should have multiple
    let line_count = output.matches("├").count() + output.matches("─").count();
    tracing::debug!(line_count = line_count, "Separator count");

    assert!(line_count > 3, "Should have multiple row separators");

    tracing::info!("E2E table with row separators test PASSED");
}

// =============================================================================
// Scenario 3: Width Constraints
// =============================================================================

#[test]
fn e2e_table_fixed_column_width() {
    init_test_logging();
    tracing::info!("Starting E2E fixed column width test");

    let mut table = Table::new()
        .with_column(Column::new("Name").width(10))
        .with_column(Column::new("Description").width(20));

    table.add_row_cells(["A", "Short"]);
    table.add_row_cells(["B", "A longer description here"]);

    let output = table.render_plain(50);
    tracing::debug!(output = %output, "Fixed width table");

    // Content should be present (possibly truncated)
    assert!(output.contains("Name"), "Missing header");
    assert!(output.contains("Description"), "Missing header");

    tracing::info!("E2E fixed column width test PASSED");
}

#[test]
fn e2e_table_min_max_width() {
    init_test_logging();
    tracing::info!("Starting E2E min/max column width test");

    let mut table = Table::new()
        .with_column(Column::new("ID").min_width(5).max_width(10))
        .with_column(Column::new("Data"));

    table.add_row_cells(["1", "Some data"]);

    let output = table.render_plain(60);
    tracing::debug!(output = %output, "Min/max width table");

    assert!(output.contains("ID"), "Missing ID column");
    assert!(output.contains("Data"), "Missing Data column");

    tracing::info!("E2E min/max column width test PASSED");
}

#[test]
fn e2e_table_expand() {
    init_test_logging();
    tracing::info!("Starting E2E table expand test");

    let mut table = Table::new()
        .expand(true)
        .with_column(Column::new("A"))
        .with_column(Column::new("B"));

    table.add_row_cells(["X", "Y"]);

    let narrow = table.render_plain(30);
    let wide = table.render_plain(60);

    tracing::debug!(
        narrow_len = narrow.lines().next().map(|l| l.len()),
        "Narrow table"
    );
    tracing::debug!(
        wide_len = wide.lines().next().map(|l| l.len()),
        "Wide table"
    );

    // Wide table should be wider than narrow (expanded to fill)
    let narrow_first_line = narrow.lines().next().unwrap_or("").len();
    let wide_first_line = wide.lines().next().unwrap_or("").len();

    assert!(
        wide_first_line >= narrow_first_line,
        "Expanded table should be at least as wide"
    );

    tracing::info!("E2E table expand test PASSED");
}

#[test]
fn e2e_table_collapse_narrow() {
    init_test_logging();
    tracing::info!("Starting E2E table collapse (narrow width) test");

    let mut table = Table::new()
        .with_column(Column::new("Very Long Header"))
        .with_column(Column::new("Another Long Header"));

    table.add_row_cells(["Short", "Values"]);

    // Render at a very narrow width - should collapse/truncate
    let output = table.render_plain(25);
    tracing::debug!(output = %output, "Narrow table");

    // Should still render without panic
    assert!(!output.is_empty(), "Table should render something");

    tracing::info!("E2E table collapse (narrow width) test PASSED");
}

// =============================================================================
// Scenario 4: Wide Characters in Cells
// =============================================================================

#[test]
fn e2e_table_cjk_content() {
    init_test_logging();
    tracing::info!("Starting E2E table with CJK content test");

    let mut table = Table::new()
        .with_column(Column::new("日本語"))
        .with_column(Column::new("English"));

    table.add_row_cells(["東京", "Tokyo"]);
    table.add_row_cells(["大阪", "Osaka"]);
    table.add_row_cells(["京都", "Kyoto"]);

    let output = table.render_plain(40);
    tracing::debug!(output = %output, "CJK table");

    // Verify content preserved
    assert!(output.contains("日本語"), "Missing Japanese header");
    assert!(output.contains("東京"), "Missing Tokyo in Japanese");
    assert!(output.contains("Tokyo"), "Missing Tokyo in English");

    tracing::info!("E2E table with CJK content test PASSED");
}

#[test]
fn e2e_table_emoji_content() {
    init_test_logging();
    tracing::info!("Starting E2E table with emoji content test");

    let mut table = Table::new()
        .with_column(Column::new("Status"))
        .with_column(Column::new("Task"));

    table.add_row_cells(["✓", "Complete"]);
    table.add_row_cells(["⏳", "In Progress"]);
    table.add_row_cells(["✗", "Failed"]);

    let output = table.render_plain(30);
    tracing::debug!(output = %output, "Emoji table");

    assert!(output.contains("✓"), "Missing checkmark");
    assert!(output.contains("⏳"), "Missing hourglass");
    assert!(output.contains("✗"), "Missing X");

    tracing::info!("E2E table with emoji content test PASSED");
}

#[test]
fn e2e_table_mixed_width_chars() {
    init_test_logging();
    tracing::info!("Starting E2E table with mixed-width characters test");

    let mut table = Table::new()
        .with_column(Column::new("Mixed"))
        .with_column(Column::new("Content"));

    table.add_row_cells(["Hello世界!", "αβγδ"]);
    table.add_row_cells(["你好World", "Café"]);

    let output = table.render_plain(40);
    tracing::debug!(output = %output, "Mixed-width table");

    assert!(output.contains("Hello世界!"), "Missing mixed content");
    assert!(output.contains("你好World"), "Missing mixed content");

    tracing::info!("E2E table with mixed-width characters test PASSED");
}

// =============================================================================
// Scenario 5: Multi-line Cells
// =============================================================================

#[test]
fn e2e_table_cell_wrapping() {
    init_test_logging();
    tracing::info!("Starting E2E table cell wrapping test");

    // Note: The current implementation may not support multi-line cells fully,
    // but let's test that long content at least renders without panic
    let mut table = Table::new().with_column(Column::new("Description").width(15));

    table.add_row_cells(["This is a very long piece of text that would need wrapping"]);

    let output = table.render_plain(30);
    tracing::debug!(output = %output, "Wrapped cell table");

    // Should render something
    assert!(!output.is_empty(), "Table should render");
    assert!(output.contains("Description"), "Should have header");

    tracing::info!("E2E table cell wrapping test PASSED");
}

// =============================================================================
// Scenario 6: Column Alignment
// =============================================================================

#[test]
fn e2e_table_right_align() {
    init_test_logging();
    tracing::info!("Starting E2E table right alignment test");

    let mut table = Table::new()
        .with_column(Column::new("Item"))
        .with_column(Column::new("Price").justify(JustifyMethod::Right));

    table.add_row_cells(["Widget", "$10"]);
    table.add_row_cells(["Gadget", "$100"]);
    table.add_row_cells(["Gizmo", "$1,000"]);

    let output = table.render_plain(30);
    tracing::debug!(output = %output, "Right-aligned table");

    assert!(output.contains("Item"), "Missing header");
    assert!(output.contains("Price"), "Missing header");
    assert!(output.contains("$10"), "Missing price");

    tracing::info!("E2E table right alignment test PASSED");
}

#[test]
fn e2e_table_center_align() {
    init_test_logging();
    tracing::info!("Starting E2E table center alignment test");

    let mut table = Table::new().with_column(
        Column::new("Centered")
            .justify(JustifyMethod::Center)
            .width(20),
    );

    table.add_row_cells(["X"]);
    table.add_row_cells(["Short"]);
    table.add_row_cells(["Medium Text"]);

    let output = table.render_plain(30);
    tracing::debug!(output = %output, "Center-aligned table");

    assert!(output.contains("Centered"), "Missing header");
    assert!(output.contains("X"), "Missing short content");

    tracing::info!("E2E table center alignment test PASSED");
}

// =============================================================================
// Scenario 7: Edge Cases
// =============================================================================

#[test]
fn e2e_table_empty() {
    init_test_logging();
    tracing::info!("Starting E2E empty table test");

    let table = Table::new();
    let output = table.render_plain(40);
    tracing::debug!(output = %output, "Empty table");

    // Empty table should render to empty string or minimal content
    assert!(
        output.is_empty() || output.trim().is_empty(),
        "Empty table should be empty"
    );

    tracing::info!("E2E empty table test PASSED");
}

#[test]
fn e2e_table_columns_no_rows() {
    init_test_logging();
    tracing::info!("Starting E2E table with columns but no rows test");

    let table = Table::new()
        .with_column(Column::new("Header1"))
        .with_column(Column::new("Header2"));

    let output = table.render_plain(40);
    tracing::debug!(output = %output, "Headers-only table");

    // Should show headers
    assert!(output.contains("Header1"), "Missing header");
    assert!(output.contains("Header2"), "Missing header");

    tracing::info!("E2E table with columns but no rows test PASSED");
}

#[test]
fn e2e_table_single_cell() {
    init_test_logging();
    tracing::info!("Starting E2E single cell table test");

    let mut table = Table::new().with_column(Column::new("Solo"));

    table.add_row_cells(["One"]);

    let output = table.render_plain(20);
    tracing::debug!(output = %output, "Single cell table");

    assert!(output.contains("Solo"), "Missing header");
    assert!(output.contains("One"), "Missing cell");

    tracing::info!("E2E single cell table test PASSED");
}

#[test]
fn e2e_table_empty_cells() {
    init_test_logging();
    tracing::info!("Starting E2E table with empty cells test");

    let mut table = Table::new()
        .with_column(Column::new("A"))
        .with_column(Column::new("B"))
        .with_column(Column::new("C"));

    table.add_row_cells(["1", "", "3"]);
    table.add_row_cells(["", "2", ""]);

    let output = table.render_plain(30);
    tracing::debug!(output = %output, "Table with empty cells");

    assert!(output.contains("A"), "Missing header");
    assert!(output.contains("1"), "Missing cell content");
    assert!(output.contains("3"), "Missing cell content");

    tracing::info!("E2E table with empty cells test PASSED");
}

#[test]
fn e2e_table_sparse_rows() {
    init_test_logging();
    tracing::info!("Starting E2E table with sparse rows test");

    // Rows with fewer cells than columns
    let mut table = Table::new()
        .with_column(Column::new("A"))
        .with_column(Column::new("B"))
        .with_column(Column::new("C"));

    table.add_row_cells(["Only A"]); // Missing B and C
    table.add_row_cells(["X", "Y"]); // Missing C

    let output = table.render_plain(40);
    tracing::debug!(output = %output, "Sparse rows table");

    assert!(output.contains("Only A"), "Missing sparse row content");
    assert!(output.contains("X"), "Missing row content");
    assert!(output.contains("Y"), "Missing row content");

    tracing::info!("E2E table with sparse rows test PASSED");
}

#[test]
fn e2e_table_no_edges() {
    init_test_logging();
    tracing::info!("Starting E2E table without edges test");

    let mut table = Table::new()
        .show_edge(false)
        .with_column(Column::new("Col1"))
        .with_column(Column::new("Col2"));

    table.add_row_cells(["A", "B"]);

    let output = table.render_plain(30);
    tracing::debug!(output = %output, "Table without edges");

    // Should have content
    assert!(output.contains("Col1"), "Missing header");
    assert!(output.contains("A"), "Missing cell");

    tracing::info!("E2E table without edges test PASSED");
}

// =============================================================================
// Scenario 8: Styled Tables
// =============================================================================

#[test]
fn e2e_table_with_styles() {
    init_test_logging();
    tracing::info!("Starting E2E styled table test");

    let bold = Style::new().bold();
    let red = Style::new().color(Color::parse("red").unwrap());

    let mut table = Table::new()
        .header_style(bold)
        .border_style(red)
        .with_column(Column::new("Styled"))
        .with_column(Column::new("Table"));

    table.add_row_cells(["Data", "Here"]);

    let segments = table.render(40);
    let output: String = segments.iter().map(|s| s.text.as_str()).collect();
    tracing::debug!(output = %output, "Styled table output");

    // Check that styles are present (segments have style)
    let has_styled_segments = segments.iter().any(|s| s.style.is_some());
    assert!(has_styled_segments, "Should have styled segments");

    tracing::info!("E2E styled table test PASSED");
}

#[test]
fn e2e_table_alternating_rows() {
    init_test_logging();
    tracing::info!("Starting E2E table with alternating row styles test");

    let style1 = Style::new();
    let style2 = Style::new().dim();

    let mut table = Table::new()
        .row_styles(vec![style1, style2])
        .with_column(Column::new("Row"));

    table.add_row_cells(["One"]);
    table.add_row_cells(["Two"]);
    table.add_row_cells(["Three"]);
    table.add_row_cells(["Four"]);

    let output = table.render_plain(20);
    tracing::debug!(output = %output, "Alternating row styles table");

    assert!(output.contains("One"), "Missing row 1");
    assert!(output.contains("Four"), "Missing row 4");

    tracing::info!("E2E table with alternating row styles test PASSED");
}

// =============================================================================
// Snapshot Tests for Visual Regression
// =============================================================================

#[test]
fn e2e_snapshot_simple_table() {
    init_test_logging();

    let mut table = Table::new()
        .with_column(Column::new("Name"))
        .with_column(Column::new("Age"))
        .with_column(Column::new("City"));

    table.add_row_cells(["Alice", "30", "NYC"]);
    table.add_row_cells(["Bob", "25", "LA"]);
    table.add_row_cells(["Carol", "35", "Chicago"]);

    let output = table.render_plain(50);
    insta::assert_snapshot!("e2e_simple_table", output);
}

#[test]
fn e2e_snapshot_ascii_table() {
    init_test_logging();

    let mut table = Table::new()
        .ascii()
        .with_column(Column::new("ID"))
        .with_column(Column::new("Status"));

    table.add_row_cells(["1", "OK"]);
    table.add_row_cells(["2", "FAIL"]);

    let output = table.render_plain(30);
    insta::assert_snapshot!("e2e_ascii_table", output);
}

#[test]
fn e2e_snapshot_table_with_title() {
    init_test_logging();

    let mut table = Table::new()
        .title("Monthly Report")
        .caption("Generated: 2024-01-15")
        .with_column(Column::new("Month"))
        .with_column(Column::new("Revenue").justify(JustifyMethod::Right));

    table.add_row_cells(["January", "$10,000"]);
    table.add_row_cells(["February", "$12,500"]);
    table.add_row_cells(["March", "$15,000"]);

    let output = table.render_plain(40);
    insta::assert_snapshot!("e2e_table_with_title", output);
}

#[test]
fn e2e_snapshot_table_all_features() {
    init_test_logging();

    let mut table = Table::new()
        .title("Complete Table")
        .caption("All features enabled")
        .show_footer(true)
        .show_lines(true)
        .with_column(Column::new("Product").footer("Total"))
        .with_column(
            Column::new("Qty")
                .justify(JustifyMethod::Center)
                .footer("10"),
        )
        .with_column(
            Column::new("Price")
                .justify(JustifyMethod::Right)
                .footer("$250"),
        );

    table.add_row_cells(["Widget", "3", "$75"]);
    table.add_row_cells(["Gadget", "5", "$125"]);
    table.add_row_cells(["Gizmo", "2", "$50"]);

    let output = table.render_plain(45);
    insta::assert_snapshot!("e2e_table_all_features", output);
}
