//! Panel - bordered box containing content.
//!
//! A Panel renders content inside a decorative border with optional
//! title and subtitle.

use crate::r#box::{ASCII, BoxChars, ROUNDED, SQUARE};
use crate::cells;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::{JustifyMethod, Text};

use super::padding::PaddingDimensions;

/// A bordered panel containing content.
#[derive(Debug, Clone)]
pub struct Panel {
    /// Content lines to render inside the panel.
    content_lines: Vec<Vec<Segment>>,
    /// Box drawing style.
    box_style: &'static BoxChars,
    /// Use ASCII-safe characters.
    safe_box: bool,
    /// Expand to fill available width.
    expand: bool,
    /// Style for the panel background.
    style: Style,
    /// Style for the border.
    border_style: Style,
    /// Fixed width (None = auto).
    width: Option<usize>,
    /// Fixed height (None = auto).
    height: Option<usize>,
    /// Padding inside the border.
    padding: PaddingDimensions,
    /// Optional title.
    title: Option<Text>,
    /// Title alignment.
    title_align: JustifyMethod,
    /// Optional subtitle (bottom).
    subtitle: Option<Text>,
    /// Subtitle alignment.
    subtitle_align: JustifyMethod,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            content_lines: Vec::new(),
            box_style: &ROUNDED,
            safe_box: false,
            expand: true,
            style: Style::new(),
            border_style: Style::new(),
            width: None,
            height: None,
            padding: PaddingDimensions::symmetric(0, 1),
            title: None,
            title_align: JustifyMethod::Center,
            subtitle: None,
            subtitle_align: JustifyMethod::Center,
        }
    }
}

impl Panel {
    /// Create a new panel with content lines.
    #[must_use]
    pub fn new(content_lines: Vec<Vec<Segment>>) -> Self {
        Self {
            content_lines,
            ..Self::default()
        }
    }

    /// Create a panel from plain text content.
    #[must_use]
    pub fn from_text(text: &str) -> Self {
        let lines: Vec<Vec<Segment>> = text
            .lines()
            .map(|line| vec![Segment::new(line, None)])
            .collect();
        Self::new(lines)
    }

