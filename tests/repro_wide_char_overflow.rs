use rich_rust::console::Console;
use rich_rust::renderables::table::{Column, Table};
use rich_rust::segment::Segment;

#[test]
#[ignore = "Bug reproduction test: wide char overflow in narrow column"]
fn test_wide_char_in_narrow_column() {
    let mut table = Table::new().with_column(Column::new("A").width(1)).ascii(); // Use ASCII to make length calc easier

    // CJK character '日' is 2 cells wide
    table.add_row_cells(["日"]);

    // Render with enough space that the table constraint (width=1) is the limiting factor
    let segments = table.render(20);
    let output: String = segments.iter().map(|s| s.text.as_ref()).collect();

    // Expected visual:
    // +-+
    // |A|
    // +-+
    // |日|  <-- This line will be width 4 (border+char+border), but header is width 3.
    // +-+

    // Let's verify line lengths.
    let lines: Vec<&str> = output.lines().collect();

    println!("Output:\n{}", output);

    let header_width = lines[1].len(); // "|A|" -> 3 chars
    let row_width = lines[3].len(); // "|日|" -> '|' (1) + '日' (3 bytes) + '|' (1) = 5 bytes? 
    // Wait, we need cell length, not byte length.

    use rich_rust::cells::cell_len;

    let header_cell_len = cell_len(lines[1]);
    let row_cell_len = cell_len(lines[3]);

    assert_eq!(
        header_cell_len, row_cell_len,
        "Table border misalignment detected! Header: {}, Row: {}",
        header_cell_len, row_cell_len
    );
}
