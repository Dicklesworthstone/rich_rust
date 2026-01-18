//! Box drawing characters for tables and panels.
//!
//! This module provides box drawing character sets for creating
//! bordered tables and panels in the terminal.

use std::fmt;

/// Row level for box drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowLevel {
    /// Top of the box.
    Top,
    /// Header row separator.
    HeadRow,
    /// Middle row separator.
    Mid,
    /// Regular row separator.
    Row,
    /// Footer row separator.
    FootRow,
    /// Bottom of the box.
    Bottom,
}

/// Box drawing character set.
///
/// Each row is 4 characters: [left, middle, cross/divider, right]
/// - left: leftmost edge character
/// - middle: horizontal line character
/// - cross: intersection or divider character
/// - right: rightmost edge character
#[derive(Debug, Clone)]
pub struct BoxChars {
    /// Top row: ┌─┬┐
    pub top: [char; 4],
    /// Head row (cell content): │ ││
    pub head: [char; 4],
    /// Head separator: ├─┼┤
    pub head_row: [char; 4],
    /// Mid separator: ├─┼┤
    pub mid: [char; 4],
    /// Row separator: ├─┼┤
    pub row: [char; 4],
    /// Foot separator: ├─┼┤
    pub foot_row: [char; 4],
    /// Foot row (cell content): │ ││
    pub foot: [char; 4],
    /// Bottom row: └─┴┘
    pub bottom: [char; 4],
    /// Whether this box uses ASCII-only characters.
    pub ascii: bool,
}

impl BoxChars {
    /// Create a new box from character arrays.
    #[must_use]
    pub const fn new(
        top: [char; 4],
        head: [char; 4],
        head_row: [char; 4],
        mid: [char; 4],
        row: [char; 4],
        foot_row: [char; 4],
        foot: [char; 4],
        bottom: [char; 4],
        ascii: bool,
    ) -> Self {
        Self {
            top,
            head,
            head_row,
            mid,
            row,
            foot_row,
            foot,
            bottom,
            ascii,
        }
    }

    /// Get the row characters for a specific level.
    #[must_use]
    pub fn get_row_chars(&self, level: RowLevel) -> &[char; 4] {
        match level {
            RowLevel::Top => &self.top,
            RowLevel::HeadRow => &self.head_row,
            RowLevel::Mid => &self.mid,
            RowLevel::Row => &self.row,
            RowLevel::FootRow => &self.foot_row,
            RowLevel::Bottom => &self.bottom,
        }
    }

    /// Build a row string for the given column widths.
    #[must_use]
    pub fn build_row(&self, widths: &[usize], level: RowLevel, edge: bool) -> String {
        let chars = self.get_row_chars(level);
        let left = chars[0];
        let middle = chars[1];
        let cross = chars[2];
        let right = chars[3];

        let mut result = String::new();

        if edge && left != ' ' {
            result.push(left);
        }

        for (i, &width) in widths.iter().enumerate() {
            // Add horizontal line for this column
            for _ in 0..width {
                result.push(middle);
            }

            // Add cross or right edge
            if i < widths.len() - 1 {
                result.push(cross);
            } else if edge && right != ' ' {
                result.push(right);
            }
        }

        result
    }

    /// Build the top border.
    #[must_use]
    pub fn get_top(&self, widths: &[usize]) -> String {
        self.build_row(widths, RowLevel::Top, true)
    }

    /// Build the bottom border.
    #[must_use]
    pub fn get_bottom(&self, widths: &[usize]) -> String {
        self.build_row(widths, RowLevel::Bottom, true)
    }

    /// Build the header separator.
    #[must_use]
    pub fn get_head_row(&self, widths: &[usize]) -> String {
        self.build_row(widths, RowLevel::HeadRow, true)
    }

    /// Build a mid-table separator.
    #[must_use]
    pub fn get_mid(&self, widths: &[usize]) -> String {
        self.build_row(widths, RowLevel::Mid, true)
    }

