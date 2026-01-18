//! Table - structured data display with columns and rows.
//!
//! A [`Table`] renders data in a grid with configurable columns,
//! optional headers/footers, and flexible styling. Tables support:
//!
//! - Automatic column width calculation
//! - Fixed, minimum, and maximum column widths
//! - Text wrapping and overflow handling
//! - Header and footer rows
//! - Cell-level styling and alignment
//! - Unicode and ASCII box characters
//!
//! # Examples
//!
//! ## Basic Table
//!
//! ```
//! use rich_rust::renderables::table::{Table, Column, Row, Cell};
//!
//! let mut table = Table::new()
//!     .with_column(Column::new("Name"))
//!     .with_column(Column::new("Age"));
//! table.add_row_cells(["Alice", "30"]);
//! table.add_row_cells(["Bob", "25"]);
//!
//! // Render at 40 characters width
//! let segments = table.render(40);
//! for seg in segments {
//!     print!("{}", seg.text);
//! }
//! ```
//!
//! ## Styled Table
//!
//! ```
//! use rich_rust::renderables::table::{Table, Column, Row, VerticalAlign};
//! use rich_rust::style::Style;
//! use rich_rust::text::JustifyMethod;
//!
//! let table = Table::new()
//!     .title("Employee Directory")
//!     .with_column(Column::new("Name")
//!         .style(Style::new().bold())
//!         .min_width(15))
//!     .with_column(Column::new("Department")
//!         .justify(JustifyMethod::Center))
//!     .with_column(Column::new("Salary")
//!         .justify(JustifyMethod::Right));
//! ```
//!
//! ## Column Configuration
//!
//! Columns support various configuration options:
//!
//! - `width(n)`: Fixed width in characters
//! - `min_width(n)`: Minimum width
//! - `max_width(n)`: Maximum width
//! - `justify(method)`: Left, right, center, or full justification
//! - `no_wrap`: Disable text wrapping
//! - `style(s)`: Apply a style to cell content

use crate::r#box::{ASCII, BoxChars, RowLevel, SQUARE};
use crate::cells;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::{JustifyMethod, OverflowMethod, Text};
use num_rational::Ratio;

// PaddingDimensions is available but not needed for current implementation

/// Vertical alignment methods for cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalAlign {
    /// Align to top of cell.
    #[default]
    Top,
    /// Align to middle of cell.
    Middle,
    /// Align to bottom of cell.
    Bottom,
}

/// Column definition for a table.
#[derive(Debug, Clone)]
pub struct Column {
    /// Header text.
    pub header: Text,
    /// Footer text.
    pub footer: Text,
    /// Style for header.
    pub header_style: Style,
    /// Style for footer.
    pub footer_style: Style,
    /// Style for cell content.
    pub style: Style,
    /// Content justification.
    pub justify: JustifyMethod,
    /// Vertical alignment.
    pub vertical: VerticalAlign,
    /// Overflow handling.
    pub overflow: OverflowMethod,
    /// Fixed width.
    pub width: Option<usize>,
    /// Minimum width.
    pub min_width: Option<usize>,
    /// Maximum width.
    pub max_width: Option<usize>,
    /// Ratio for flexible sizing.
    pub ratio: Option<usize>,
    /// Disable text wrapping.
    pub no_wrap: bool,
}

impl Default for Column {
    fn default() -> Self {
        Self {
            header: Text::new(""),
            footer: Text::new(""),
            header_style: Style::new(),
            footer_style: Style::new(),
            style: Style::new(),
            justify: JustifyMethod::Left,
            vertical: VerticalAlign::Top,
            overflow: OverflowMethod::Fold,
            width: None,
            min_width: None,
            max_width: None,
            ratio: None,
            no_wrap: false,
        }
    }
}

impl Column {
    /// Create a new column with a header.
    #[must_use]
    pub fn new(header: impl Into<Text>) -> Self {
        Self {
            header: header.into(),
            ..Self::default()
        }
    }

    /// Set the footer.
    #[must_use]
    pub fn footer(mut self, footer: impl Into<Text>) -> Self {
        self.footer = footer.into();
        self
    }

    /// Set header style.
    #[must_use]
    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    /// Set footer style.
    #[must_use]
    pub fn footer_style(mut self, style: Style) -> Self {
        self.footer_style = style;
        self
    }

    /// Set cell style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set content justification.
    #[must_use]
    pub fn justify(mut self, justify: JustifyMethod) -> Self {
        self.justify = justify;
        self
    }

    /// Set vertical alignment.
    #[must_use]
    pub fn vertical(mut self, align: VerticalAlign) -> Self {
        self.vertical = align;
        self
    }

    /// Set fixed width.
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set minimum width.
    #[must_use]
    pub fn min_width(mut self, width: usize) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Set maximum width.
    #[must_use]
    pub fn max_width(mut self, width: usize) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set overflow handling.
    #[must_use]
    pub fn overflow(mut self, overflow: OverflowMethod) -> Self {
        self.overflow = overflow;
        self
    }

    /// Set flex ratio.
    #[must_use]
    pub fn ratio(mut self, ratio: usize) -> Self {
        self.ratio = Some(ratio);
        self
    }

