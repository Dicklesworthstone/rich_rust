//! Style system for terminal text attributes.
//!
//! This module provides the `Style` struct for representing visual attributes
//! including colors, text decorations (bold, italic, etc.), and hyperlinks.

use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;
use bitflags::bitflags;
use lru::LruCache;
use std::sync::Mutex;
use std::num::NonZeroUsize;

use crate::color::{Color, ColorParseError, ColorSystem, ColorTriplet};

bitflags! {
    /// Text attribute flags.
    ///
    /// Each flag corresponds to an ANSI SGR (Select Graphic Rendition) code.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Attributes: u16 {
        /// Bold/bright text (SGR 1).
        const BOLD      = 1 << 0;
        /// Dim/faint text (SGR 2).
        const DIM       = 1 << 1;
        /// Italic text (SGR 3).
        const ITALIC    = 1 << 2;
        /// Single underline (SGR 4).
        const UNDERLINE = 1 << 3;
        /// Slow blinking text (SGR 5).
        const BLINK     = 1 << 4;
        /// Fast blinking text (SGR 6).
        const BLINK2    = 1 << 5;
        /// Reverse video (SGR 7).
        const REVERSE   = 1 << 6;
        /// Concealed/hidden text (SGR 8).
        const CONCEAL   = 1 << 7;
        /// Strikethrough text (SGR 9).
        const STRIKE    = 1 << 8;
        /// Double underline (SGR 21).
        const UNDERLINE2 = 1 << 9;
        /// Framed text (SGR 51).
        const FRAME     = 1 << 10;
        /// Encircled text (SGR 52).
        const ENCIRCLE  = 1 << 11;
        /// Overlined text (SGR 53).
        const OVERLINE  = 1 << 12;
    }
}

impl Attributes {
    /// Map of attribute flags to their ANSI SGR codes.
    const SGR_CODES: [(Self, u8); 13] = [
        (Self::BOLD, 1),
        (Self::DIM, 2),
        (Self::ITALIC, 3),
        (Self::UNDERLINE, 4),
        (Self::BLINK, 5),
        (Self::BLINK2, 6),
        (Self::REVERSE, 7),
        (Self::CONCEAL, 8),
        (Self::STRIKE, 9),
        (Self::UNDERLINE2, 21),
        (Self::FRAME, 51),
        (Self::ENCIRCLE, 52),
        (Self::OVERLINE, 53),
    ];