    /// Build a regular row separator.
    #[must_use]
    pub fn get_row(&self, widths: &[usize]) -> String {
        self.build_row(widths, RowLevel::Row, true)
    }

    /// Get the cell left edge character.
    #[must_use]
    pub fn cell_left(&self) -> char {
        self.head[0]
    }

    /// Get the cell divider character.
    #[must_use]
    pub fn cell_divider(&self) -> char {
        self.head[2]
    }

    /// Get the cell right edge character.
    #[must_use]
    pub fn cell_right(&self) -> char {
        self.head[3]
    }

    /// Substitute characters for ASCII-safe rendering.
    #[must_use]
    pub fn substitute(&self, safe: bool) -> &Self {
        // For now, return self; implement ASCII substitution if needed
        if safe && !self.ascii {
            // Could return ASCII equivalent here
            self
        } else {
            self
        }
    }
}

impl fmt::Display for BoxChars {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display a sample 3x3 box
        let widths = [3, 3, 3];
        writeln!(f, "{}", self.get_top(&widths))?;
        writeln!(
            f,
            "{}   {}   {}   {}",
            self.head[0], self.head[2], self.head[2], self.head[3]
        )?;
        writeln!(f, "{}", self.get_head_row(&widths))?;
        writeln!(
            f,
            "{}   {}   {}   {}",
            self.head[0], self.head[2], self.head[2], self.head[3]
        )?;
        write!(f, "{}", self.get_bottom(&widths))
    }
}

// ============================================================================
// Built-in Box Styles
// ============================================================================

/// ASCII box (safe for all terminals).
pub const ASCII: BoxChars = BoxChars::new(
    ['+', '-', '+', '+'],
    ['|', ' ', '|', '|'],
    ['|', '-', '+', '|'],
    ['|', '-', '+', '|'],
    ['|', '-', '+', '|'],
    ['|', '-', '+', '|'],
    ['|', ' ', '|', '|'],
    ['+', '-', '+', '+'],
    true,
);

/// ASCII2 box with double lines at intersections.
pub const ASCII2: BoxChars = BoxChars::new(
    ['+', '-', '+', '+'],
    ['|', ' ', '|', '|'],
    ['+', '-', '+', '+'],
    ['+', '-', '+', '+'],
    ['+', '-', '+', '+'],
    ['+', '-', '+', '+'],
    ['|', ' ', '|', '|'],
    ['+', '-', '+', '+'],
    true,
);

/// ASCII with double header.
pub const ASCII_DOUBLE_HEAD: BoxChars = BoxChars::new(
    ['+', '-', '+', '+'],
    ['|', ' ', '|', '|'],
    ['+', '=', '+', '+'],
    ['|', '-', '+', '|'],
    ['|', '-', '+', '|'],
    ['|', '-', '+', '|'],
    ['|', ' ', '|', '|'],
    ['+', '-', '+', '+'],
    true,
);

/// Unicode rounded box.
pub const ROUNDED: BoxChars = BoxChars::new(
    ['\u{256D}', '\u{2500}', '\u{252C}', '\u{256E}'], // ╭─┬╮
    ['\u{2502}', ' ', '\u{2502}', '\u{2502}'],        // │ ││
    ['\u{251C}', '\u{2500}', '\u{253C}', '\u{2524}'], // ├─┼┤
    ['\u{251C}', '\u{2500}', '\u{253C}', '\u{2524}'], // ├─┼┤
    ['\u{251C}', '\u{2500}', '\u{253C}', '\u{2524}'], // ├─┼┤
    ['\u{251C}', '\u{2500}', '\u{253C}', '\u{2524}'], // ├─┼┤
    ['\u{2502}', ' ', '\u{2502}', '\u{2502}'],        // │ ││
    ['\u{2570}', '\u{2500}', '\u{2534}', '\u{256F}'], // ╰─┴╯
    false,
);