    /// Disable text wrapping.
    #[must_use]
    pub fn no_wrap(mut self) -> Self {
        self.no_wrap = true;
        self
    }

    /// Get the header width.
    fn header_width(&self) -> usize {
        cells::cell_len(self.header.plain())
    }

    /// Get the footer width.
    fn footer_width(&self) -> usize {
        cells::cell_len(self.footer.plain())
    }
}

/// A table cell.
#[derive(Debug, Clone)]
pub struct Cell {
    /// Cell content.
    pub content: Text,
    /// Cell-specific style (overrides column style).
    pub style: Option<Style>,
}

impl Cell {
    /// Create a new cell.
    #[must_use]
    pub fn new(content: impl Into<Text>) -> Self {
        Self {
            content: content.into(),
            style: None,
        }
    }

    /// Set cell style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Get cell width.
    fn width(&self) -> usize {
        cells::cell_len(self.content.plain())
    }
}

impl<T: Into<Text>> From<T> for Cell {
    fn from(value: T) -> Self {
        Cell::new(value)
    }
}

/// A table row.
#[derive(Debug, Clone, Default)]
pub struct Row {
    /// Cells in this row.
    pub cells: Vec<Cell>,
    /// Row-level style.
    pub style: Style,
    /// Draw separator after this row.
    pub end_section: bool,
}

impl Row {
    /// Create a new row with cells.
    #[must_use]
    pub fn new(cells: Vec<Cell>) -> Self {
        Self {
            cells,
            ..Self::default()
        }
    }

    /// Set row style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Mark this row as ending a section.
    #[must_use]
    pub fn end_section(mut self) -> Self {
        self.end_section = true;
        self
    }
}

impl From<Vec<Cell>> for Row {
    fn from(cells: Vec<Cell>) -> Self {
        Self::new(cells)
    }
}

/// A data table with columns and rows.
#[derive(Debug, Clone)]
pub struct Table {
    /// Column definitions.
    columns: Vec<Column>,
    /// Data rows.
    rows: Vec<Row>,
    /// Table title.
    title: Option<Text>,
    /// Table caption (below).
    caption: Option<Text>,
    /// Fixed width.
    width: Option<usize>,
    /// Minimum width.
    min_width: Option<usize>,
    /// Box style.
    box_style: &'static BoxChars,
    /// Force ASCII boxes.
    safe_box: bool,
    /// Cell padding (horizontal, vertical).
    padding: (usize, usize),
    /// Collapse padding between cells.
    collapse_padding: bool,
    /// Pad outer edges.
    pad_edge: bool,
    /// Expand to fill width.
    expand: bool,
    /// Show header row.
    show_header: bool,
    /// Show footer row.
    show_footer: bool,
    /// Show left/right edges.
    show_edge: bool,
    /// Show lines between rows.
    show_lines: bool,
    /// Extra blank lines between rows.
    leading: usize,
    /// Table-level style.
    style: Style,
    /// Alternating row styles.
    row_styles: Vec<Style>,
    /// Header style.
    header_style: Style,
    /// Footer style.
    footer_style: Style,
    /// Border style.
    border_style: Style,
    /// Title style.
    title_style: Style,
    /// Caption style.
    caption_style: Style,
    /// Title justification.
    title_justify: JustifyMethod,
    /// Caption justification.
    caption_justify: JustifyMethod,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            title: None,
            caption: None,
            width: None,
            min_width: None,
            box_style: &SQUARE,
            safe_box: false,
            padding: (1, 0),
            collapse_padding: false,
            pad_edge: true,
            expand: false,
            show_header: true,
            show_footer: false,
            show_edge: true,
            show_lines: false,
            leading: 0,
            style: Style::new(),
            row_styles: Vec::new(),
            header_style: Style::new().bold(),
            footer_style: Style::new(),
            border_style: Style::new(),
            title_style: Style::new(),
            caption_style: Style::new(),
            title_justify: JustifyMethod::Center,
            caption_justify: JustifyMethod::Center,
        }
    }
}

impl Table {
    /// Create a new empty table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a column to the table.
    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    /// Add multiple columns to the table.
    pub fn add_columns(&mut self, columns: impl IntoIterator<Item = Column>) {
        self.columns.extend(columns);
    }

    /// Add a column (builder pattern).
    #[must_use]
    pub fn with_column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    /// Add multiple columns (builder pattern).
    #[must_use]
    pub fn with_columns(mut self, columns: impl IntoIterator<Item = Column>) -> Self {
        self.columns.extend(columns);
        self
    }

