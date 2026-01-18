//! Rich text with styled spans.
//!
//! This module provides the `Text` type for representing styled text with
//! overlapping style spans. It's the primary way to build complex styled
//! output that gets rendered to segments.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign};

use crate::cells::cell_len;
use crate::segment::Segment;
use crate::style::Style;

/// Text justification method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JustifyMethod {
    /// Use console default justification.
    #[default]
    Default,
    /// Left-align text.
    Left,
    /// Center text.
    Center,
    /// Right-align text.
    Right,
    /// Justify to fill width (add spaces between words).
    Full,
}

/// Overflow handling method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverflowMethod {
    /// Fold onto next line (default).
    #[default]
    Fold,
    /// Crop at boundary.
    Crop,
    /// Show "..." at truncation point.
    Ellipsis,
    /// No overflow handling.
    Ignore,
}

/// A span of styled text.
///
/// Spans use character indices (not byte indices) to define regions
/// of styled text within a `Text` object. Spans can overlap, with
/// later spans taking precedence during rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Start character index (inclusive).
    pub start: usize,
    /// End character index (exclusive).
    pub end: usize,
    /// Style to apply to this span.
    pub style: Style,
}

impl Span {
    /// Create a new span.
    #[must_use]
    pub fn new(start: usize, end: usize, style: Style) -> Self {
        Self {
            start: start.min(end),
            end: end.max(start),
            style,
        }
    }

    /// Check if this span is empty (zero length).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Get the length of this span in characters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Right-adjust span by offset, clamped to max.
    #[must_use]
    pub fn move_right(&self, offset: usize, max: usize) -> Self {
        Self {
            start: (self.start + offset).min(max),
            end: (self.end + offset).min(max),
            style: self.style.clone(),
        }
    }

    /// Split span at a relative offset.
    ///
    /// Returns (left, right) spans where left ends at `self.start + offset`
    /// and right starts there.
    #[must_use]
    pub fn split(&self, offset: usize) -> (Self, Self) {
        let split_point = self.start + offset;
        (
            Self {
                start: self.start,
                end: split_point.min(self.end),
                style: self.style.clone(),
            },
            Self {
                start: split_point.min(self.end),
                end: self.end,
                style: self.style.clone(),
            },
        )
    }

    /// Adjust span to be relative to a new start position.
    #[must_use]
    pub fn adjust(&self, offset: usize) -> Self {
        Self {
            start: self.start.saturating_sub(offset),
            end: self.end.saturating_sub(offset),
            style: self.style.clone(),
        }
    }
}

/// Rich text with styled spans.
///
/// `Text` represents styled text where different regions can have different
/// styles. Styles are applied via `Span` objects which can overlap - when
/// they do, later spans take precedence.
#[derive(Debug, Clone, Default)]
pub struct Text {
    /// Plain text content.
    plain: String,
    /// Style spans (character indices).
    spans: Vec<Span>,
    /// Cached character length.
    length: usize,
    /// Base style for entire text.
    style: Style,
    /// Text justification method.
    pub justify: JustifyMethod,
    /// Overflow handling method.
    pub overflow: OverflowMethod,
    /// Disable wrapping.
    pub no_wrap: bool,
    /// String to append after text (default "\n").
    pub end: String,
    /// Tab expansion size (default 8).
    pub tab_size: usize,
}