    /// Get the ANSI SGR codes for enabled attributes.
    #[must_use]
    pub fn to_sgr_codes(&self) -> Vec<u8> {
        Self::SGR_CODES
            .iter()
            .filter_map(|(attr, code)| {
                if self.contains(*attr) {
                    Some(*code)
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Visual style for terminal text.
///
/// A `Style` represents the complete visual appearance of text including:
/// - Foreground and background colors
/// - Text attributes (bold, italic, etc.)
/// - Hyperlinks
///
/// Styles can be combined using the `+` operator, where the right-hand style
/// takes precedence for conflicting properties.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Style {
    /// Foreground color.
    pub color: Option<Color>,
    /// Background color.
    pub bgcolor: Option<Color>,
    /// Enabled attributes.
    pub attributes: Attributes,
    /// Which attributes are explicitly set (vs inherited).
    pub set_attributes: Attributes,
    /// URL for hyperlinks.
    pub link: Option<String>,
    /// Whether this is a null/empty style.
    null: bool,
}

impl Style {
    /// Create an empty (null) style.
    #[must_use]
    pub fn null() -> Self {
        Self {
            null: true,
            ..Default::default()
        }
    }

    /// Create a new style builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if this is a null/empty style.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.null
    }

    /// Set the foreground color.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self.null = false;
        self
    }

    /// Set the foreground color from a string.
    pub fn color_str(self, color: &str) -> Result<Self, StyleParseError> {
        let c = Color::parse(color)?;
        Ok(self.color(c))
    }

    /// Set the background color.
    #[must_use]
    pub fn bgcolor(mut self, color: Color) -> Self {
        self.bgcolor = Some(color);
        self.null = false;
        self
    }

    /// Set the background color from a string.
    pub fn bgcolor_str(self, color: &str) -> Result<Self, StyleParseError> {
        let c = Color::parse(color)?;
        Ok(self.bgcolor(c))
    }

    /// Enable bold text.
    #[must_use]
    pub fn bold(mut self) -> Self {
        self.attributes.insert(Attributes::BOLD);
        self.set_attributes.insert(Attributes::BOLD);
        self.null = false;
        self
    }

    /// Enable dim/faint text.
    #[must_use]
    pub fn dim(mut self) -> Self {
        self.attributes.insert(Attributes::DIM);
        self.set_attributes.insert(Attributes::DIM);
        self.null = false;
        self
    }

    /// Enable italic text.
    #[must_use]
    pub fn italic(mut self) -> Self {
        self.attributes.insert(Attributes::ITALIC);
        self.set_attributes.insert(Attributes::ITALIC);
        self.null = false;
        self
    }

    /// Enable underlined text.
    #[must_use]
    pub fn underline(mut self) -> Self {
        self.attributes.insert(Attributes::UNDERLINE);
        self.set_attributes.insert(Attributes::UNDERLINE);
        self.null = false;
        self
    }

    /// Enable blinking text.
    #[must_use]
    pub fn blink(mut self) -> Self {
        self.attributes.insert(Attributes::BLINK);
        self.set_attributes.insert(Attributes::BLINK);
        self.null = false;
        self
    }

    /// Enable reverse video.
    #[must_use]
    pub fn reverse(mut self) -> Self {
        self.attributes.insert(Attributes::REVERSE);
        self.set_attributes.insert(Attributes::REVERSE);
        self.null = false;
        self
    }

    /// Enable concealed/hidden text.
    #[must_use]
    pub fn conceal(mut self) -> Self {
        self.attributes.insert(Attributes::CONCEAL);
        self.set_attributes.insert(Attributes::CONCEAL);
        self.null = false;
        self
    }

    /// Enable strikethrough text.
    #[must_use]
    pub fn strike(mut self) -> Self {
        self.attributes.insert(Attributes::STRIKE);
        self.set_attributes.insert(Attributes::STRIKE);
        self.null = false;
        self
    }

    /// Enable overlined text.
    #[must_use]
    pub fn overline(mut self) -> Self {
        self.attributes.insert(Attributes::OVERLINE);
        self.set_attributes.insert(Attributes::OVERLINE);
        self.null = false;
        self
    }

    /// Set a hyperlink URL.
    #[must_use]
    pub fn link(mut self, url: impl Into<String>) -> Self {
        self.link = Some(url.into());
        self.null = false;
        self
    }

    /// Disable a specific attribute.
    #[must_use]
    pub fn not(mut self, attr: Attributes) -> Self {
        self.attributes.remove(attr);
        self.set_attributes.insert(attr);
        self.null = false;
        self
    }

    /// Combine this style with another, with the other style taking precedence.
    #[must_use]
    pub fn combine(&self, other: &Style) -> Style {
        if other.is_null() {
            return self.clone();
        }
        if self.is_null() {
            return other.clone();
        }

        Style {
            color: other.color.clone().or_else(|| self.color.clone()),
            bgcolor: other.bgcolor.clone().or_else(|| self.bgcolor.clone()),
            attributes: (self.attributes & !other.set_attributes)
                | (other.attributes & other.set_attributes),
            set_attributes: self.set_attributes | other.set_attributes,
            link: other.link.clone().or_else(|| self.link.clone()),
            null: false,
        }
    }

    /// Generate ANSI escape codes for this style.
    #[must_use]
    pub fn make_ansi_codes(&self, color_system: ColorSystem) -> String {
        let mut codes: Vec<String> = Vec::new();

        // Add attribute codes
        for code in self.attributes.to_sgr_codes() {
            codes.push(code.to_string());
        }

        // Add foreground color codes
        if let Some(color) = &self.color {
            let downgraded = color.downgrade(color_system);
            codes.extend(downgraded.get_ansi_codes(true));
        }

        // Add background color codes
        if let Some(bgcolor) = &self.bgcolor {
            let downgraded = bgcolor.downgrade(color_system);
            codes.extend(downgraded.get_ansi_codes(false));
        }

        codes.join(";")
    }

    /// Render text with this style applied.
    #[must_use]
    pub fn render(&self, text: &str, color_system: ColorSystem) -> String {
        if self.is_null() {
            return text.to_string();
        }

        let codes = self.make_ansi_codes(color_system);
        if codes.is_empty() {
            return text.to_string();
        }

        let mut result = String::with_capacity(text.len() + codes.len() + 10);

        // Handle hyperlinks (OSC 8)
        if let Some(link) = &self.link {
            result.push_str(&format!("\x1b]8;;{link}\x1b\\"));
        }

        // Apply style
        result.push_str(&format!("\x1b[{codes}m"));
        result.push_str(text);
        result.push_str("\x1b[0m");

        // Close hyperlink
        if self.link.is_some() {
            result.push_str("\x1b]8;;\x1b\\");
        }

        result
    }

    /// Get ANSI codes as (prefix, suffix) tuple.
    ///
    /// The prefix contains the escape codes to apply the style,
    /// and the suffix contains the reset codes.
    #[must_use]
    pub fn render_ansi(&self, color_system: ColorSystem) -> (String, String) {
        if self.is_null() {
            return (String::new(), String::new());
        }

        let codes = self.make_ansi_codes(color_system);
        if codes.is_empty() {
            return (String::new(), String::new());
        }

        let mut prefix = String::new();
        let suffix;

        // Handle hyperlinks (OSC 8)
        if let Some(link) = &self.link {
            prefix.push_str(&format!("\x1b]8;;{link}\x1b\\"));
        }

        // Apply style
        prefix.push_str(&format!("\x1b[{codes}m"));

        // Build suffix
        if self.link.is_some() {
            suffix = String::from("\x1b[0m\x1b]8;;\x1b\\");
        } else {
            suffix = String::from("\x1b[0m");
        }

        (prefix, suffix)
    }

    /// Parse a style from a string (cached).
    ///
    /// Supported formats:
    /// - Empty/none: `""`, `"none"` -> null style
    /// - Attribute: `"bold"`, `"italic"`, `"underline"`
    /// - Negative: `"not bold"`
    /// - Color: `"red"`, `"#ff0000"`
    /// - Background: `"on red"`, `"on #ff0000"`
    /// - Link: `"link https://..."`
    /// - Combined: `"bold red on white"`
    pub fn parse(style: &str) -> Result<Self, StyleParseError> {
        static CACHE: LazyLock<Mutex<LruCache<String, Style>>> = LazyLock::new(|| {
            Mutex::new(LruCache::new(NonZeroUsize::new(512).expect("non-zero")))
        });

        let normalized = style.trim().to_lowercase();

        if let Ok(mut cache) = CACHE.lock() {
            if let Some(cached) = cache.get(&normalized) {
                return Ok(cached.clone());
            }
        }

        let result = Self::parse_uncached(&normalized)?;

        if let Ok(mut cache) = CACHE.lock() {
            cache.put(normalized, result.clone());
        }

        Ok(result)
    }

    fn parse_uncached(style: &str) -> Result<Self, StyleParseError> {
        if style.is_empty() || style == "none" {
            return Ok(Self::null());
        }

        let mut result = Style::new();
        let words: Vec<&str> = style.split_whitespace().collect();
        let mut i = 0;

        while i < words.len() {
            let word = words[i];

            // Handle "not <attribute>"
            if word == "not" {
                if i + 1 >= words.len() {
                    return Err(StyleParseError::InvalidFormat(
                        "'not' requires an attribute".to_string(),
                    ));
                }
                i += 1;
                let attr_name = words[i];
                if let Some(attr) = parse_attribute(attr_name) {
                    result = result.not(attr);
                } else {
                    return Err(StyleParseError::UnknownAttribute(attr_name.to_string()));
                }
                i += 1;
                continue;
            }

            // Handle "on <color>" for background
            if word == "on" {
                if i + 1 >= words.len() {
                    return Err(StyleParseError::InvalidFormat(
                        "'on' requires a color".to_string(),
                    ));
                }
                i += 1;
                let color_name = words[i];
                result = result.bgcolor_str(color_name)?;
                i += 1;
                continue;
            }

            // Handle "link <url>"
            if word == "link" {
                if i + 1 >= words.len() {
                    return Err(StyleParseError::InvalidFormat(
                        "'link' requires a URL".to_string(),
                    ));
                }
                i += 1;
                result = result.link(words[i]);
                i += 1;
                continue;
            }

            // Try as attribute
            if let Some(attr) = parse_attribute(word) {
                match attr {
                    Attributes::BOLD => result = result.bold(),
                    Attributes::DIM => result = result.dim(),
                    Attributes::ITALIC => result = result.italic(),
                    Attributes::UNDERLINE => result = result.underline(),
                    Attributes::BLINK => result = result.blink(),
                    Attributes::REVERSE => result = result.reverse(),
                    Attributes::CONCEAL => result = result.conceal(),
                    Attributes::STRIKE => result = result.strike(),
                    Attributes::OVERLINE => result = result.overline(),
                    // Attributes without dedicated builder methods
                    Attributes::BLINK2
                    | Attributes::UNDERLINE2
                    | Attributes::FRAME
                    | Attributes::ENCIRCLE => {
                        result.attributes.insert(attr);
                        result.set_attributes.insert(attr);
                        result.null = false;
                    }
                    _ => {}
                }
                i += 1;
                continue;
            }

            // Try as foreground color
            if Color::parse(word).is_ok() {
                result = result.color_str(word)?;
                i += 1;
                continue;
            }

            return Err(StyleParseError::UnknownToken(word.to_string()));
        }

        Ok(result)
    }
}

impl std::ops::Add for Style {
    type Output = Style;

    fn add(self, rhs: Self) -> Self::Output {
        self.combine(&rhs)
    }
}

impl std::ops::Add<&Style> for Style {
    type Output = Style;

    fn add(self, rhs: &Self) -> Self::Output {
        self.combine(rhs)
    }
}

impl std::ops::Add<Style> for &Style {
    type Output = Style;

    fn add(self, rhs: Style) -> Self::Output {
        self.combine(&rhs)
    }
}

impl std::ops::Add<&Style> for &Style {
    type Output = Style;

    fn add(self, rhs: &Style) -> Self::Output {
        self.combine(rhs)
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            return write!(f, "none");
        }

        let mut parts = Vec::new();

        // Add attributes
        for (attr, name) in [
            (Attributes::BOLD, "bold"),
            (Attributes::DIM, "dim"),
            (Attributes::ITALIC, "italic"),
            (Attributes::UNDERLINE, "underline"),
            (Attributes::BLINK, "blink"),
            (Attributes::REVERSE, "reverse"),
            (Attributes::CONCEAL, "conceal"),
            (Attributes::STRIKE, "strike"),
            (Attributes::OVERLINE, "overline"),
        ] {
            if self.attributes.contains(attr) {
                parts.push(name.to_string());
            }
        }

        // Add foreground color
        if let Some(color) = &self.color {
            parts.push(color.to_string());
        }

        // Add background color
        if let Some(bgcolor) = &self.bgcolor {
            parts.push(format!("on {bgcolor}"));
        }

        // Add link
        if let Some(link) = &self.link {
            parts.push(format!("link {link}"));
        }

        write!(f, "{}", parts.join(" "))
    }
}

impl FromStr for Style {
    type Err = StyleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for Style {
    type Error = StyleParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<String> for Style {
    type Error = StyleParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}

impl From<Color> for Style {
    fn from(color: Color) -> Self {
        Self::new().color(color)
    }
}

impl From<ColorTriplet> for Style {
    fn from(triplet: ColorTriplet) -> Self {
        Self::new().color(Color::from(triplet))
    }
}

impl From<(u8, u8, u8)> for Style {
    fn from((red, green, blue): (u8, u8, u8)) -> Self {
        Self::new().color(Color::from((red, green, blue)))
    }
}

impl From<[u8; 3]> for Style {
    fn from([red, green, blue]: [u8; 3]) -> Self {
        Self::new().color(Color::from((red, green, blue)))
    }
}

/// Parse an attribute name to its flag.
fn parse_attribute(name: &str) -> Option<Attributes> {
    match name {
        "bold" | "b" => Some(Attributes::BOLD),
        "dim" | "d" => Some(Attributes::DIM),
        "italic" | "i" => Some(Attributes::ITALIC),
        "underline" | "u" => Some(Attributes::UNDERLINE),
        "blink" => Some(Attributes::BLINK),
        "blink2" => Some(Attributes::BLINK2),
        "reverse" | "r" => Some(Attributes::REVERSE),
        "conceal" | "c" => Some(Attributes::CONCEAL),
        "strike" | "s" => Some(Attributes::STRIKE),
        "underline2" | "uu" => Some(Attributes::UNDERLINE2),
        "frame" => Some(Attributes::FRAME),
        "encircle" => Some(Attributes::ENCIRCLE),
        "overline" | "o" => Some(Attributes::OVERLINE),
        _ => None,
    }
}

/// Style stack for nested style application.
#[derive(Debug, Clone)]
pub struct StyleStack {
    stack: Vec<Style>,
}

impl StyleStack {
    /// Create a new style stack with a default base style.
    #[must_use]
    pub fn new(default: Style) -> Self {
        Self {
            stack: vec![default],
        }
    }

    /// Get the current combined style.
    #[must_use]
    pub fn current(&self) -> &Style {
        self.stack.last().expect("stack should never be empty")
    }

    /// Push a new style onto the stack, combining with current.
    pub fn push(&mut self, style: Style) {
        let combined = self.current().combine(&style);
        self.stack.push(combined);
    }

    /// Pop the most recent style from the stack.
    pub fn pop(&mut self) -> &Style {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
        self.current()
    }

    /// Get the depth of the stack.
    #[must_use]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Check if the stack is empty (only base style).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stack.len() <= 1
    }
}

impl Default for StyleStack {
    fn default() -> Self {
        Self::new(Style::null())
    }
}

/// Error type for style parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleParseError {
    InvalidFormat(String),
    UnknownAttribute(String),
    UnknownToken(String),
    ColorError(ColorParseError),
}

impl fmt::Display for StyleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat(s) => write!(f, "Invalid style format: {s}"),
            Self::UnknownAttribute(s) => write!(f, "Unknown attribute: {s}"),
            Self::UnknownToken(s) => write!(f, "Unknown token: {s}"),
            Self::ColorError(e) => write!(f, "Color error: {e}"),
        }
    }
}