    /// Add a row to the table.
    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }

    /// Add multiple rows to the table.
    pub fn add_rows(&mut self, rows: impl IntoIterator<Item = Row>) {
        self.rows.extend(rows);
    }

    /// Add a row (builder pattern).
    #[must_use]
    pub fn with_row(mut self, row: Row) -> Self {
        self.rows.push(row);
        self
    }

    /// Add multiple rows (builder pattern).
    #[must_use]
    pub fn with_rows(mut self, rows: impl IntoIterator<Item = Row>) -> Self {
        self.rows.extend(rows);
        self
    }

    /// Add a row from cell values.
    pub fn add_row_cells<T: Into<Cell>>(&mut self, cells: impl IntoIterator<Item = T>) {
        let cells: Vec<Cell> = cells.into_iter().map(Into::into).collect();
        self.rows.push(Row::new(cells));
    }

    /// Add a row from cell values (builder pattern).
    #[must_use]
    pub fn with_row_cells<T: Into<Cell>>(mut self, cells: impl IntoIterator<Item = T>) -> Self {
        self.add_row_cells(cells);
        self
    }

    /// Set the title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Text>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the caption.
    #[must_use]
    pub fn caption(mut self, caption: impl Into<Text>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Set fixed width.
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set minimum width.
    #[must_use]
    pub fn min_width(mut self, width: usize) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Set the box style.
    #[must_use]
    pub fn box_style(mut self, style: &'static BoxChars) -> Self {
        self.box_style = style;
        self
    }

    /// Use ASCII boxes.
    #[must_use]
    pub fn ascii(mut self) -> Self {
        self.box_style = &ASCII;
        self.safe_box = true;
        self
    }

    /// Set safe box mode.
    #[must_use]
    pub fn safe_box(mut self, safe: bool) -> Self {
        self.safe_box = safe;
        self
    }

    /// Set cell padding.
    #[must_use]
    pub fn padding(mut self, horizontal: usize, vertical: usize) -> Self {
        self.padding = (horizontal, vertical);
        self
    }

    /// Collapse padding between cells.
    #[must_use]
    pub fn collapse_padding(mut self, collapse: bool) -> Self {
        self.collapse_padding = collapse;
        self
    }

    /// Set whether to pad outer edges.
    #[must_use]
    pub fn pad_edge(mut self, pad: bool) -> Self {
        self.pad_edge = pad;
        self
    }

    /// Set whether to expand to fill width.
    #[must_use]
    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    /// Set whether to show header.
    #[must_use]
    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Set whether to show footer.
    #[must_use]
    pub fn show_footer(mut self, show: bool) -> Self {
        self.show_footer = show;
        self
    }

    /// Set whether to show edges.
    #[must_use]
    pub fn show_edge(mut self, show: bool) -> Self {
        self.show_edge = show;
        self
    }

    /// Set whether to show lines between rows.
    #[must_use]
    pub fn show_lines(mut self, show: bool) -> Self {
        self.show_lines = show;
        self
    }

    /// Set the number of extra blank lines between rows.
    #[must_use]
    pub fn leading(mut self, leading: usize) -> Self {
        self.leading = leading;
        self
    }

    /// Set border style.
    #[must_use]
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Set title style.
    #[must_use]
    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Set caption style.
    #[must_use]
    pub fn caption_style(mut self, style: Style) -> Self {
        self.caption_style = style;
        self
    }

    /// Set title justification.
    #[must_use]
    pub fn title_justify(mut self, justify: JustifyMethod) -> Self {
        self.title_justify = justify;
        self
    }

    /// Set caption justification.
    #[must_use]
    pub fn caption_justify(mut self, justify: JustifyMethod) -> Self {
        self.caption_justify = justify;
        self
    }

    /// Set header style.
    #[must_use]
    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    /// Set alternating row styles.
    #[must_use]
    pub fn row_styles(mut self, styles: Vec<Style>) -> Self {
        self.row_styles = styles;
        self
    }

    /// Get the effective box characters.
    fn effective_box(&self) -> &'static BoxChars {
        if self.safe_box && !self.box_style.ascii {
            &ASCII
        } else {
            self.box_style
        }
    }

    /// Calculate column widths.
    fn calculate_widths(&self, max_width: usize) -> Vec<usize> {
        if self.columns.is_empty() {
            return Vec::new();
        }

        let num_cols = self.columns.len();

        // Calculate overhead (borders + padding)
        let border_width = if self.show_edge { 2 } else { 0 };
        let separator_width = if num_cols > 1 {
            if self.collapse_padding {
                num_cols - 1
            } else {
                (num_cols - 1) * (1 + self.padding.0 * 2)
            }
        } else {
            0
        };
        let edge_padding = if self.pad_edge { self.padding.0 * 2 } else { 0 };

        let overhead = border_width + separator_width + edge_padding;
        let available = max_width.saturating_sub(overhead);

        // Calculate natural widths for each column
        let mut widths: Vec<usize> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                // Get max width from header, footer, and all cells
                let mut max_w = col.header_width();
                max_w = max_w.max(col.footer_width());

                for row in &self.rows {
                    if let Some(cell) = row.cells.get(i) {
                        max_w = max_w.max(cell.width());
                    }
                }

                // Apply column constraints
                if let Some(fixed) = col.width {
                    return fixed;
                }

                let min_w = col.min_width.unwrap_or(1);
                let max_col_w = col.max_width.unwrap_or(usize::MAX);

                max_w.max(min_w).min(max_col_w)
            })
            .collect();

        // Calculate total and adjust if needed
        let total: usize = widths.iter().sum();

        if total > available {
            // Need to shrink columns
            widths = self.collapse_widths(&widths, available);
        } else if self.expand && total < available {
            // Expand to fill
            widths = self.expand_widths(&widths, available);
        }

        widths
    }

    /// Collapse column widths to fit available space.
    fn collapse_widths(&self, widths: &[usize], available: usize) -> Vec<usize> {
        let total: usize = widths.iter().sum();
        if total <= available {
            return widths.to_vec();
        }

        let mut result = widths.to_vec();
        let excess = total - available;

        // Get minimum widths
        let minimums: Vec<usize> = self
            .columns
            .iter()
            .map(|col| col.min_width.unwrap_or(1))
            .collect();

        // Calculate shrinkable amount per column
        let shrinkable: Vec<usize> = result
            .iter()
            .zip(minimums.iter())
            .map(|(w, m)| w.saturating_sub(*m))
            .collect();

        let total_shrinkable: usize = shrinkable.iter().sum();
        if total_shrinkable == 0 {
            return result;
        }

        // Shrink proportionally
        for (i, shrink) in shrinkable.iter().enumerate() {
            if *shrink > 0 {
                let reduction = *shrink * excess / total_shrinkable;
                result[i] = result[i].saturating_sub(reduction);
            }
        }

        // Handle rounding errors (RICH_SPEC Section 9.3, lines 1680-1694)
        let new_total: usize = result.iter().sum();
        if new_total > available {
            let mut diff = new_total - available;
            // Remove from columns in reverse order (largest first assumption)
            for i in (0..result.len()).rev() {
                if diff == 0 {
                    break;
                }
                if result[i] > minimums[i] {
                    let can_remove = (result[i] - minimums[i]).min(diff);
                    result[i] -= can_remove;
                    diff -= can_remove;
                }
            }
        }

        result
    }

    /// Expand column widths to fill available space.
    fn expand_widths(&self, widths: &[usize], available: usize) -> Vec<usize> {
        let total: usize = widths.iter().sum();
        if total >= available {
            return widths.to_vec();
        }

        let remaining = available - total;
        let mut sizes = widths.to_vec();

        let ratios: Vec<usize> = self
            .columns
            .iter()
            .zip(sizes.iter())
            .map(|(col, &size)| {
                let ratio = col.ratio.unwrap_or(0);
                if ratio > 0 && size < available {
                    ratio
                } else {
                    0
                }
            })
            .collect();

        let total_ratio: usize = ratios.iter().sum();
        if total_ratio == 0 {
            return sizes;
        }

        let flexible_count = ratios.iter().filter(|&&r| r > 0).count();
        let mut distributed = 0;
        let mut flex_idx = 0;

        for (i, &ratio) in ratios.iter().enumerate() {
            if ratio > 0 {
                flex_idx += 1;
                let share = Ratio::new(ratio, total_ratio);
                let extra = if flex_idx == flexible_count {
                    remaining - distributed
                } else {
                    (share * remaining).round().to_integer()
                };
                sizes[i] = sizes[i].saturating_add(extra);
                distributed += extra;
            }
        }

        sizes
    }

    /// Render the table to segments.
    #[must_use]
    pub fn render(&self, max_width: usize) -> Vec<Segment> {
        let box_chars = self.effective_box();
        let widths = self.calculate_widths(max_width);

        if widths.is_empty() {
            return Vec::new();
        }

        let mut segments = Vec::new();
        let has_body_rows = !self.rows.is_empty();
        let has_footer = self.show_footer && !self.columns.is_empty();

        // Title
        if let Some(title) = &self.title {
            let total_width = self.total_row_width(&widths);
            segments.extend(self.render_title_or_caption(
                title,
                total_width,
                &self.title_style,
                self.title_justify,
            ));
            segments.push(Segment::line());
        }

        // Top border
        if self.show_edge {
            let top = self.build_separator(box_chars, &widths, RowLevel::Top);
            segments.push(Segment::new(&top, Some(self.border_style.clone())));
            segments.push(Segment::line());
        }

        // Header
        if self.show_header && !self.columns.is_empty() {
            let header_cells: Vec<&Text> = self.columns.iter().map(|c| &c.header).collect();
            let header_styles: Vec<&Style> = self.columns.iter().map(|c| &c.header_style).collect();
            let header_overrides: Vec<Option<Style>> = vec![None; self.columns.len()];
            segments.extend(self.render_row_content(
                box_chars,
                &widths,
                &header_cells,
                &header_styles,
                &self.header_style,
                &header_overrides,
            ));

            if self.leading > 0 && (has_body_rows || has_footer) {
                segments.extend(self.render_leading_lines(
                    box_chars,
                    &widths,
                    &self.header_style,
                    self.leading,
                ));
            }

            // Header separator
            let sep = self.build_separator(box_chars, &widths, RowLevel::HeadRow);
            segments.push(Segment::new(&sep, Some(self.border_style.clone())));
            segments.push(Segment::line());
        }

        // Data rows
        for (row_idx, row) in self.rows.iter().enumerate() {
            let row_style = if self.row_styles.is_empty() {
                &row.style
            } else {
                &self.row_styles[row_idx % self.row_styles.len()]
            };

            // Pad cells to match column count
            let mut cells: Vec<Text> = Vec::with_capacity(self.columns.len());
            let mut overrides: Vec<Option<Style>> = Vec::with_capacity(self.columns.len());
            for i in 0..self.columns.len() {
                if let Some(cell) = row.cells.get(i) {
                    cells.push(cell.content.clone());
                    overrides.push(cell.style.clone());
                } else {
                    cells.push(Text::new(""));
                    overrides.push(None);
                }
            }
            let cell_refs: Vec<&Text> = cells.iter().collect();

            let col_styles: Vec<&Style> = self.columns.iter().map(|c| &c.style).collect();
            segments.extend(self.render_row_content(
                box_chars,
                &widths,
                &cell_refs,
                &col_styles,
                row_style,
                &overrides,
            ));

            let is_last = row_idx == self.rows.len() - 1;
            let has_next_row = row_idx + 1 < self.rows.len() || has_footer;

            // Leading blank lines between rows
            if self.leading > 0 && has_next_row {
                segments.extend(self.render_leading_lines(
                    box_chars,
                    &widths,
                    row_style,
                    self.leading,
                ));
            }

            // Row separator
            if (self.show_lines || row.end_section) && (!is_last || self.show_footer) {
                let sep = self.build_separator(box_chars, &widths, RowLevel::Row);
                segments.push(Segment::new(&sep, Some(self.border_style.clone())));
                segments.push(Segment::line());
            }
        }

        // Footer
        if self.show_footer && !self.columns.is_empty() {
            // Footer separator (if not already drawn)
            if !self.show_lines {
                let sep = self.build_separator(box_chars, &widths, RowLevel::FootRow);
                segments.push(Segment::new(&sep, Some(self.border_style.clone())));
                segments.push(Segment::line());
            }

            let footer_cells: Vec<&Text> = self.columns.iter().map(|c| &c.footer).collect();
            let footer_styles: Vec<&Style> = self.columns.iter().map(|c| &c.footer_style).collect();
            let footer_overrides: Vec<Option<Style>> = vec![None; self.columns.len()];
            segments.extend(self.render_row_content(
                box_chars,
                &widths,
                &footer_cells,
                &footer_styles,
                &self.footer_style,
                &footer_overrides,
            ));
        }

        // Bottom border
        if self.show_edge {
            let bottom = self.build_separator(box_chars, &widths, RowLevel::Bottom);
            segments.push(Segment::new(&bottom, Some(self.border_style.clone())));
            segments.push(Segment::line());
        }

        // Caption
        if let Some(caption) = &self.caption {
            let total_width = self.total_row_width(&widths);
            segments.extend(self.render_title_or_caption(
                caption,
                total_width,
                &self.caption_style,
                self.caption_justify,
            ));
            segments.push(Segment::line());
        }

        segments
    }

    /// Build a separator line.
    fn build_separator(&self, box_chars: &BoxChars, widths: &[usize], level: RowLevel) -> String {
        let chars = box_chars.get_row_chars(level);
        let left = chars[0];
        let mid = chars[1];
        let cross = chars[2];
        let right = chars[3];

        let mut result = String::new();

        if self.show_edge {
            result.push(left);
        }

        let last_idx = widths.len().saturating_sub(1);

        for (i, &width) in widths.iter().enumerate() {
            // Left padding
            let pad_left = if self.collapse_padding {
                self.pad_edge && i == 0
            } else {
                self.pad_edge || i > 0
            };
            if pad_left {
                for _ in 0..self.padding.0 {
                    result.push(mid);
                }
            }

            // Column content width
            for _ in 0..width {
                result.push(mid);
            }

            // Right padding
            let pad_right = if self.collapse_padding {
                self.pad_edge && i == last_idx
            } else {
                self.pad_edge || i < widths.len() - 1
            };
            if pad_right {
                for _ in 0..self.padding.0 {
                    result.push(mid);
                }
            }

            // Cross or right edge
            if i < widths.len() - 1 {
                result.push(cross);
            }
        }

        if self.show_edge {
            result.push(right);
        }

        result
    }

    /// Calculate total row width.
    fn total_row_width(&self, widths: &[usize]) -> usize {
        let content: usize = widths.iter().sum();
        let separators = if widths.len() > 1 {
            if self.collapse_padding {
                widths.len() - 1
            } else {
                (widths.len() - 1) * (1 + self.padding.0 * 2)
            }
        } else {
            0
        };
        let edge_padding = if self.pad_edge { self.padding.0 * 2 } else { 0 };
        let edges = if self.show_edge { 2 } else { 0 };
        content + separators + edge_padding + edges
    }

    /// Render a row's content.
    fn render_row_content(
        &self,
        box_chars: &BoxChars,
        widths: &[usize],
        cells: &[&Text],
        cell_styles: &[&Style],
        row_style: &Style,
        cell_overrides: &[Option<Style>],
    ) -> Vec<Segment> {
        let mut segments = Vec::new();
        let pad_str = " ".repeat(self.padding.0);
        let last_idx = widths.len().saturating_sub(1);

        // Left edge
        if self.show_edge {
            segments.push(Segment::new(
                box_chars.head[0].to_string(),
                Some(self.border_style.clone()),
            ));
        }

        for (i, (&width, &cell)) in widths.iter().zip(cells.iter()).enumerate() {
            let cell_style = cell_styles.get(i).copied().unwrap_or(&self.style);
            let override_style = cell_overrides.get(i).and_then(|style| style.as_ref());

            let mut combined_style = self.style.combine(row_style).combine(cell_style);
            if let Some(override_style) = override_style {
                combined_style = combined_style.combine(override_style);
            }
            combined_style = combined_style.combine(cell.style());

            // Left padding
            let pad_left = if self.collapse_padding {
                self.pad_edge && i == 0
            } else {
                self.pad_edge || i > 0
            };
            if pad_left {
                segments.push(Segment::new(&pad_str, Some(combined_style.clone())));
            }

            // Cell content
            let justify = self
                .columns
                .get(i)
                .map_or(JustifyMethod::Left, |c| c.justify);
            let overflow = self
                .columns
                .get(i)
                .map_or(OverflowMethod::Crop, |c| c.overflow);

            let mut cell_text = cell.clone();
            cell_text.set_style(combined_style.clone());

            let content_width = cell_text.cell_len();
            if content_width > width {
                cell_text.truncate(width, overflow, false);
            }

            let displayed_width = cell_text.cell_len();
            let space = width.saturating_sub(displayed_width);
            let (left_space, right_space) = match justify {
                JustifyMethod::Left | JustifyMethod::Default => (0, space),
                JustifyMethod::Right => (space, 0),
                JustifyMethod::Center => {
                    let left = space / 2;
                    (left, space - left)
                }
                JustifyMethod::Full => (0, space),
            };

            if left_space > 0 {
                segments.push(Segment::new(
                    " ".repeat(left_space),
                    Some(combined_style.clone()),
                ));
            }

            segments.extend(cell_text.render(""));

            if right_space > 0 {
                segments.push(Segment::new(
                    " ".repeat(right_space),
                    Some(combined_style.clone()),
                ));
            }

            // Right padding
            let pad_right = if self.collapse_padding {
                self.pad_edge && i == last_idx
            } else {
                self.pad_edge || i < widths.len() - 1
            };
            if pad_right {
                segments.push(Segment::new(&pad_str, Some(combined_style)));
            }

            // Cell divider
            if i < widths.len() - 1 {
                segments.push(Segment::new(
                    box_chars.head[2].to_string(),
                    Some(self.border_style.clone()),
                ));
            }
        }

        // Right edge
        if self.show_edge {
            segments.push(Segment::new(
                box_chars.head[3].to_string(),
                Some(self.border_style.clone()),
            ));
        }

        segments.push(Segment::line());
        segments
    }

    /// Render multiple leading blank lines between rows.
    fn render_leading_lines(
        &self,
        box_chars: &BoxChars,
        widths: &[usize],
        row_style: &Style,
        count: usize,
    ) -> Vec<Segment> {
        if count == 0 {
            return Vec::new();
        }

        let empty_cells: Vec<Text> = (0..widths.len()).map(|_| Text::new("")).collect();
        let cell_refs: Vec<&Text> = empty_cells.iter().collect();
        let cell_styles: Vec<&Style> = self.columns.iter().map(|c| &c.style).collect();
        let overrides: Vec<Option<Style>> = vec![None; self.columns.len()];

        let mut segments = Vec::new();
        for _ in 0..count {
            segments.extend(self.render_row_content(
                box_chars,
                widths,
                &cell_refs,
                &cell_styles,
                row_style,
                &overrides,
            ));
        }
        segments
    }

    /// Render title or caption.
    fn render_title_or_caption(
        &self,
        text: &Text,
        width: usize,
        style: &Style,
        justify: JustifyMethod,
    ) -> Vec<Segment> {
        if width == 0 {
            return Vec::new();
        }

        let mut content_text = text.clone();
        if content_text.cell_len() > width {
            content_text.truncate(width, OverflowMethod::Crop, false);
        }
        let content_width = content_text.cell_len();
        let space = width.saturating_sub(content_width);

        let (left_space, right_space) = match justify {
            JustifyMethod::Left | JustifyMethod::Default => (0, space),
            JustifyMethod::Right => (space, 0),
            JustifyMethod::Center | JustifyMethod::Full => {
                let left = space / 2;
                (left, space - left)
            }
        };

        let mut segments = Vec::new();

        if left_space > 0 {
            segments.push(Segment::new(" ".repeat(left_space), Some(style.clone())));
        }

        let mut content_segments = content_text.render("");
        for segment in &mut content_segments {
            if !segment.is_control() {
                segment.style = Some(match segment.style.take() {
                    Some(existing) => style.combine(&existing),
                    None => style.clone(),
                });
            }
        }
        let mut remaining = width;
        let mut trimmed_segments = Vec::new();
        for segment in content_segments {
            if segment.is_control() {
                trimmed_segments.push(segment);
                continue;
            }

            if remaining == 0 {
                break;
            }

            let seg_width = segment.cell_length();
            if seg_width <= remaining {
                remaining = remaining.saturating_sub(seg_width);
                trimmed_segments.push(segment);
            } else {
                let (left, _right) = segment.split_at_cell(remaining);
                if !left.is_empty() {
                    trimmed_segments.push(left);
                }
                break;
            }
        }

        segments.extend(trimmed_segments);

        if right_space > 0 {
            segments.push(Segment::new(" ".repeat(right_space), Some(style.clone())));
        }

        segments
    }

    /// Render to plain text.
    #[must_use]
    pub fn render_plain(&self, max_width: usize) -> String {
        self.render(max_width)
            .into_iter()
            .map(|seg| seg.text)
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::similar_names)]
mod tests {
    use super::*;
    use crate::cells::cell_len;
    use crate::color::Color;
    use crate::style::Attributes;

