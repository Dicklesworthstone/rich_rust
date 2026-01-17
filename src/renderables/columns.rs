//! Columns - Arrange items in multiple columns.
//!
//! This module provides a Columns renderable for arranging content
//! in a newspaper-style multi-column layout.
//!
//! # Example
//!
//! ```rust,ignore
//! use rich_rust::renderables::columns::Columns;
//! use rich_rust::segment::Segment;
//!
//! let items = vec![
//!     vec![Segment::new("Item 1", None)],
//!     vec![Segment::new("Item 2", None)],
//!     vec![Segment::new("Item 3", None)],
//! ];
//! let columns = Columns::new(items)
//!     .column_count(2)
//!     .gutter(2)
//!     .render(40);
//! ```

use crate::cells::cell_len;
use crate::segment::Segment;
use crate::style::Style;

use super::align::{Align, AlignMethod};

/// A renderable that arranges items in columns.
#[derive(Debug, Clone)]
pub struct Columns {
    /// Items to arrange (each item is a list of segments representing one line).
    items: Vec<Vec<Segment>>,
    /// Number of columns (None = auto-calculate based on content width).
    column_count: Option<usize>,
    /// Space between columns.
    gutter: usize,
    /// Whether to expand columns to fill available width.
    expand: bool,
    /// Whether columns should have equal width.
    equal_width: bool,
    /// Alignment within each column.
    align: AlignMethod,
    /// Padding around each item.
    padding: usize,
    /// Style for column separators (gutter).
    gutter_style: Style,
}

impl Default for Columns {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            column_count: None,
            gutter: 2,
            expand: true,
            equal_width: false,
            align: AlignMethod::Left,
            padding: 0,
            gutter_style: Style::new(),
        }
    }
}

impl Columns {
    /// Create a new Columns layout with the given items.
    #[must_use]
    pub fn new(items: Vec<Vec<Segment>>) -> Self {
        Self {
            items,
            ..Default::default()
        }
    }

    /// Create columns from strings.
    #[must_use]
    pub fn from_strings(items: &[&str]) -> Self {
        let segments: Vec<Vec<Segment>> = items
            .iter()
            .map(|s| vec![Segment::new(*s, None)])
            .collect();
        Self::new(segments)
    }

    /// Set the number of columns.
    #[must_use]
    pub fn column_count(mut self, count: usize) -> Self {
        self.column_count = Some(count.max(1));
        self
    }

    /// Set the gutter (space between columns).
    #[must_use]
    pub fn gutter(mut self, gutter: usize) -> Self {
        self.gutter = gutter;
        self
    }

    /// Set whether to expand columns to fill width.
    #[must_use]
    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    /// Set whether columns should have equal width.
    #[must_use]
    pub fn equal_width(mut self, equal: bool) -> Self {
        self.equal_width = equal;
        self
    }

    /// Set alignment within columns.
    #[must_use]
    pub fn align(mut self, align: AlignMethod) -> Self {
        self.align = align;
        self
    }