impl Text {
    /// Create a new empty Text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let plain: String = text.into();
        let length = plain.chars().count();
        Self {
            plain,
            spans: Vec::new(),
            length,
            style: Style::default(),
            justify: JustifyMethod::Default,
            overflow: OverflowMethod::Fold,
            no_wrap: false,
            end: String::from("\n"),
            tab_size: 8,
        }
    }

    /// Create a styled Text.
    #[must_use]
    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        let plain: String = text.into();
        let length = plain.chars().count();
        let span = if length > 0 {
            vec![Span::new(0, length, style.clone())]
        } else {
            Vec::new()
        };
        Self {
            plain,
            spans: span,
            length,
            style,
            justify: JustifyMethod::Default,
            overflow: OverflowMethod::Fold,
            no_wrap: false,
            end: String::from("\n"),
            tab_size: 8,
        }
    }

    /// Create Text by assembling multiple styled pieces.
    #[must_use]
    pub fn assemble(pieces: &[(&str, Option<Style>)]) -> Self {
        let mut text = Self::new("");
        for (content, style) in pieces {
            if let Some(s) = style {
                text.append_styled(content, s.clone());
            } else {
                text.append(content);
            }
        }
        text
    }

    /// Get the plain text content.
    #[must_use]
    pub fn plain(&self) -> &str {
        &self.plain
    }

    /// Get the spans.
    #[must_use]
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    /// Get the character length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.length
    }

    /// Check if the text is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.plain.is_empty()
    }

    /// Get the cell width (for terminal display).
    #[must_use]
    pub fn cell_len(&self) -> usize {
        cell_len(&self.plain)
    }

    /// Get the base style.
    #[must_use]
    pub fn style(&self) -> &Style {
        &self.style
    }

    /// Set the base style.
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    /// Append plain text.
    pub fn append(&mut self, text: &str) {
        self.plain.push_str(text);
        self.length += text.chars().count();
    }

    /// Append styled text.
    pub fn append_styled(&mut self, text: &str, style: Style) {
        let start = self.length;
        let text_len = text.chars().count();
        self.plain.push_str(text);
        self.length += text_len;

        if text_len > 0 {
            self.spans.push(Span::new(start, self.length, style));
        }
    }

    /// Append another Text object, merging spans.
    pub fn append_text(&mut self, other: &Text) {
        let offset = self.length;
        self.plain.push_str(&other.plain);
        self.length += other.length;

        // Adjust and add spans from other text
        for span in &other.spans {
            self.spans.push(span.move_right(offset, self.length));
        }
    }

    /// Apply a style to a character range.
    pub fn stylize(&mut self, start: usize, end: usize, style: Style) {
        let clamped_start = start.min(self.length);
        let clamped_end = end.min(self.length);
        if clamped_start < clamped_end {
            self.spans
                .push(Span::new(clamped_start, clamped_end, style));
        }
    }

    /// Apply style to entire text.
    pub fn stylize_all(&mut self, style: Style) {
        if self.length > 0 {
            self.spans.push(Span::new(0, self.length, style));
        }
    }

    /// Highlight text matching a pattern with a style.
    pub fn highlight_regex(&mut self, pattern: &str, style: Style) -> Result<(), regex::Error> {
        let re = regex::Regex::new(pattern)?;
        let chars: Vec<char> = self.plain.chars().collect();

        // Find all matches and convert byte indices to char indices
        for mat in re.find_iter(&self.plain) {
            let byte_start = mat.start();
            let byte_end = mat.end();

            // Convert byte indices to character indices
            let char_start = self.plain[..byte_start].chars().count();
            let char_end = self.plain[..byte_end].chars().count();

            if char_start < char_end && char_end <= chars.len() {
                self.spans
                    .push(Span::new(char_start, char_end, style.clone()));
            }
        }

        Ok(())
    }

    /// Highlight specific words with a style.
    pub fn highlight_words(&mut self, words: &[&str], style: Style, case_sensitive: bool) {
        let text = if case_sensitive {
            self.plain.clone()
        } else {
            self.plain.to_lowercase()
        };

        for word in words {
            let search_word = if case_sensitive {
                word.to_string()
            } else {
                word.to_lowercase()
            };

            let mut search_start = 0;
            while let Some(pos) = text[search_start..].find(&search_word) {
                let byte_start = search_start + pos;
                let byte_end = byte_start + word.len();

                // Convert to char indices
                let char_start = self.plain[..byte_start].chars().count();
                let char_end = self.plain[..byte_end].chars().count();

                self.spans
                    .push(Span::new(char_start, char_end, style.clone()));
                search_start = byte_end;
            }
        }
    }

    /// Get a slice of the text as a new Text object.
    #[must_use]
    pub fn slice(&self, start: usize, end: usize) -> Self {
        let clamped_start = start.min(self.length);
        let clamped_end = end.min(self.length).max(clamped_start);

        if clamped_start >= clamped_end {
            return Self::new("");
        }

        // Extract the substring
        let chars: Vec<char> = self.plain.chars().collect();
        let plain: String = chars[clamped_start..clamped_end].iter().collect();

        // Adjust spans that overlap with the slice
        let mut spans = Vec::new();
        for span in &self.spans {
            if span.end <= clamped_start || span.start >= clamped_end {
                continue; // Span doesn't overlap
            }

            // Calculate intersection and adjust to new coordinates
            let new_start = span.start.max(clamped_start) - clamped_start;
            let new_end = span.end.min(clamped_end) - clamped_start;

            if new_start < new_end {
                spans.push(Span::new(new_start, new_end, span.style.clone()));
            }
        }

        Self {
            plain,
            spans,
            length: clamped_end - clamped_start,
            style: self.style.clone(),
            justify: self.justify,
            overflow: self.overflow,
            no_wrap: self.no_wrap,
            end: self.end.clone(),
            tab_size: self.tab_size,
        }
    }

    /// Split text at newlines.
    #[must_use]
    pub fn split_lines(&self) -> Vec<Self> {
        let mut lines = Vec::new();
        let mut start = 0;

        for (i, c) in self.plain.char_indices() {
            if c == '\n' {
                let char_pos = self.plain[..i].chars().count();
                lines.push(self.slice(start, char_pos));
                start = char_pos + 1;
            }
        }

        // Add the remaining text (or empty line if ends with \n)
        if start <= self.length {
            lines.push(self.slice(start, self.length));
        }

        if lines.is_empty() {
            lines.push(Self::new(""));
        }

        lines
    }

    /// Divide text at specified character offsets.
    #[must_use]
    pub fn divide(&self, offsets: &[usize]) -> Vec<Self> {
        if offsets.is_empty() {
            return vec![self.clone()];
        }

        let mut result = Vec::new();
        let mut prev = 0;

        for &offset in offsets {
            let clamped = offset.min(self.length);
            result.push(self.slice(prev, clamped));
            prev = clamped;
        }

        // Add remaining text
        if prev < self.length {
            result.push(self.slice(prev, self.length));
        } else {
            result.push(Self::new(""));
        }

        result
    }

    /// Expand tabs to spaces.
    #[must_use]
    pub fn expand_tabs(&self, tab_size: usize) -> Self {
        if tab_size == 0 || !self.plain.contains('\t') {
            return self.clone();
        }

        let mut new_plain = String::new();
        let mut char_map: Vec<usize> = Vec::new(); // Maps new char index to old char index
        let mut new_len = 0;
        let mut col = 0;

        for (old_idx, c) in self.plain.chars().enumerate() {
            if c == '\t' {
                let spaces = tab_size - (col % tab_size);
                for _ in 0..spaces {
                    new_plain.push(' ');
                    char_map.push(old_idx);
                    new_len += 1;
                    col += 1;
                }
            } else {
                new_plain.push(c);
                char_map.push(old_idx);
                new_len += 1;
                if c == '\n' {
                    col = 0;
                } else {
                    col += 1;
                }
            }
        }

        // Remap spans to new indices
        let mut new_spans = Vec::new();
        for span in &self.spans {
            // Find new start position
            let new_start = char_map
                .iter()
                .position(|&old| old >= span.start)
                .unwrap_or(new_len);

            // Find new end position
            let new_end = char_map
                .iter()
                .rposition(|&old| old < span.end)
                .map(|p| p + 1)
                .unwrap_or(new_start);

            if new_start < new_end {
                new_spans.push(Span::new(new_start, new_end, span.style.clone()));
            }
        }

        Self {
            plain: new_plain,
            spans: new_spans,
            length: new_len,
            style: self.style.clone(),
            justify: self.justify,
            overflow: self.overflow,
            no_wrap: self.no_wrap,
            end: self.end.clone(),
            tab_size: self.tab_size,
        }
    }

    /// Truncate text to a maximum cell width.
    pub fn truncate(&mut self, max_width: usize, overflow: OverflowMethod, pad: bool) {
        let current_width = self.cell_len();

        if current_width <= max_width {
            if pad && current_width < max_width {
                let padding = " ".repeat(max_width - current_width);
                self.append(&padding);
            }
            return;
        }

        match overflow {
            OverflowMethod::Crop | OverflowMethod::Fold => {
                // Find character position that fits
                let chars: Vec<char> = self.plain.chars().collect();
                let mut width = 0;
                let mut cut_pos = 0;

                for (i, c) in chars.iter().enumerate() {
                    let char_width = crate::cells::get_character_cell_size(*c);
                    if width + char_width > max_width {
                        break;
                    }
                    width += char_width;
                    cut_pos = i + 1;
                }

                *self = self.slice(0, cut_pos);

                if pad && width < max_width {
                    let padding = " ".repeat(max_width - width);
                    self.append(&padding);
                }
            }
            OverflowMethod::Ellipsis => {
                if max_width < 3 {
                    *self = self.slice(0, max_width);
                    return;
                }

                let target_width = max_width - 3;
                let chars: Vec<char> = self.plain.chars().collect();
                let mut width = 0;
                let mut cut_pos = 0;

                for (i, c) in chars.iter().enumerate() {
                    let char_width = crate::cells::get_character_cell_size(*c);
                    if width + char_width > target_width {
                        break;
                    }
                    width += char_width;
                    cut_pos = i + 1;
                }

                *self = self.slice(0, cut_pos);
                self.append("...");

                if pad {
                    let final_width = self.cell_len();
                    if final_width < max_width {
                        let padding = " ".repeat(max_width - final_width);
                        self.append(&padding);
                    }
                }
            }
            OverflowMethod::Ignore => {
                // Do nothing
            }
        }
    }

    /// Pad text to a specific width.
    pub fn pad(&mut self, width: usize, align: JustifyMethod) {
        let current_width = self.cell_len();
        if current_width >= width {
            return;
        }

        let padding = width - current_width;

        match align {
            JustifyMethod::Left | JustifyMethod::Default => {
                self.append(&" ".repeat(padding));
            }
            JustifyMethod::Right => {
                let mut new_text = Self::new(&" ".repeat(padding));
                new_text.append_text(self);
                *self = new_text;
            }
            JustifyMethod::Center => {
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                let mut new_text = Self::new(&" ".repeat(left_pad));
                new_text.append_text(self);
                new_text.append(&" ".repeat(right_pad));
                *self = new_text;
            }
            JustifyMethod::Full => {
                // Full justify doesn't make sense for single text, just right-pad
                self.append(&" ".repeat(padding));
            }
        }
    }

    /// Strip leading and trailing whitespace.
    #[must_use]
    pub fn strip(&self) -> Self {
        let chars: Vec<char> = self.plain.chars().collect();

        // Find first non-whitespace
        let start = chars
            .iter()
            .position(|c| !c.is_whitespace())
            .unwrap_or(chars.len());

        // Find last non-whitespace
        let end = chars
            .iter()
            .rposition(|c| !c.is_whitespace())
            .map(|p| p + 1)
            .unwrap_or(0);

        if start >= end {
            Self::new("")
        } else {
            self.slice(start, end)
        }
    }

    /// Convert text to lowercase.
    #[must_use]
    pub fn to_lowercase(&self) -> Self {
        Self {
            plain: self.plain.to_lowercase(),
            spans: self.spans.clone(),
            length: self.length, // Character count unchanged for most cases
            style: self.style.clone(),
            justify: self.justify,
            overflow: self.overflow,
            no_wrap: self.no_wrap,
            end: self.end.clone(),
            tab_size: self.tab_size,
        }
    }

    /// Convert text to uppercase.
    #[must_use]
    pub fn to_uppercase(&self) -> Self {
        Self {
            plain: self.plain.to_uppercase(),
            spans: self.spans.clone(),
            length: self.length,
            style: self.style.clone(),
            justify: self.justify,
            overflow: self.overflow,
            no_wrap: self.no_wrap,
            end: self.end.clone(),
            tab_size: self.tab_size,
        }
    }

    /// Render text to segments.
    #[must_use]
    pub fn render(&self, end: &str) -> Vec<Segment> {
        if self.plain.is_empty() {
            return if end.is_empty() {
                Vec::new()
            } else {
                vec![Segment::new(end, None)]
            };
        }

        // Build event map: position -> list of (span_index, is_start)
        let mut events: BTreeMap<usize, Vec<(usize, bool)>> = BTreeMap::new();
        for (idx, span) in self.spans.iter().enumerate() {
            events.entry(span.start).or_default().push((idx, true));
            events.entry(span.end).or_default().push((idx, false));
        }

        let chars: Vec<char> = self.plain.chars().collect();
        let mut result = Vec::new();
        let mut active_spans: Vec<usize> = Vec::new();
        let mut style_cache: HashMap<u64, Style> = HashMap::new();
        let mut pos = 0;

        for (event_pos, span_events) in events {
            // Emit text before this event
            if event_pos > pos && pos < chars.len() {
                let text: String = chars[pos..event_pos.min(chars.len())].iter().collect();
                let style = self.compute_style(&active_spans, &mut style_cache);
                result.push(Segment::new(text, Some(style)));
                pos = event_pos;
            }

            // Process events (ends before starts for correct nesting)
            let mut ends: Vec<usize> = Vec::new();
            let mut starts: Vec<usize> = Vec::new();

            for (span_idx, is_start) in span_events {
                if is_start {
                    starts.push(span_idx);
                } else {
                    ends.push(span_idx);
                }
            }

            // Remove ended spans
            for span_idx in ends {
                active_spans.retain(|&x| x != span_idx);
            }

            // Add started spans
            active_spans.extend(starts);
        }

        // Emit remaining text
        if pos < chars.len() {
            let text: String = chars[pos..].iter().collect();
            let style = self.compute_style(&active_spans, &mut style_cache);
            result.push(Segment::new(text, Some(style)));
        }

        // Append end string
        if !end.is_empty() {
            result.push(Segment::new(end, None));
        }

        result
    }

    /// Compute combined style from active spans.
    fn compute_style(&self, active_spans: &[usize], cache: &mut HashMap<u64, Style>) -> Style {
        // Create cache key
        let cache_key = self.hash_spans(active_spans);

        if let Some(cached) = cache.get(&cache_key) {
            return cached.clone();
        }

        let mut combined = self.style.clone();
        for &span_idx in active_spans {
            if let Some(span) = self.spans.get(span_idx) {
                combined = combined.combine(&span.style);
            }
        }

        cache.insert(cache_key, combined.clone());
        combined
    }

    /// Hash span indices for caching.
    fn hash_spans(&self, spans: &[usize]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        spans.hash(&mut hasher);
        hasher.finish()
    }

    /// Word wrap text to fit within a width.
    #[must_use]
    pub fn wrap(&self, width: usize) -> Vec<Self> {
        if width == 0 {
            return vec![Self::new("")];
        }

        let expanded = self.expand_tabs(self.tab_size);

        if expanded.no_wrap || expanded.cell_len() <= width {
            return vec![expanded];
        }

        let mut lines = Vec::new();

        for line in expanded.split_lines() {
            if line.cell_len() <= width {
                lines.push(line);
            } else {
                lines.extend(self.wrap_line(&line, width));
            }
        }

        lines
    }

    /// Wrap a single line of text.
    fn wrap_line(&self, line: &Text, width: usize) -> Vec<Self> {
        let mut result = Vec::new();
        let chars: Vec<char> = line.plain.chars().collect();

        if chars.is_empty() {
            return vec![Self::new("")];
        }

        match line.overflow {
            OverflowMethod::Fold => {
                // Wrap at word boundaries when possible
                let mut current_line_start = 0;
                let mut current_width = 0;
                let mut last_space = None;

                for (i, c) in chars.iter().enumerate() {
                    let char_width = crate::cells::get_character_cell_size(*c);

                    if c.is_whitespace() && *c != '\n' {
                        last_space = Some(i);
                    }

                    if current_width + char_width > width {
                        // Need to wrap
                        let wrap_at = if let Some(space_pos) = last_space {
                            if space_pos > current_line_start {
                                space_pos
                            } else {
                                i
                            }
                        } else {
                            i
                        };

                        if wrap_at > current_line_start {
                            result.push(line.slice(current_line_start, wrap_at));
                        }

                        // Skip whitespace at wrap point
                        current_line_start = wrap_at;
                        while current_line_start < chars.len()
                            && chars[current_line_start].is_whitespace()
                        {
                            current_line_start += 1;
                        }

                        current_width = 0;
                        last_space = None;

                        // Recalculate width from new start
                        for j in current_line_start..=i {
                            if j < chars.len() {
                                current_width += crate::cells::get_character_cell_size(chars[j]);
                            }
                        }
                    } else {
                        current_width += char_width;
                    }
                }

                // Add remaining text
                if current_line_start < chars.len() {
                    result.push(line.slice(current_line_start, chars.len()));
                }
            }
            OverflowMethod::Crop => {
                result.push(line.slice(0, self.char_pos_for_width(line, width)));
            }
            OverflowMethod::Ellipsis => {
                if width >= 3 {
                    let mut truncated = line.slice(0, self.char_pos_for_width(line, width - 3));
                    truncated.append("...");
                    result.push(truncated);
                } else {
                    result.push(line.slice(0, width));
                }
            }
            OverflowMethod::Ignore => {
                result.push(line.clone());
            }
        }

        if result.is_empty() {
            result.push(Self::new(""));
        }

        result
    }

    /// Find character position for a target cell width.
    fn char_pos_for_width(&self, text: &Text, target_width: usize) -> usize {
        let mut width = 0;
        for (i, c) in text.plain.chars().enumerate() {
            let char_width = crate::cells::get_character_cell_size(c);
            if width + char_width > target_width {
                return i;
            }
            width += char_width;
        }
        text.length
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.plain)
    }
}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.plain == other.plain && self.spans == other.spans
    }
}