    #[test]
    fn test_column_new() {
        let col = Column::new("Name");
        assert_eq!(col.header.plain(), "Name");
    }

    #[test]
    fn test_table_new() {
        let table = Table::new();
        assert!(table.columns.is_empty());
        assert!(table.rows.is_empty());
    }

    #[test]
    fn test_table_with_columns() {
        let table = Table::new()
            .with_column(Column::new("Name"))
            .with_column(Column::new("Age"));
        assert_eq!(table.columns.len(), 2);
    }

    #[test]
    fn test_table_add_row() {
        let mut table = Table::new()
            .with_column(Column::new("Name"))
            .with_column(Column::new("Age"));

        table.add_row_cells(["Alice", "30"]);
        table.add_row_cells(["Bob", "25"]);

        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn test_table_render() {
        let mut table = Table::new()
            .with_column(Column::new("Name"))
            .with_column(Column::new("Age"));

        table.add_row_cells(["Alice", "30"]);

        let segments = table.render(40);
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();

        assert!(text.contains("Name"));
        assert!(text.contains("Age"));
        assert!(text.contains("Alice"));
        assert!(text.contains("30"));
    }

    #[test]
    fn test_table_ascii() {
        let mut table = Table::new().with_column(Column::new("X")).ascii();

        table.add_row_cells(["1"]);

        let text = table.render_plain(20);
        assert!(text.contains('+')); // ASCII corners
        assert!(text.contains('-')); // ASCII horizontal
    }

    #[test]
    fn test_table_no_header() {
        let mut table = Table::new()
            .with_column(Column::new("Name"))
            .show_header(false);

        table.add_row_cells(["Alice"]);

        let text = table.render_plain(30);
        assert!(!text.contains("Name")); // Header hidden
        assert!(text.contains("Alice"));
    }

    #[test]
    fn test_table_with_title() {
        let mut table = Table::new()
            .with_column(Column::new("X").width(10))
            .title("My Table");

        table.add_row_cells(["1"]);

        let text = table.render_plain(30);
        assert!(text.contains("My Table"));
    }

    #[test]
    fn test_table_title_preserves_spans_and_style() {
        use crate::style::Attributes;

        let mut title = Text::new("Title");
        title.stylize(0, 5, Style::new().bold());

        let red = Style::new().color(crate::color::Color::parse("red").unwrap());
        let mut table = Table::new()
            .with_column(Column::new("X"))
            .title(title)
            .title_style(red);

        table.add_row_cells(["1"]);

        let segments = table.render(30);
        let has_styled_title = segments.iter().any(|seg| {
            seg.text.contains("Title")
                && seg
                    .style
                    .as_ref()
                    .is_some_and(|style| style.color.is_some())
                && seg
                    .style
                    .as_ref()
                    .is_some_and(|style| style.attributes.contains(Attributes::BOLD))
        });

        assert!(has_styled_title);
    }

    #[test]
    fn test_caption_alignment_preserves_line_width() {
        let justifies = [
            JustifyMethod::Left,
            JustifyMethod::Center,
            JustifyMethod::Right,
        ];

        for justify in justifies {
            let mut table = Table::new()
                .with_column(Column::new("Col").width(6))
                .caption("A very long caption")
                .caption_justify(justify);
            table.add_row_cells(["Value"]);

            let output = table.render_plain(40);
            let lines: Vec<&str> = output.lines().collect();
            assert!(lines.len() >= 2, "Expected at least border + caption");

            let caption_line = lines.last().expect("caption line");
            let border_line = lines.iter().rev().nth(1).expect("bottom border line");

            assert_eq!(
                cell_len(caption_line),
                cell_len(border_line),
                "caption width mismatch for {justify:?}"
            );
        }
    }

    #[test]
    fn test_calculate_widths() {
        let mut table = Table::new()
            .with_column(Column::new("Name"))
            .with_column(Column::new("Age"));

        table.add_row_cells(["Alice", "30"]);

        let widths = table.calculate_widths(50);
        assert_eq!(widths.len(), 2);
        assert!(widths[0] >= 4); // "Name" or "Alice"
        assert!(widths[1] >= 2); // "30"
    }

    #[test]
    fn test_column_constraints() {
        let table = Table::new()
            .with_column(Column::new("X").width(10))
            .with_column(Column::new("Y").min_width(5));

        let widths = table.calculate_widths(50);
        assert_eq!(widths[0], 10);
        assert!(widths[1] >= 5);
    }

    #[test]
    fn test_vertical_align() {
        let col = Column::new("Test").vertical(VerticalAlign::Middle);
        assert_eq!(col.vertical, VerticalAlign::Middle);
    }

    #[test]
    fn test_cell_from_string() {
        let cell: Cell = "Hello".into();
        assert_eq!(cell.content.plain(), "Hello");
    }

    #[test]
    fn test_row_end_section() {
        let row = Row::new(vec![Cell::new("X")]).end_section();
        assert!(row.end_section);
    }

    #[test]
    fn test_table_cell_style_applies_with_column_style() {
        let red = Style::new().color(Color::parse("red").unwrap());
        let bold = Style::new().bold();

        let mut table = Table::new().with_column(Column::new("Col").style(red.clone()));
        table.add_row(Row::new(vec![Cell::new("X").style(bold.clone())]));

        let segments = table.render(20);
        let cell_seg = segments
            .iter()
            .find(|seg| seg.text.contains('X'))
            .expect("expected cell content segment");

        let style = cell_seg.style.as_ref().expect("expected styled segment");
        assert!(style.attributes.contains(Attributes::BOLD));
        assert_eq!(style.color, red.color);
    }

    #[test]
    fn test_table_preserves_text_spans() {
        let mut text = Text::new("ab");
        text.stylize(0, 1, Style::new().italic());

        let mut table = Table::new().with_column(Column::new("Col"));
        table.add_row(Row::new(vec![Cell::new(text)]));

        let segments = table.render(20);
        let styled_seg = segments
            .iter()
            .find(|seg| seg.text.contains('a'))
            .expect("expected styled text segment");

        let style = styled_seg.style.as_ref().expect("expected styled segment");
        assert!(style.attributes.contains(Attributes::ITALIC));
    }

    #[test]
    fn test_table_leading_adds_blank_lines() {
        let mut table = Table::new().with_column(Column::new("X")).leading(2);

        table.add_row_cells(["A"]);
        table.add_row_cells(["B"]);

        let text = table.render_plain(20);
        let lines: Vec<&str> = text.lines().collect();

        // With leading=2, we expect:
        // 1. Top border
        // 2. Header row
        // 3. Header separator
        // 4. Row A
        // 5-6. Two blank lines (leading)
        // 7. Row B
        // 8. Bottom border
        assert!(
            lines.len() >= 8,
            "expected at least 8 lines with leading=2, got {}",
            lines.len()
        );

        // Check that there are blank content lines between A and B
        let line_with_a = lines.iter().position(|l| l.contains('A')).expect("row A");
        let line_with_b = lines.iter().position(|l| l.contains('B')).expect("row B");

        // There should be 2 blank lines between row A and row B
        assert_eq!(
            line_with_b - line_with_a - 1,
            2,
            "expected 2 blank lines between rows A and B"
        );
    }

    #[test]
    fn test_table_leading_with_show_lines() {
        let mut table = Table::new()
            .with_column(Column::new("X"))
            .leading(1)
            .show_lines(true);

        table.add_row_cells(["A"]);
        table.add_row_cells(["B"]);

        let text = table.render_plain(20);
        let lines: Vec<&str> = text.lines().collect();

        // With leading=1 and show_lines=true:
        // 1. Top border
        // 2. Header row
        // 3. Header separator
        // 4. Row A
        // 5. Blank line (leading)
        // 6. Row separator
        // 7. Row B
        // 8. Bottom border

        let line_with_a = lines.iter().position(|l| l.contains('A')).expect("row A");
        let line_with_b = lines.iter().position(|l| l.contains('B')).expect("row B");

        // There should be 2 lines between row A and row B (1 leading + 1 separator)
        assert_eq!(
            line_with_b - line_with_a - 1,
            2,
            "expected 1 leading line + 1 separator between rows"
        );
    }

    #[test]
    fn test_table_leading_zero_no_extra_lines() {
        let mut table = Table::new().with_column(Column::new("X")).leading(0);

        table.add_row_cells(["A"]);
        table.add_row_cells(["B"]);

        let text = table.render_plain(20);
        let lines: Vec<&str> = text.lines().collect();

        let line_with_a = lines.iter().position(|l| l.contains('A')).expect("row A");
        let line_with_b = lines.iter().position(|l| l.contains('B')).expect("row B");

        // With leading=0, rows should be adjacent
        assert_eq!(
            line_with_b - line_with_a - 1,
            0,
            "expected no blank lines between rows with leading=0"
        );
    }

    #[test]
    fn test_table_leading_preserves_border_structure() {
        let mut table = Table::new()
            .with_column(Column::new("Col").width(5))
            .leading(1)
            .ascii();

        table.add_row_cells(["A"]);
        table.add_row_cells(["B"]);

        let text = table.render_plain(20);
        let lines: Vec<&str> = text.lines().collect();

        // Find the blank leading line (between row A and row B)
        let row_a_idx = lines.iter().position(|l| l.contains('A')).expect("row A");
        let blank_line = lines[row_a_idx + 1];

        // Blank line should have proper border characters
        assert!(
            blank_line.starts_with('|') && blank_line.ends_with('|'),
            "blank leading line should have borders: {blank_line}"
        );
    }
}