impl std::error::Error for StyleParseError {}

impl From<ColorParseError> for StyleParseError {
    fn from(err: ColorParseError) -> Self {
        Self::ColorError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attributes_sgr_codes() {
        let attrs = Attributes::BOLD | Attributes::ITALIC;
        let codes = attrs.to_sgr_codes();
        assert!(codes.contains(&1));
        assert!(codes.contains(&3));
    }

    #[test]
    fn test_style_null() {
        let style = Style::null();
        assert!(style.is_null());
    }

    #[test]
    fn test_style_builder() {
        let style = Style::new()
            .bold()
            .italic()
            .color(Color::from_ansi(1));

        assert!(style.attributes.contains(Attributes::BOLD));
        assert!(style.attributes.contains(Attributes::ITALIC));
        assert!(style.color.is_some());
    }

    #[test]
    fn test_style_combine() {
        let style1 = Style::new().bold().color(Color::from_ansi(1));
        let style2 = Style::new().italic().color(Color::from_ansi(2));

        let combined = style1.combine(&style2);

        assert!(combined.attributes.contains(Attributes::BOLD));
        assert!(combined.attributes.contains(Attributes::ITALIC));
        // style2's color should override
        assert_eq!(combined.color.unwrap().number, Some(2));
    }

    #[test]
    fn test_style_combine_null() {
        let style = Style::new().bold();
        let null = Style::null();

        assert_eq!(style.combine(&null), style);
        assert_eq!(null.combine(&style), style);
    }

    #[test]
    fn test_style_parse_simple() {
        let style = Style::parse("bold").unwrap();
        assert!(style.attributes.contains(Attributes::BOLD));
    }

    #[test]
    fn test_style_parse_color() {
        let style = Style::parse("red").unwrap();
        assert!(style.color.is_some());
    }

    #[test]
    fn test_style_parse_background() {
        let style = Style::parse("on blue").unwrap();
        assert!(style.bgcolor.is_some());
    }

    #[test]
    fn test_style_parse_combined() {
        let style = Style::parse("bold red on white").unwrap();
        assert!(style.attributes.contains(Attributes::BOLD));
        assert!(style.color.is_some());
        assert!(style.bgcolor.is_some());
    }

    #[test]
    fn test_style_parse_not() {
        let style = Style::parse("not bold").unwrap();
        assert!(style.set_attributes.contains(Attributes::BOLD));
        assert!(!style.attributes.contains(Attributes::BOLD));
    }

    #[test]
    fn test_style_parse_link() {
        let style = Style::parse("link https://example.com").unwrap();
        assert_eq!(style.link, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_style_render() {
        let style = Style::new().bold();
        let rendered = style.render("test", ColorSystem::TrueColor);
        assert!(rendered.contains("\x1b[1m"));
        assert!(rendered.contains("\x1b[0m"));
    }

    #[test]
    fn test_style_stack() {
        let mut stack = StyleStack::new(Style::null());

        stack.push(Style::new().bold());
        assert!(stack.current().attributes.contains(Attributes::BOLD));

        stack.push(Style::new().italic());
        assert!(stack.current().attributes.contains(Attributes::BOLD));
        assert!(stack.current().attributes.contains(Attributes::ITALIC));

        stack.pop();
        assert!(stack.current().attributes.contains(Attributes::BOLD));
        assert!(!stack.current().attributes.contains(Attributes::ITALIC));
    }

    #[test]
    fn test_style_add_operator() {
        let s1 = Style::new().bold();
        let s2 = Style::new().italic();
        let combined = s1 + s2;

        assert!(combined.attributes.contains(Attributes::BOLD));
        assert!(combined.attributes.contains(Attributes::ITALIC));
    }
}