/// Unicode square/single line box.
pub const SQUARE: BoxChars = BoxChars::new(
    ['\u{250C}', '\u{2500}', '\u{252C}', '\u{2510}'], // ┌─┬┐
    ['\u{2502}', ' ', '\u{2502}', '\u{2502}'],        // │ ││
    ['\u{251C}', '\u{2500}', '\u{253C}', '\u{2524}'], // ├─┼┤
    ['\u{251C}', '\u{2500}', '\u{253C}', '\u{2524}'], // ├─┼┤
    ['\u{251C}', '\u{2500}', '\u{253C}', '\u{2524}'], // ├─┼┤
    ['\u{251C}', '\u{2500}', '\u{253C}', '\u{2524}'], // ├─┼┤
    ['\u{2502}', ' ', '\u{2502}', '\u{2502}'],        // │ ││
    ['\u{2514}', '\u{2500}', '\u{2534}', '\u{2518}'], // └─┴┘
    false,
);

/// Unicode double line box.
pub const DOUBLE: BoxChars = BoxChars::new(
    ['\u{2554}', '\u{2550}', '\u{2566}', '\u{2557}'], // ╔═╦╗
    ['\u{2551}', ' ', '\u{2551}', '\u{2551}'],        // ║ ║║
    ['\u{2560}', '\u{2550}', '\u{256C}', '\u{2563}'], // ╠═╬╣
    ['\u{2560}', '\u{2550}', '\u{256C}', '\u{2563}'], // ╠═╬╣
    ['\u{2560}', '\u{2550}', '\u{256C}', '\u{2563}'], // ╠═╬╣
    ['\u{2560}', '\u{2550}', '\u{256C}', '\u{2563}'], // ╠═╬╣
    ['\u{2551}', ' ', '\u{2551}', '\u{2551}'],        // ║ ║║
    ['\u{255A}', '\u{2550}', '\u{2569}', '\u{255D}'], // ╚═╩╝
    false,
);

/// Heavy (thick) line box.
pub const HEAVY: BoxChars = BoxChars::new(
    ['\u{250F}', '\u{2501}', '\u{2533}', '\u{2513}'], // ┏━┳┓
    ['\u{2503}', ' ', '\u{2503}', '\u{2503}'],        // ┃ ┃┃
    ['\u{2523}', '\u{2501}', '\u{254B}', '\u{252B}'], // ┣━╋┫
    ['\u{2523}', '\u{2501}', '\u{254B}', '\u{252B}'], // ┣━╋┫
    ['\u{2523}', '\u{2501}', '\u{254B}', '\u{252B}'], // ┣━╋┫
    ['\u{2523}', '\u{2501}', '\u{254B}', '\u{252B}'], // ┣━╋┫
    ['\u{2503}', ' ', '\u{2503}', '\u{2503}'],        // ┃ ┃┃
    ['\u{2517}', '\u{2501}', '\u{253B}', '\u{251B}'], // ┗━┻┛
    false,
);

/// Heavy head with single body.
pub const HEAVY_HEAD: BoxChars = BoxChars::new(
    ['\u{250F}', '\u{2501}', '\u{2533}', '\u{2513}'], // ┏━┳┓
    ['\u{2503}', ' ', '\u{2503}', '\u{2503}'],        // ┃ ┃┃
    ['\u{2521}', '\u{2501}', '\u{2547}', '\u{2529}'], // ┡━╇┩
    ['\u{251C}', '\u{2500}', '\u{253C}', '\u{2524}'], // ├─┼┤
    ['\u{251C}', '\u{2500}', '\u{253C}', '\u{2524}'], // ├─┼┤
    ['\u{251C}', '\u{2500}', '\u{253C}', '\u{2524}'], // ├─┼┤
    ['\u{2502}', ' ', '\u{2502}', '\u{2502}'],        // │ ││
    ['\u{2514}', '\u{2500}', '\u{2534}', '\u{2518}'], // └─┴┘
    false,
);