impl Eq for Text {}

impl Add for Text {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.append_text(&rhs);
        self
    }
}

impl AddAssign for Text {
    fn add_assign(&mut self, rhs: Self) {
        self.append_text(&rhs);
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_new() {
        let text = Text::new("hello");
        assert_eq!(text.plain(), "hello");
        assert_eq!(text.len(), 5);
        assert!(!text.is_empty());
    }

    #[test]
    fn test_text_styled() {
        let style = Style::new().bold();
        let text = Text::styled("hello", style);
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].start, 0);
        assert_eq!(text.spans()[0].end, 5);
    }

    #[test]
    fn test_text_append() {
        let mut text = Text::new("hello");
        text.append(" world");
        assert_eq!(text.plain(), "hello world");
        assert_eq!(text.len(), 11);
    }

    #[test]
    fn test_text_append_styled() {
        let mut text = Text::new("hello ");
        text.append_styled("world", Style::new().bold());
        assert_eq!(text.plain(), "hello world");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].start, 6);
        assert_eq!(text.spans()[0].end, 11);
    }

    #[test]
    fn test_text_slice() {
        let mut text = Text::new("hello world");
        text.stylize(0, 5, Style::new().bold());
        text.stylize(6, 11, Style::new().italic());

        let slice = text.slice(3, 8);
        assert_eq!(slice.plain(), "lo wo");
        assert_eq!(slice.len(), 5);
        // Should have adjusted spans
        assert_eq!(slice.spans().len(), 2);
    }

    #[test]
    fn test_text_split_lines() {
        let text = Text::new("line1\nline2\nline3");
        let lines = text.split_lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].plain(), "line1");
        assert_eq!(lines[1].plain(), "line2");
        assert_eq!(lines[2].plain(), "line3");
    }

    #[test]
    fn test_text_divide() {
        let text = Text::new("hello world");
        let parts = text.divide(&[5]);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].plain(), "hello");
        assert_eq!(parts[1].plain(), " world");
    }

    #[test]
    fn test_text_expand_tabs() {
        let text = Text::new("a\tb");
        let expanded = text.expand_tabs(8);
        assert_eq!(expanded.plain(), "a       b");
    }

    #[test]
    fn test_text_truncate() {
        let mut text = Text::new("hello world");
        text.truncate(8, OverflowMethod::Ellipsis, false);
        assert_eq!(text.plain(), "hello...");
    }

    #[test]
    fn test_text_pad() {
        let mut text = Text::new("hi");
        text.pad(5, JustifyMethod::Center);
        assert_eq!(text.cell_len(), 5);
    }

    #[test]
    fn test_text_strip() {
        let text = Text::new("  hello  ");
        let stripped = text.strip();
        assert_eq!(stripped.plain(), "hello");
    }

    #[test]
    fn test_text_render() {
        let mut text = Text::new("hello world");
        text.stylize(0, 5, Style::new().bold());

        let segments = text.render("");
        assert!(segments.len() >= 2);
    }

    #[test]
    fn test_text_add() {
        let a = Text::new("hello ");
        let b = Text::new("world");
        let combined = a + b;
        assert_eq!(combined.plain(), "hello world");
    }

    #[test]
    fn test_span_split() {
        let span = Span::new(0, 10, Style::new().bold());
        let (left, right) = span.split(5);
        assert_eq!(left.start, 0);
        assert_eq!(left.end, 5);
        assert_eq!(right.start, 5);
        assert_eq!(right.end, 10);
    }

    #[test]
    fn test_span_move_right() {
        let span = Span::new(0, 5, Style::new().bold());
        let moved = span.move_right(10, 20);
        assert_eq!(moved.start, 10);
        assert_eq!(moved.end, 15);
    }

    #[test]
    fn test_cell_len_cjk() {
        let text = Text::new("Hello\u{4e2d}\u{6587}");
        // "Hello" = 5 cells, "中文" = 4 cells (2 chars * 2 cells each)
        assert_eq!(text.cell_len(), 9);
    }

    #[test]
    fn test_assemble() {
        let text = Text::assemble(&[("hello ", None), ("world", Some(Style::new().bold()))]);
        assert_eq!(text.plain(), "hello world");
        assert_eq!(text.spans().len(), 1);
    }

    // ============================================================
    // Additional tests for comprehensive coverage (rich_rust-zca)
    // ============================================================

    // --- Text Construction Tests ---

    #[test]
    fn test_text_from_str() {
        let text: Text = "hello".into();
        assert_eq!(text.plain(), "hello");
        assert_eq!(text.len(), 5);
    }

    #[test]
    fn test_text_from_string() {
        let text: Text = String::from("hello").into();
        assert_eq!(text.plain(), "hello");
        assert_eq!(text.len(), 5);
    }

    #[test]
    fn test_text_empty() {
        let text = Text::new("");
        assert!(text.is_empty());
        assert_eq!(text.len(), 0);
        assert_eq!(text.plain(), "");
    }

    #[test]
    fn test_text_styled_empty() {
        let text = Text::styled("", Style::new().bold());
        assert!(text.is_empty());
        // Empty styled text should have no spans
        assert!(text.spans().is_empty());
    }

    // --- Overlapping Spans Tests ---

    #[test]
    fn test_overlapping_spans() {
        let mut text = Text::new("hello world");
        // First span: bold for "hello world"
        text.stylize(0, 11, Style::new().bold());
        // Second span: italic for "llo wor" (overlapping)
        text.stylize(2, 9, Style::new().italic());

        assert_eq!(text.spans().len(), 2);
        // Both spans should be stored
        let segments = text.render("");
        // Should render with combined styles where spans overlap
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_adjacent_spans() {
        let mut text = Text::new("helloworld");
        // Adjacent spans: "hello" bold, "world" italic
        text.stylize(0, 5, Style::new().bold());
        text.stylize(5, 10, Style::new().italic());

        assert_eq!(text.spans().len(), 2);
        let segments = text.render("");
        // Should have at least 2 segments with different styles
        assert!(segments.len() >= 2);
    }

    #[test]
    fn test_nested_spans() {
        let mut text = Text::new("hello world");
        // Outer: entire text bold
        text.stylize(0, 11, Style::new().bold());
        // Inner: "world" also red
        text.stylize(6, 11, Style::new().color_str("red").unwrap_or_default());

        let segments = text.render("");
        // "hello " should be bold only, "world" should be bold+red
        assert!(!segments.is_empty());
    }

    // --- Text Rendering Tests ---

    #[test]
    fn test_render_empty() {
        let text = Text::new("");
        let segments = text.render("");
        assert!(segments.is_empty());
    }

    #[test]
    fn test_render_with_end() {
        let text = Text::new("hello");
        let segments = text.render("\n");
        // Should have text segment + end segment
        assert!(segments.len() >= 2);
        assert_eq!(segments.last().unwrap().text, "\n");
    }

    #[test]
    fn test_render_base_style() {
        let mut text = Text::new("hello");
        text.set_style(Style::new().bold());
        let segments = text.render("");
        // Base style should be applied
        assert!(!segments.is_empty());
        let style = segments[0].style.as_ref().unwrap();
        assert!(style.attributes.contains(crate::style::Attributes::BOLD));
    }

    // --- Text Division Tests (CRITICAL) ---

    #[test]
    fn test_divide_with_span_crossing_boundary() {
        let mut text = Text::new("hello world");
        // Span covers "lo wor" (positions 3-9)
        text.stylize(3, 9, Style::new().bold());

        // Divide at position 5 (between "hello" and " world")
        let parts = text.divide(&[5]);

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].plain(), "hello");
        assert_eq!(parts[1].plain(), " world");

        // First part should have span for "lo" (original 3-5 → adjusted to 3-5)
        assert_eq!(parts[0].spans().len(), 1);
        assert_eq!(parts[0].spans()[0].start, 3);
        assert_eq!(parts[0].spans()[0].end, 5);

        // Second part should have span for " wor" (original 5-9 → adjusted to 0-4)
        assert_eq!(parts[1].spans().len(), 1);
        assert_eq!(parts[1].spans()[0].start, 0);
        assert_eq!(parts[1].spans()[0].end, 4);
    }

    #[test]
    fn test_divide_span_starts_at_cut() {
        let mut text = Text::new("hello world");
        // Span starts exactly at position 5
        text.stylize(5, 11, Style::new().bold());

        let parts = text.divide(&[5]);

        assert_eq!(parts[0].spans().len(), 0); // No span in first part
        assert_eq!(parts[1].spans().len(), 1); // Span in second part
        assert_eq!(parts[1].spans()[0].start, 0);
        assert_eq!(parts[1].spans()[0].end, 6);
    }

    #[test]
    fn test_divide_span_ends_at_cut() {
        let mut text = Text::new("hello world");
        // Span ends exactly at position 5
        text.stylize(0, 5, Style::new().bold());

        let parts = text.divide(&[5]);

        assert_eq!(parts[0].spans().len(), 1); // Span in first part
        assert_eq!(parts[0].spans()[0].start, 0);
        assert_eq!(parts[0].spans()[0].end, 5);
        assert_eq!(parts[1].spans().len(), 0); // No span in second part
    }

    #[test]
    fn test_divide_multiple_spans() {
        let mut text = Text::new("hello world!");
        text.stylize(0, 5, Style::new().bold()); // "hello"
        text.stylize(6, 11, Style::new().italic()); // "world"

        let parts = text.divide(&[6]);

        assert_eq!(parts[0].plain(), "hello ");
        assert_eq!(parts[1].plain(), "world!");

        // First part has "hello" span
        assert_eq!(parts[0].spans().len(), 1);
        // Second part has "world" span (adjusted)
        assert_eq!(parts[1].spans().len(), 1);
        assert_eq!(parts[1].spans()[0].start, 0);
        assert_eq!(parts[1].spans()[0].end, 5);
    }

    #[test]
    fn test_divide_empty_offsets() {
        let text = Text::new("hello");
        let parts = text.divide(&[]);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].plain(), "hello");
    }

    #[test]
    fn test_divide_multiple_cuts() {
        let text = Text::new("hello world!");
        let parts = text.divide(&[5, 6, 11]);

        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0].plain(), "hello");
        assert_eq!(parts[1].plain(), " ");
        assert_eq!(parts[2].plain(), "world");
        assert_eq!(parts[3].plain(), "!");
    }

    #[test]
    fn test_divide_cut_at_end() {
        let text = Text::new("hello");
        let parts = text.divide(&[5]);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].plain(), "hello");
        assert_eq!(parts[1].plain(), "");
    }

    #[test]
    fn test_divide_cut_beyond_length() {
        let text = Text::new("hello");
        let parts = text.divide(&[10]);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].plain(), "hello");
        assert_eq!(parts[1].plain(), "");
    }

    // --- Text Wrapping Tests ---

    #[test]
    fn test_wrap_basic() {
        let text = Text::new("hello world foo bar");
        let lines = text.wrap(10);
        // Should wrap at word boundaries
        assert!(lines.len() >= 2);
        for line in &lines {
            assert!(line.cell_len() <= 10);
        }
    }

    #[test]
    fn test_wrap_long_word() {
        let text = Text::new("supercalifragilistic");
        let lines = text.wrap(10);
        // Should break the long word
        assert!(lines.len() >= 2);
        for line in &lines {
            assert!(line.cell_len() <= 10);
        }
    }

    #[test]
    fn test_wrap_zero_width() {
        let text = Text::new("hello");
        let lines = text.wrap(0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].plain(), "");
    }

    #[test]
    fn test_wrap_no_wrap_flag() {
        let mut text = Text::new("hello world this is long");
        text.no_wrap = true;
        let lines = text.wrap(10);
        // Should return original text without wrapping
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].plain(), "hello world this is long");
    }

    #[test]
    fn test_wrap_fits_width() {
        let text = Text::new("hello");
        let lines = text.wrap(20);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].plain(), "hello");
    }

    #[test]
    fn test_wrap_with_wide_chars() {
        let text = Text::new("你好世界"); // 8 cells (4 chars * 2 cells each)
        let lines = text.wrap(6);
        // Should wrap CJK characters correctly
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_wrap_preserves_spans() {
        let mut text = Text::new("hello world");
        text.stylize(0, 5, Style::new().bold());
        let lines = text.wrap(6);
        // First line should contain span
        assert!(!lines[0].spans().is_empty());
    }

    #[test]
    fn test_wrap_overflow_crop() {
        let mut text = Text::new("hello world this is too long");
        text.overflow = OverflowMethod::Crop;
        let lines = text.wrap(10);
        // Should crop, not wrap
        assert_eq!(lines.len(), 1);
        assert!(lines[0].cell_len() <= 10);
    }

    #[test]
    fn test_wrap_overflow_ellipsis() {
        let mut text = Text::new("hello world this is too long");
        text.overflow = OverflowMethod::Ellipsis;
        let lines = text.wrap(10);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].plain().ends_with("..."));
    }

    // --- Justification Tests ---

    #[test]
    fn test_pad_left() {
        let mut text = Text::new("hi");
        text.pad(5, JustifyMethod::Left);
        assert_eq!(text.cell_len(), 5);
        assert_eq!(text.plain(), "hi   ");
    }

    #[test]
    fn test_pad_right() {
        let mut text = Text::new("hi");
        text.pad(5, JustifyMethod::Right);
        assert_eq!(text.cell_len(), 5);
        assert_eq!(text.plain(), "   hi");
    }

    #[test]
    fn test_pad_center() {
        let mut text = Text::new("hi");
        text.pad(6, JustifyMethod::Center);
        assert_eq!(text.cell_len(), 6);
        assert_eq!(text.plain(), "  hi  ");
    }

    #[test]
    fn test_pad_full() {
        let mut text = Text::new("hi");
        text.pad(5, JustifyMethod::Full);
        // Full justify just right-pads for single text
        assert_eq!(text.cell_len(), 5);
    }

    #[test]
    fn test_pad_already_wide() {
        let mut text = Text::new("hello");
        text.pad(3, JustifyMethod::Center);
        // Should not change if already wider
        assert_eq!(text.plain(), "hello");
    }

    // --- Slice Tests ---

    #[test]
    fn test_slice_empty_range() {
        let text = Text::new("hello");
        let slice = text.slice(3, 3);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_slice_out_of_bounds() {
        let text = Text::new("hello");
        let slice = text.slice(10, 20);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_slice_reversed_range() {
        let text = Text::new("hello");
        let slice = text.slice(4, 2);
        // Should handle reversed range gracefully
        assert!(slice.is_empty());
    }

    #[test]
    fn test_slice_preserves_span() {
        let mut text = Text::new("hello world");
        text.stylize(0, 5, Style::new().bold());
        let slice = text.slice(0, 3);
        assert_eq!(slice.plain(), "hel");
        assert_eq!(slice.spans().len(), 1);
        assert_eq!(slice.spans()[0].start, 0);
        assert_eq!(slice.spans()[0].end, 3);
    }

    // --- Span Helper Tests ---

    #[test]
    fn test_span_is_empty() {
        let empty = Span::new(5, 5, Style::new());
        assert!(empty.is_empty());

        let non_empty = Span::new(0, 5, Style::new());
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(3, 10, Style::new());
        assert_eq!(span.len(), 7);
    }

    #[test]
    fn test_span_adjust() {
        let span = Span::new(10, 15, Style::new());
        let adjusted = span.adjust(5);
        assert_eq!(adjusted.start, 5);
        assert_eq!(adjusted.end, 10);
    }

    #[test]
    fn test_span_new_swaps_if_needed() {
        // If start > end, they should be swapped
        let span = Span::new(10, 5, Style::new());
        assert_eq!(span.start, 5);
        assert_eq!(span.end, 10);
    }

    // --- Tab Expansion Tests ---

    #[test]
    fn test_expand_tabs_multiple() {
        let text = Text::new("a\tb\tc");
        let expanded = text.expand_tabs(4);
        // Each tab expands to fill to next multiple of 4
        assert!(!expanded.plain().contains('\t'));
    }

    #[test]
    fn test_expand_tabs_preserves_spans() {
        let mut text = Text::new("a\tb");
        text.stylize(0, 1, Style::new().bold()); // Just "a"
        let expanded = text.expand_tabs(4);
        // Span should still exist
        assert!(!expanded.spans().is_empty());
    }

    #[test]
    fn test_expand_tabs_zero_size() {
        let text = Text::new("a\tb");
        let expanded = text.expand_tabs(0);
        // Should return unchanged
        assert_eq!(expanded.plain(), "a\tb");
    }

    // --- Other Tests ---

    #[test]
    fn test_text_display() {
        let text = Text::new("hello");
        assert_eq!(format!("{text}"), "hello");
    }

    #[test]
    fn test_text_equality() {
        let a = Text::new("hello");
        let b = Text::new("hello");
        assert_eq!(a, b);

        let c = Text::new("world");
        assert_ne!(a, c);
    }

    #[test]
    fn test_text_add_assign() {
        let mut text = Text::new("hello ");
        text += Text::new("world");
        assert_eq!(text.plain(), "hello world");
    }

    #[test]
    fn test_stylize_all() {
        let mut text = Text::new("hello");
        text.stylize_all(Style::new().bold());
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].start, 0);
        assert_eq!(text.spans()[0].end, 5);
    }

    #[test]
    fn test_stylize_clamps() {
        let mut text = Text::new("hello");
        // Stylize beyond text length - should clamp
        text.stylize(3, 100, Style::new().bold());
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].end, 5); // Clamped to text length
    }

    #[test]
    fn test_to_lowercase() {
        let text = Text::new("Hello WORLD");
        let lower = text.to_lowercase();
        assert_eq!(lower.plain(), "hello world");
    }

    #[test]
    fn test_to_uppercase() {
        let text = Text::new("Hello World");
        let upper = text.to_uppercase();
        assert_eq!(upper.plain(), "HELLO WORLD");
    }

    #[test]
    fn test_append_text_merges_spans() {
        let mut a = Text::new("hello");
        a.stylize(0, 5, Style::new().bold());

        let mut b = Text::new("world");
        b.stylize(0, 5, Style::new().italic());

        a.append_text(&b);

        assert_eq!(a.plain(), "helloworld");
        assert_eq!(a.spans().len(), 2);
        // Second span should be offset
        assert_eq!(a.spans()[1].start, 5);
        assert_eq!(a.spans()[1].end, 10);
    }

    #[test]
    fn test_highlight_regex() {
        let mut text = Text::new("hello world hello");
        text.highlight_regex("hello", Style::new().bold()).unwrap();
        // Should have 2 spans for the two "hello" matches
        assert_eq!(text.spans().len(), 2);
    }

    #[test]
    fn test_highlight_words() {
        let mut text = Text::new("Hello World HELLO");
        text.highlight_words(&["hello"], Style::new().bold(), false);
        // Case insensitive - should find 2 matches
        assert_eq!(text.spans().len(), 2);
    }

    #[test]
    fn test_split_lines_empty() {
        let text = Text::new("");
        let lines = text.split_lines();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].is_empty());
    }

    #[test]
    fn test_split_lines_trailing_newline() {
        let text = Text::new("hello\n");
        let lines = text.split_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].plain(), "hello");
        assert_eq!(lines[1].plain(), "");
    }

    #[test]
    fn test_truncate_crop() {
        let mut text = Text::new("hello world");
        text.truncate(5, OverflowMethod::Crop, false);
        assert_eq!(text.plain(), "hello");
    }

    #[test]
    fn test_truncate_with_pad() {
        let mut text = Text::new("hi");
        text.truncate(5, OverflowMethod::Crop, true);
        assert_eq!(text.cell_len(), 5);
        assert_eq!(text.plain(), "hi   ");
    }
}