    /// Create a panel from a Text object.
    #[must_use]
    pub fn from_rich_text(text: &Text, width: usize) -> Self {
        // Render text to lines
        let segments = text.render("");
        let mut lines = Vec::new();
        let mut current_line = Vec::new();

        for segment in segments {
            if segment.is_control() && segment.text == "\n" {
                lines.push(std::mem::take(&mut current_line));
            } else {
                current_line.push(segment);
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        Self {
            content_lines: lines,
            width: Some(width),
            ..Self::default()
        }
    }

    /// Set the box style.
    #[must_use]
    pub fn box_style(mut self, style: &'static BoxChars) -> Self {
        self.box_style = style;
        self
    }

    /// Use rounded box style.
    #[must_use]
    pub fn rounded(mut self) -> Self {
        self.box_style = &ROUNDED;
        self
    }

    /// Use square box style.
    #[must_use]
    pub fn square(mut self) -> Self {
        self.box_style = &SQUARE;
        self
    }

    /// Use ASCII-safe box style.
    #[must_use]
    pub fn ascii(mut self) -> Self {
        self.box_style = &ASCII;
        self.safe_box = true;
        self
    }

    /// Force ASCII-safe rendering.
    #[must_use]
    pub fn safe_box(mut self, safe: bool) -> Self {
        self.safe_box = safe;
        self
    }

    /// Set whether to expand to fill width.
    #[must_use]
    pub fn expand(mut self, expand: bool) -> Self {
        self.expand = expand;
        self
    }

    /// Set the background style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Set fixed width.
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set fixed height.
    #[must_use]
    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    /// Set padding.
    #[must_use]
    pub fn padding(mut self, padding: impl Into<PaddingDimensions>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Set the title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Text>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set title alignment.
    #[must_use]
    pub fn title_align(mut self, align: JustifyMethod) -> Self {
        self.title_align = align;
        self
    }

    /// Set the subtitle.
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<Text>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set subtitle alignment.
    #[must_use]
    pub fn subtitle_align(mut self, align: JustifyMethod) -> Self {
        self.subtitle_align = align;
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

    /// Calculate content width from content lines.
    fn content_width(&self) -> usize {
        self.content_lines
            .iter()
            .map(|line| line.iter().map(|seg| cells::cell_len(&seg.text)).sum())
            .max()
            .unwrap_or(0)
    }

    /// Render the panel to segments.
    #[must_use]
    pub fn render(&self, max_width: usize) -> Vec<Segment> {
        let box_chars = self.effective_box();

        // Calculate panel width
        let panel_width = if self.expand {
            self.width.unwrap_or(max_width)
        } else {
            let content_w = self.content_width();
            let min_width = content_w + 2 + self.padding.horizontal();
            self.width.unwrap_or(min_width).min(max_width)
        };

        // Inner width (inside borders)
        let inner_width = panel_width.saturating_sub(2);
        // Content width (inside borders and padding)
        let content_width = inner_width.saturating_sub(self.padding.horizontal());

        let mut segments = Vec::new();

        // Top border with optional title
        segments.extend(self.render_top_border(box_chars, inner_width));
        segments.push(Segment::line());

        // Top padding
        for _ in 0..self.padding.top {
            segments.push(Segment::new(
                box_chars.head[0].to_string(),
                Some(self.border_style.clone()),
            ));
            segments.push(Segment::new(
                " ".repeat(inner_width),
                Some(self.style.clone()),
            ));
            segments.push(Segment::new(
                box_chars.head[3].to_string(),
                Some(self.border_style.clone()),
            ));
            segments.push(Segment::line());
        }

        // Content lines
        let left_pad = " ".repeat(self.padding.left);
        let right_pad = " ".repeat(self.padding.right);

        for line in &self.content_lines {
            // Left border
            segments.push(Segment::new(
                box_chars.head[0].to_string(),
                Some(self.border_style.clone()),
            ));

            // Left padding
            if self.padding.left > 0 {
                segments.push(Segment::new(left_pad.clone(), Some(self.style.clone())));
            }

            // Content (with right-padding to fill width)
            let line_width: usize = line.iter().map(|seg| cells::cell_len(&seg.text)).sum();
            for seg in line {
                segments.push(seg.clone());
            }

            // Fill remaining content space
            let fill_width = content_width.saturating_sub(line_width);
            if fill_width > 0 {
                segments.push(Segment::new(
                    " ".repeat(fill_width),
                    Some(self.style.clone()),
                ));
            }

            // Right padding
            if self.padding.right > 0 {
                segments.push(Segment::new(right_pad.clone(), Some(self.style.clone())));
            }

            // Right border
            segments.push(Segment::new(
                box_chars.head[3].to_string(),
                Some(self.border_style.clone()),
            ));
            segments.push(Segment::line());
        }

        // Bottom padding
        for _ in 0..self.padding.bottom {
            segments.push(Segment::new(
                box_chars.head[0].to_string(),
                Some(self.border_style.clone()),
            ));
            segments.push(Segment::new(
                " ".repeat(inner_width),
                Some(self.style.clone()),
            ));
            segments.push(Segment::new(
                box_chars.head[3].to_string(),
                Some(self.border_style.clone()),
            ));
            segments.push(Segment::line());
        }

        // Bottom border with optional subtitle
        segments.extend(self.render_bottom_border(box_chars, inner_width));
        segments.push(Segment::line());

        segments
    }

    /// Render the top border with optional title.
    fn render_top_border(&self, box_chars: &BoxChars, inner_width: usize) -> Vec<Segment> {
        let mut segments = Vec::new();

        // Left corner
        segments.push(Segment::new(
            &box_chars.top[0].to_string(),
            Some(self.border_style.clone()),
        ));

        if let Some(title) = &self.title {
            // Make title with surrounding spaces
            let title_text = format!(" {} ", title.plain());
            let title_width = cells::cell_len(&title_text);

            if title_width + 2 >= inner_width {
                // Title too long, truncate
                let available = inner_width.saturating_sub(2);
                let truncated = truncate_str(&title_text, available);
                segments.push(Segment::new(
                    &box_chars.top[1].to_string(),
                    Some(self.border_style.clone()),
                ));
                segments.push(Segment::new(&truncated, Some(title.style().clone())));
                let remaining = inner_width.saturating_sub(cells::cell_len(&truncated) + 1);
                segments.push(Segment::new(
                    &box_chars.top[1].to_string().repeat(remaining),
                    Some(self.border_style.clone()),
                ));
            } else {
                // Calculate rule sections based on alignment
                let rule_width = inner_width - title_width;
                let (left_rule, right_rule) = match self.title_align {
                    JustifyMethod::Left | JustifyMethod::Default => {
                        (1, rule_width.saturating_sub(1))
                    }
                    JustifyMethod::Right => (rule_width.saturating_sub(1), 1),
                    JustifyMethod::Center | JustifyMethod::Full => {
                        let left = rule_width / 2;
                        (left, rule_width - left)
                    }
                };

                // Left rule section
                segments.push(Segment::new(
                    &box_chars.top[1].to_string().repeat(left_rule),
                    Some(self.border_style.clone()),
                ));

                // Title
                segments.push(Segment::new(&title_text, Some(title.style().clone())));

                // Right rule section
                segments.push(Segment::new(
                    &box_chars.top[1].to_string().repeat(right_rule),
                    Some(self.border_style.clone()),
                ));
            }
        } else {
            // No title, just a line
            segments.push(Segment::new(
                &box_chars.top[1].to_string().repeat(inner_width),
                Some(self.border_style.clone()),
            ));
        }

        // Right corner
        segments.push(Segment::new(
            &box_chars.top[3].to_string(),
            Some(self.border_style.clone()),
        ));

        segments
    }

    /// Render the bottom border with optional subtitle.
    fn render_bottom_border(&self, box_chars: &BoxChars, inner_width: usize) -> Vec<Segment> {
        let mut segments = Vec::new();

        // Left corner
        segments.push(Segment::new(
            &box_chars.bottom[0].to_string(),
            Some(self.border_style.clone()),
        ));

        if let Some(subtitle) = &self.subtitle {
            // Make subtitle with surrounding spaces
            let subtitle_text = format!(" {} ", subtitle.plain());
            let subtitle_width = cells::cell_len(&subtitle_text);

            if subtitle_width + 2 >= inner_width {
                // Subtitle too long, truncate
                let available = inner_width.saturating_sub(2);
                let truncated = truncate_str(&subtitle_text, available);
                segments.push(Segment::new(
                    &box_chars.bottom[1].to_string(),
                    Some(self.border_style.clone()),
                ));
                segments.push(Segment::new(&truncated, Some(subtitle.style().clone())));
                let remaining = inner_width.saturating_sub(cells::cell_len(&truncated) + 1);
                segments.push(Segment::new(
                    &box_chars.bottom[1].to_string().repeat(remaining),
                    Some(self.border_style.clone()),
                ));
            } else {
                // Calculate rule sections based on alignment
                let rule_width = inner_width - subtitle_width;
                let (left_rule, right_rule) = match self.subtitle_align {
                    JustifyMethod::Left | JustifyMethod::Default => {
                        (1, rule_width.saturating_sub(1))
                    }
                    JustifyMethod::Right => (rule_width.saturating_sub(1), 1),
                    JustifyMethod::Center | JustifyMethod::Full => {
                        let left = rule_width / 2;
                        (left, rule_width - left)
                    }
                };

                // Left rule section
                segments.push(Segment::new(
                    &box_chars.bottom[1].to_string().repeat(left_rule),
                    Some(self.border_style.clone()),
                ));

                // Subtitle
                segments.push(Segment::new(&subtitle_text, Some(subtitle.style().clone())));

                // Right rule section
                segments.push(Segment::new(
                    &box_chars.bottom[1].to_string().repeat(right_rule),
                    Some(self.border_style.clone()),
                ));
            }
        } else {
            // No subtitle, just a line
            segments.push(Segment::new(
                &box_chars.bottom[1].to_string().repeat(inner_width),
                Some(self.border_style.clone()),
            ));
        }

        // Right corner
        segments.push(Segment::new(
            &box_chars.bottom[3].to_string(),
            Some(self.border_style.clone()),
        ));

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

/// Truncate a string to fit within a cell width.
fn truncate_str(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut width = 0;

    for ch in s.chars() {
        let ch_width = cells::get_character_cell_size(ch);
        if width + ch_width > max_width {
            if max_width > 3 && width + 3 <= max_width {
                result.push_str("...");
            }
            break;
        }
        result.push(ch);
        width += ch_width;
    }

    result
}

/// Create a panel with content that fits (doesn't expand).
#[must_use]
pub fn fit_panel(text: &str) -> Panel {
    Panel::from_text(text).expand(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_from_text() {
        let panel = Panel::from_text("Hello\nWorld");
        assert_eq!(panel.content_lines.len(), 2);
    }

    #[test]
    fn test_panel_render() {
        let panel = Panel::from_text("Hello").width(20);
        let segments = panel.render(80);
        assert!(!segments.is_empty());

        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains("Hello"));
        // Should have rounded corners by default
        assert!(text.contains('\u{256D}')); // ╭
    }

    #[test]
    fn test_panel_with_title() {
        let panel = Panel::from_text("Content").title("Title").width(30);
        let text = panel.render_plain(80);
        assert!(text.contains("Title"));
        assert!(text.contains("Content"));
    }

    #[test]
    fn test_panel_ascii() {
        let panel = Panel::from_text("Hello").ascii().width(20);
        let text = panel.render_plain(80);
        assert!(text.contains('+')); // ASCII corners
        assert!(text.contains('-')); // ASCII horizontal
    }

    #[test]
    fn test_panel_square() {
        let panel = Panel::from_text("Hello").square().width(20);
        let text = panel.render_plain(80);
        assert!(text.contains('\u{250C}')); // ┌
    }

    #[test]
    fn test_panel_padding() {
        let panel = Panel::from_text("Hi").padding((1, 2)).width(20);
        let segments = panel.render(80);
        // Count newlines to verify padding
        let newlines = segments.iter().filter(|s| s.text == "\n").count();
        // Should have: top border, 1 top pad, content, 1 bottom pad, bottom border
        assert!(newlines >= 5);
    }

    #[test]
    fn test_panel_subtitle() {
        let panel = Panel::from_text("Content").subtitle("Footer").width(30);
        let text = panel.render_plain(80);
        assert!(text.contains("Footer"));
    }

    #[test]
    fn test_fit_panel() {
        let panel = fit_panel("Short");
        assert!(!panel.expand);
    }

    #[test]
    fn test_truncate_str() {
        let s = "Hello World";
        let truncated = truncate_str(s, 5);
        assert_eq!(truncated, "Hello");
    }
}
