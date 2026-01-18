//! Table rendering example demonstrating columns, rows, and styling.
//!
//! Run with: `cargo run --example tables`

use rich_rust::r#box::{DOUBLE, HEAVY, MINIMAL, ROUNDED, SIMPLE};
use rich_rust::prelude::*;

fn main() {
    let console = Console::new();
    let width = console.width().min(80);

    // ========================================================================
    // Basic Table
    // ========================================================================
    println!("\n=== Basic Table ===\n");

    let mut basic_table = Table::new()
        .with_column(Column::new("Name"))
        .with_column(Column::new("Age"))
        .with_column(Column::new("City"));

    basic_table.add_row_cells(["Alice", "30", "New York"]);
    basic_table.add_row_cells(["Bob", "25", "San Francisco"]);
    basic_table.add_row_cells(["Charlie", "35", "Chicago"]);

    for seg in basic_table.render(width) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Table with Title and Caption
    // ========================================================================
    println!("\n=== Table with Title and Caption ===\n");

    let mut titled_table = Table::new()
        .title("Employee Directory")
        .caption("Updated: 2024")
        .with_column(Column::new("ID"))
        .with_column(Column::new("Name"))
        .with_column(Column::new("Department"));

    titled_table.add_row_cells(["001", "Alice Smith", "Engineering"]);
    titled_table.add_row_cells(["002", "Bob Jones", "Marketing"]);
    titled_table.add_row_cells(["003", "Carol White", "Finance"]);

    for seg in titled_table.render(width) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Column Alignment
    // ========================================================================
    println!("\n=== Column Alignment ===\n");

    let mut aligned_table = Table::new()
        .title("Sales Report")
        .with_column(Column::new("Product").justify(JustifyMethod::Left))
        .with_column(Column::new("Quantity").justify(JustifyMethod::Center))
        .with_column(Column::new("Price").justify(JustifyMethod::Right))
        .with_column(Column::new("Total").justify(JustifyMethod::Right));

    aligned_table.add_row_cells(["Widget A", "100", "$10.00", "$1,000.00"]);
    aligned_table.add_row_cells(["Widget B", "50", "$25.00", "$1,250.00"]);
    aligned_table.add_row_cells(["Widget C", "200", "$5.00", "$1,000.00"]);

    for seg in aligned_table.render(width) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Styled Headers and Borders
    // ========================================================================
    println!("\n=== Styled Headers and Borders ===\n");

    let mut styled_table = Table::new()
        .title("System Status")
        .header_style(Style::parse("bold cyan").unwrap_or_default())
        .border_style(Style::parse("green").unwrap_or_default())
        .title_style(Style::parse("bold yellow").unwrap_or_default())
        .with_column(Column::new("Service"))
        .with_column(Column::new("Status"))
        .with_column(Column::new("Uptime"));

    styled_table.add_row_cells(["Database", "Running", "99.9%"]);
    styled_table.add_row_cells(["API Server", "Running", "99.5%"]);
    styled_table.add_row_cells(["Cache", "Degraded", "95.0%"]);

    for seg in styled_table.render(width) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Different Box Styles
    // ========================================================================
    println!("\n=== Box Styles ===\n");

    // Rounded (default)
    println!("Rounded (default):");
    let mut rounded_table = Table::new()
        .box_style(&ROUNDED)
        .with_column(Column::new("A"))
        .with_column(Column::new("B"));
    rounded_table.add_row_cells(["1", "2"]);
    for seg in rounded_table.render(40) {
        print!("{}", seg.text);
    }

    // Square
    println!("Square:");
    let mut square_table = Table::new()
        .with_column(Column::new("A"))
        .with_column(Column::new("B"));
    square_table.add_row_cells(["1", "2"]);
    for seg in square_table.render(40) {
        print!("{}", seg.text);
    }

    // ASCII
    println!("ASCII:");
    let mut ascii_table = Table::new()
        .ascii()
        .with_column(Column::new("A"))
        .with_column(Column::new("B"));
    ascii_table.add_row_cells(["1", "2"]);
    for seg in ascii_table.render(40) {
        print!("{}", seg.text);
    }

    // Double
    println!("Double:");
    let mut double_table = Table::new()
        .box_style(&DOUBLE)
        .with_column(Column::new("A"))
        .with_column(Column::new("B"));
    double_table.add_row_cells(["1", "2"]);
    for seg in double_table.render(40) {
        print!("{}", seg.text);
    }

    // Heavy
    println!("Heavy:");
    let mut heavy_table = Table::new()
        .box_style(&HEAVY)
        .with_column(Column::new("A"))
        .with_column(Column::new("B"));
    heavy_table.add_row_cells(["1", "2"]);
    for seg in heavy_table.render(40) {
        print!("{}", seg.text);
    }

    // Minimal
    println!("Minimal:");
    let mut minimal_table = Table::new()
        .box_style(&MINIMAL)
        .with_column(Column::new("A"))
        .with_column(Column::new("B"));
    minimal_table.add_row_cells(["1", "2"]);
    for seg in minimal_table.render(40) {
        print!("{}", seg.text);
    }

    // Simple
    println!("Simple:");
    let mut simple_table = Table::new()
        .box_style(&SIMPLE)
        .with_column(Column::new("A"))
        .with_column(Column::new("B"));
    simple_table.add_row_cells(["1", "2"]);
    for seg in simple_table.render(40) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Table with Row Lines
    // ========================================================================
    println!("\n=== Table with Row Lines ===\n");

    let mut lined_table = Table::new()
        .show_lines(true)
        .with_column(Column::new("Step"))
        .with_column(Column::new("Description"));

    lined_table.add_row_cells(["1", "Initialize system"]);
    lined_table.add_row_cells(["2", "Load configuration"]);
    lined_table.add_row_cells(["3", "Start services"]);
    lined_table.add_row_cells(["4", "Ready"]);

    for seg in lined_table.render(width) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Table without Header
    // ========================================================================
    println!("\n=== Table without Header ===\n");

    let mut no_header_table = Table::new()
        .show_header(false)
        .with_column(Column::new("Key"))
        .with_column(Column::new("Value"));

    no_header_table.add_row_cells(["name", "rich_rust"]);
    no_header_table.add_row_cells(["version", "0.1.0"]);
    no_header_table.add_row_cells(["license", "MIT"]);

    for seg in no_header_table.render(width) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Table without Edges
    // ========================================================================
    println!("\n=== Table without Edges ===\n");

    let mut no_edge_table = Table::new()
        .show_edge(false)
        .with_column(Column::new("Col A"))
        .with_column(Column::new("Col B"))
        .with_column(Column::new("Col C"));

    no_edge_table.add_row_cells(["A1", "B1", "C1"]);
    no_edge_table.add_row_cells(["A2", "B2", "C2"]);

    for seg in no_edge_table.render(width) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Compact Table (No Padding)
    // ========================================================================
    println!("\n=== Compact Table (No Padding) ===\n");

    let mut compact_table = Table::new()
        .padding(0, 0)
        .with_column(Column::new("X"))
        .with_column(Column::new("Y"))
        .with_column(Column::new("Z"));

    compact_table.add_row_cells(["1", "2", "3"]);
    compact_table.add_row_cells(["4", "5", "6"]);

    for seg in compact_table.render(width) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Table with Column Widths
    // ========================================================================
    println!("\n=== Table with Column Widths ===\n");

    let mut width_table = Table::new()
        .title("Fixed Width Columns")
        .with_column(Column::new("ID").width(6))
        .with_column(Column::new("Name").width(20))
        .with_column(Column::new("Score").width(10).justify(JustifyMethod::Right));

    width_table.add_row_cells(["1", "Alice", "95"]);
    width_table.add_row_cells(["2", "Bob", "87"]);
    width_table.add_row_cells(["3", "Charlie", "92"]);

    for seg in width_table.render(width) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Alternating Row Styles
    // ========================================================================
    println!("\n=== Alternating Row Styles ===\n");

    let mut zebra_table = Table::new()
        .title("Zebra Striped Table")
        .row_styles(vec![Style::new(), Style::parse("dim").unwrap_or_default()])
        .with_column(Column::new("Item"))
        .with_column(Column::new("Qty"))
        .with_column(Column::new("Price"));

    zebra_table.add_row_cells(["Apples", "10", "$5.00"]);
    zebra_table.add_row_cells(["Bananas", "6", "$3.00"]);
    zebra_table.add_row_cells(["Oranges", "8", "$4.00"]);
    zebra_table.add_row_cells(["Grapes", "2", "$6.00"]);
    zebra_table.add_row_cells(["Mangoes", "4", "$8.00"]);

    for seg in zebra_table.render(width) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Real-World Example: Server List
    // ========================================================================
    println!("\n=== Real-World: Server List ===\n");

    let mut server_table = Table::new()
        .title("Production Servers")
        .caption("Last updated: 2024-01-15 10:30 UTC")
        .header_style(Style::parse("bold white on blue").unwrap_or_default())
        .border_style(Style::parse("blue").unwrap_or_default())
        .with_column(Column::new("Hostname").min_width(15))
        .with_column(Column::new("IP Address").width(15))
        .with_column(
            Column::new("Status")
                .width(10)
                .justify(JustifyMethod::Center),
        )
        .with_column(Column::new("CPU").width(8).justify(JustifyMethod::Right))
        .with_column(
            Column::new("Memory")
                .width(10)
                .justify(JustifyMethod::Right),
        );

    server_table.add_row_cells(["web-prod-01", "10.0.1.10", "Running", "45%", "62%"]);
    server_table.add_row_cells(["web-prod-02", "10.0.1.11", "Running", "52%", "58%"]);
    server_table.add_row_cells(["api-prod-01", "10.0.2.10", "Running", "78%", "71%"]);
    server_table.add_row_cells(["db-prod-01", "10.0.3.10", "Running", "35%", "82%"]);
    server_table.add_row_cells(["cache-prod-01", "10.0.4.10", "Warning", "92%", "45%"]);

    for seg in server_table.render(width) {
        print!("{}", seg.text);
    }

    // ========================================================================
    // Section Divider
    // ========================================================================
    let rule =
        Rule::with_title("End of Table Demo").style(Style::parse("bold green").unwrap_or_default());
    for seg in rule.render(width) {
        print!("{}", seg.text);
    }

    println!();
}