/// Minimal (no outer border).
pub const MINIMAL: BoxChars = BoxChars::new(
    [' ', ' ', ' ', ' '],
    [' ', ' ', '\u{2502}', ' '],        //   │
    [' ', '\u{2500}', '\u{253C}', ' '], //  ─┼
    [' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' '],
    [' ', ' ', '\u{2502}', ' '], //   │
    [' ', ' ', ' ', ' '],
    false,
);

/// Simple (just horizontal lines).
pub const SIMPLE: BoxChars = BoxChars::new(
    [' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' '],
    [' ', '\u{2500}', '\u{2500}', ' '], //  ──
    [' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' '],
    [' ', '\u{2500}', '\u{2500}', ' '], //  ──
    [' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' '],
    false,
);

/// Simple heavy (just thick horizontal lines).
pub const SIMPLE_HEAVY: BoxChars = BoxChars::new(
    [' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' '],
    [' ', '\u{2501}', '\u{2501}', ' '], //  ━━
    [' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' '],
    [' ', '\u{2501}', '\u{2501}', ' '], //  ━━
    [' ', ' ', ' ', ' '],
    [' ', ' ', ' ', ' '],
    false,
);

/// Get a box style by name.
#[must_use]
pub fn get_box(name: &str) -> Option<&'static BoxChars> {
    match name.to_lowercase().as_str() {
        "ascii" => Some(&ASCII),
        "ascii2" => Some(&ASCII2),
        "ascii_double_head" => Some(&ASCII_DOUBLE_HEAD),
        "rounded" => Some(&ROUNDED),
        "square" => Some(&SQUARE),
        "double" => Some(&DOUBLE),
        "heavy" => Some(&HEAVY),
        "heavy_head" => Some(&HEAVY_HEAD),
        "minimal" => Some(&MINIMAL),
        "simple" => Some(&SIMPLE),
        "simple_heavy" => Some(&SIMPLE_HEAVY),
        _ => None,
    }
}

/// Get an ASCII-safe version of a box style.
#[must_use]
pub fn get_safe_box(name: &str) -> &'static BoxChars {
    let box_style = get_box(name).unwrap_or(&SQUARE);
    if box_style.ascii { box_style } else { &ASCII }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_box() {
        assert!(ASCII.ascii);
        assert_eq!(ASCII.top[0], '+');
    }

    #[test]
    fn test_get_top() {
        let widths = [5, 3, 7];
        let top = ASCII.get_top(&widths);
        assert_eq!(top, "+-----+---+-------+");
    }

    #[test]
    fn test_get_bottom() {
        let widths = [5, 3];
        let bottom = ASCII.get_bottom(&widths);
        assert_eq!(bottom, "+-----+---+");
    }

    #[test]
    fn test_unicode_square() {
        assert!(!SQUARE.ascii);
        assert_eq!(SQUARE.top[0], '\u{250C}'); // ┌
    }

    #[test]
    fn test_get_box() {
        assert!(get_box("ascii").is_some());
        assert!(get_box("SQUARE").is_some()); // Case insensitive
        assert!(get_box("nonexistent").is_none());
    }

    #[test]
    fn test_get_safe_box() {
        let safe = get_safe_box("double");
        assert!(safe.ascii); // Should return ASCII for non-ASCII box
    }

    #[test]
    fn test_build_row_widths() {
        let widths = [4, 4];
        let row = SQUARE.build_row(&widths, RowLevel::HeadRow, true);
        assert!(row.len() > 0);
        assert!(row.contains('\u{253C}')); // ┼
    }

    #[test]
    fn test_cell_characters() {
        assert_eq!(ASCII.cell_left(), '|');
        assert_eq!(ASCII.cell_divider(), '|');
        assert_eq!(ASCII.cell_right(), '|');
    }

    #[test]
    fn test_rounded_box() {
        assert!(!ROUNDED.ascii);
        assert_eq!(ROUNDED.top[0], '\u{256D}'); // ╭
        assert_eq!(ROUNDED.bottom[0], '\u{2570}'); // ╰
    }
}