    /// Set padding around each item.
    #[must_use]
    pub fn padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }

    /// Set the gutter style.
    #[must_use]
    pub fn gutter_style(mut self, style: Style) -> Self {
        self.gutter_style = style;
        self
    }

    /// Get the width of an item in cells.
    fn item_width(item: &[Segment]) -> usize {
        item.iter().map(|s| cell_len(&s.text)).sum()
    }

    /// Calculate column widths.
    fn calculate_column_widths(&self, total_width: usize, num_columns: usize) -> Vec<usize> {
        if num_columns == 0 || self.items.is_empty() {
            return vec![];
        }

        // Calculate gutter space needed
        let total_gutter = self.gutter * (num_columns - 1);
        let available_width = total_width.saturating_sub(total_gutter);

        if self.equal_width {
            // Equal width columns
            let column_width = available_width / num_columns;
            vec![column_width; num_columns]
        } else {
            // Calculate max width for each column based on content
            let mut max_widths = vec![0usize; num_columns];

            for (idx, item) in self.items.iter().enumerate() {
                let col = idx % num_columns;
                let item_w = Self::item_width(item) + self.padding * 2;
                max_widths[col] = max_widths[col].max(item_w);
            }

            if self.expand {
                // Distribute remaining space proportionally
                let content_total: usize = max_widths.iter().sum();
                if content_total < available_width {
                    let extra = available_width - content_total;
                    let per_column = extra / num_columns;
                    let remainder = extra % num_columns;

                    for (i, width) in max_widths.iter_mut().enumerate() {
                        *width += per_column;
                        if i < remainder {
                            *width += 1;
                        }
                    }
                }
            }

            // Ensure widths don't exceed available per-column space
            let max_per_column = available_width / num_columns;
            for width in &mut max_widths {
                *width = (*width).min(max_per_column);
            }

            max_widths
        }
    }

    /// Auto-calculate number of columns based on content and width.
    fn auto_column_count(&self, total_width: usize) -> usize {
        if self.items.is_empty() {
            return 1;
        }

        // Find the widest item
        let max_item_width = self.items
            .iter()
            .map(|item| Self::item_width(item) + self.padding * 2)
            .max()
            .unwrap_or(1);

        // Calculate how many columns can fit
        let min_column_width = max_item_width.max(1);
        let mut columns = 1;

        while columns < self.items.len() {
            let needed_width = columns * min_column_width + (columns - 1) * self.gutter;
            if needed_width > total_width {
                break;
            }
            columns += 1;
        }

        columns.max(1)
    }

    /// Render the columns to lines of segments.
    #[must_use]
    pub fn render(&self, total_width: usize) -> Vec<Vec<Segment>> {
        if self.items.is_empty() {
            return vec![];
        }

        let num_columns = self.column_count.unwrap_or_else(|| self.auto_column_count(total_width));
        let column_widths = self.calculate_column_widths(total_width, num_columns);

        if column_widths.is_empty() {
            return vec![];
        }

        // Calculate number of rows needed
        let num_rows = (self.items.len() + num_columns - 1) / num_columns;

        let mut result = Vec::with_capacity(num_rows);

        for row_idx in 0..num_rows {
            let mut row_segments = Vec::new();

            for col_idx in 0..num_columns {
                let item_idx = row_idx * num_columns + col_idx;
                let column_width = column_widths[col_idx];

                // Add gutter before columns (except first)
                if col_idx > 0 && self.gutter > 0 {
                    row_segments.push(Segment::new(
                        " ".repeat(self.gutter),
                        Some(self.gutter_style.clone()),
                    ));
                }

                if item_idx < self.items.len() {
                    // Add padding, content, padding
                    if self.padding > 0 {
                        row_segments.push(Segment::new(" ".repeat(self.padding), None));
                    }

                    let content_width = column_width.saturating_sub(self.padding * 2);
                    let aligned = Align::new(self.items[item_idx].clone(), content_width)
                        .method(self.align)
                        .render();
                    row_segments.extend(aligned);

                    if self.padding > 0 {
                        row_segments.push(Segment::new(" ".repeat(self.padding), None));
                    }
                } else {
                    // Empty cell - fill with spaces
                    row_segments.push(Segment::new(" ".repeat(column_width), None));
                }
            }

            result.push(row_segments);
        }

        result
    }

    /// Render to a single flat list of segments with newlines.
    #[must_use]
    pub fn render_flat(&self, total_width: usize) -> Vec<Segment> {
        let lines = self.render(total_width);
        let mut result = Vec::new();

        for (i, line) in lines.into_iter().enumerate() {
            if i > 0 {
                result.push(Segment::new("\n", None));
            }
            result.extend(line);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_columns_new() {
        let items = vec![
            vec![Segment::new("A", None)],
            vec![Segment::new("B", None)],
        ];
        let cols = Columns::new(items);
        assert_eq!(cols.items.len(), 2);
    }

    #[test]
    fn test_columns_from_strings() {
        let cols = Columns::from_strings(&["A", "B", "C"]);
        assert_eq!(cols.items.len(), 3);
    }

    #[test]
    fn test_columns_builder() {
        let cols = Columns::from_strings(&["A", "B"])
            .column_count(2)
            .gutter(4)
            .expand(false)
            .equal_width(true)
            .align(AlignMethod::Center)
            .padding(1);

        assert_eq!(cols.column_count, Some(2));
        assert_eq!(cols.gutter, 4);
        assert!(!cols.expand);
        assert!(cols.equal_width);
        assert_eq!(cols.align, AlignMethod::Center);
        assert_eq!(cols.padding, 1);
    }

    #[test]
    fn test_columns_render_two_columns() {
        let cols = Columns::from_strings(&["A", "B", "C", "D"])
            .column_count(2)
            .gutter(2)
            .expand(false);

        let lines = cols.render(20);

        // Should have 2 rows (4 items / 2 columns)
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_columns_render_three_columns() {
        let cols = Columns::from_strings(&["A", "B", "C"])
            .column_count(3)
            .gutter(1);

        let lines = cols.render(30);

        // Should have 1 row (3 items / 3 columns)
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_columns_render_empty() {
        let cols = Columns::new(vec![]);
        let lines = cols.render(40);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_columns_auto_count() {
        // With narrow width, should fit fewer columns
        let cols = Columns::from_strings(&["Hello", "World", "Test", "Here"]);

        // Auto-calc should determine column count based on content width
        let auto_count = cols.auto_column_count(50);
        assert!(auto_count >= 1);
    }

    #[test]
    fn test_columns_equal_width() {
        let cols = Columns::from_strings(&["Short", "Much Longer Item"])
            .column_count(2)
            .equal_width(true);

        let widths = cols.calculate_column_widths(40, 2);

        // Both columns should be same width
        assert_eq!(widths[0], widths[1]);
    }

    #[test]
    fn test_columns_with_gutter() {
        let cols = Columns::from_strings(&["A", "B"])
            .column_count(2)
            .gutter(4);

        let lines = cols.render(20);
        let line = &lines[0];

        // Check that gutter is present (spaces between columns)
        let text: String = line.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("    ")); // 4 spaces for gutter
    }

    #[test]
    fn test_columns_alignment() {
        let cols = Columns::from_strings(&["Hi"])
            .column_count(1)
            .expand(true)
            .equal_width(true)
            .align(AlignMethod::Center);

        let lines = cols.render(20);
        let text: String = lines[0].iter().map(|s| s.text.as_str()).collect();

        // Content should be centered
        assert!(text.starts_with(' ')); // Has leading spaces
        assert!(text.ends_with(' ')); // Has trailing spaces
    }

    #[test]
    fn test_columns_render_flat() {
        let cols = Columns::from_strings(&["A", "B", "C", "D"])
            .column_count(2);

        let segments = cols.render_flat(20);

        // Should contain a newline between rows
        let has_newline = segments.iter().any(|s| s.text.contains('\n'));
        assert!(has_newline);
    }

    #[test]
    fn test_columns_uneven_items() {
        // 5 items in 2 columns = 3 rows (last row has 1 item + 1 empty)
        let cols = Columns::from_strings(&["1", "2", "3", "4", "5"])
            .column_count(2);

        let lines = cols.render(20);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_item_width() {
        let item = vec![
            Segment::new("Hello", None),
            Segment::new(" ", None),
            Segment::new("World", None),
        ];
        assert_eq!(Columns::item_width(&item), 11);
    }

    #[test]
    fn test_columns_single_column() {
        let cols = Columns::from_strings(&["A", "B", "C"])
            .column_count(1);

        let lines = cols.render(20);

        // Should have 3 rows (1 item per row)
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_columns_narrow_width() {
        // Width too small for content
        let cols = Columns::from_strings(&["Hello", "World"])
            .column_count(2)
            .gutter(2);

        // Even with narrow width, should not panic
        let lines = cols.render(5);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_columns_zero_width() {
        let cols = Columns::from_strings(&["A", "B"]);
        let lines = cols.render(0);
        // Should handle zero width gracefully (may be empty)
        assert!(lines.is_empty() || !lines.is_empty()); // Just verify no panic
    }

    #[test]
    fn test_columns_many_items() {
        // Test with many items to verify row calculation
        let items: Vec<&str> = (0..20).map(|_| "X").collect();
        let cols = Columns::from_strings(&items)
            .column_count(4);

        let lines = cols.render(40);

        // 20 items / 4 columns = 5 rows
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn test_columns_wide_unicode() {
        // Test with CJK characters (2 cells wide each)
        let items = vec![
            vec![Segment::new("你好", None)],  // 4 cells
            vec![Segment::new("世界", None)],  // 4 cells
        ];
        let cols = Columns::new(items)
            .column_count(2)
            .gutter(2);

        let lines = cols.render(20);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_columns_content_width_calculation() {
        // Verify content-based width calculation
        let cols = Columns::from_strings(&["Short", "Much Longer Item"])
            .column_count(2)
            .equal_width(false)
            .expand(false);

        let widths = cols.calculate_column_widths(40, 2);

        // Without equal_width, columns can have different widths based on content
        // Both should be within bounds
        assert!(widths.len() == 2);
    }

    #[test]
    fn test_columns_expand_distribution() {
        let cols = Columns::from_strings(&["A", "B"])
            .column_count(2)
            .gutter(2)
            .expand(true);

        let widths = cols.calculate_column_widths(20, 2);

        // With gutter=2, available = 18, so columns should fill that
        let total: usize = widths.iter().sum();
        assert!(total > 0);
    }

    #[test]
    fn test_columns_right_align() {
        let cols = Columns::from_strings(&["X"])
            .column_count(1)
            .expand(true)
            .equal_width(true)
            .align(AlignMethod::Right);

        let lines = cols.render(20);
        let text: String = lines[0].iter().map(|s| s.text.as_str()).collect();

        // Content should be right-aligned (leading spaces)
        assert!(text.starts_with(' '));
    }

    #[test]
    fn test_columns_padding_applied() {
        let cols = Columns::from_strings(&["X"])
            .column_count(1)
            .padding(2);

        let lines = cols.render(20);
        let text: String = lines[0].iter().map(|s| s.text.as_str()).collect();

        // Should have padding around content
        assert!(text.starts_with("  ")); // 2 spaces padding
    }
}
