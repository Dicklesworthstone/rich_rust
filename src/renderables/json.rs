//! JSON - Pretty-printed JSON with syntax highlighting.
//!
//! This module provides a JSON renderable for rendering JSON data
//! with syntax highlighting and configurable formatting. It uses semantic
//! coloring to distinguish keys, strings, numbers, booleans, and null values.
//!
//! # Feature Flag
//!
//! This module requires the `json` feature to be enabled:
//!
//! ```toml
//! [dependencies]
//! rich_rust = { version = "0.1", features = ["json"] }
//! ```
//!
//! Or enable all optional features with:
//!
//! ```toml
//! rich_rust = { version = "0.1", features = ["full"] }
//! ```
//!
//! # Dependencies
//!
//! Enabling this feature adds the [`serde_json`](https://docs.rs/serde_json) crate
//! as a dependency for JSON parsing and value representation.
//!
//! # Basic Usage
//!
//! ```rust,ignore
//! use rich_rust::renderables::json::Json;
//!
//! // From a JSON string
//! let data = r#"{"name": "Alice", "age": 30, "active": true}"#;
//! let json = Json::from_str(data).unwrap();
//! let segments = json.render();
//!
//! // From a serde_json::Value
//! use serde_json::json;
//! let json = Json::new(json!({"key": "value"}));
//! ```
//!
//! # Indentation Options
//!
//! ```rust,ignore
//! use rich_rust::renderables::json::Json;
//!
//! let json = Json::from_str(r#"{"a": [1, 2, 3]}"#).unwrap()
//!     .indent(4);  // Use 4-space indentation (default is 2)
//! ```
//!
//! # Sorting Keys
//!
//! ```rust,ignore
//! use rich_rust::renderables::json::Json;
//!
//! // Keys will appear in alphabetical order
//! let json = Json::from_str(r#"{"z": 1, "a": 2, "m": 3}"#).unwrap()
//!     .sort_keys(true);
//! ```
//!
//! # Custom Themes
//!
//! The default theme uses semantic colors:
//! - **Keys**: Blue, bold
//! - **Strings**: Green
//! - **Numbers**: Cyan
//! - **Booleans**: Yellow
//! - **Null**: Magenta, italic
//! - **Brackets/braces**: White
//! - **Punctuation**: White
//!
//! You can customize the theme:
//!
//! ```rust,ignore
//! use rich_rust::renderables::json::{Json, JsonTheme};
//! use rich_rust::style::Style;
//!
//! let theme = JsonTheme {
//!     key: Style::new().bold().color_str("red").unwrap(),
//!     string: Style::new().color_str("blue").unwrap(),
//!     number: Style::new().color_str("green").unwrap(),
//!     boolean: Style::new().color_str("yellow").unwrap(),
//!     null: Style::new().color_str("white").unwrap(),
//!     bracket: Style::new().color_str("cyan").unwrap(),
//!     punctuation: Style::new().color_str("magenta").unwrap(),
//! };
//!
//! let json = Json::from_str(r#"{"key": "value"}"#).unwrap()
//!     .theme(theme);
//! ```
//!
//! # Disabling Highlighting
//!
//! ```rust,ignore
//! use rich_rust::renderables::json::Json;
//!
//! // Render without colors (plain text)
//! let json = Json::from_str(r#"{"key": "value"}"#).unwrap()
//!     .highlight(false);
//! ```
//!
//! # Plain Text Output
//!
//! ```rust,ignore
//! use rich_rust::renderables::json::Json;
//!
//! let json = Json::from_str(r#"{"key": "value"}"#).unwrap();
//! let plain = json.to_plain_string();  // Get formatted JSON without ANSI codes
//! ```
//!
//! # Known Limitations
//!
//! - **Large JSON**: Very large JSON documents may be slow to render due to
//!   per-token segment creation
//! - **Streaming**: Does not support streaming JSON parsing; the entire document
//!   must fit in memory
//! - **Compact output**: No option for compact (non-pretty-printed) output
//! - **Trailing commas**: Standard JSON only; no trailing comma support

use std::fmt::Write as _;

use serde_json::Value;

use crate::segment::Segment;
use crate::style::Style;

/// Default theme colors for JSON syntax highlighting.
#[derive(Debug, Clone)]
pub struct JsonTheme {
    /// Style for object/array keys.
    pub key: Style,
    /// Style for string values.
    pub string: Style,
    /// Style for number values.
    pub number: Style,
    /// Style for boolean values (true/false).
    pub boolean: Style,
    /// Style for null values.
    pub null: Style,
    /// Style for brackets and braces.
    pub bracket: Style,
    /// Style for colons and commas.
    pub punctuation: Style,
}

impl Default for JsonTheme {
    fn default() -> Self {
        Self {
            key: Style::new().color_str("blue").unwrap_or_default().bold(),
            string: Style::new().color_str("green").unwrap_or_default(),
            number: Style::new().color_str("cyan").unwrap_or_default().bold(),
            boolean: Style::new().color_str("yellow").unwrap_or_default().bold(),
            null: Style::new()
                .color_str("magenta")
                .unwrap_or_default()
                .italic(),
            bracket: Style::new().bold(),
            punctuation: Style::new(),
        }
    }
}

/// A renderable for JSON data with syntax highlighting.
#[derive(Debug, Clone)]
pub struct Json {
    /// The JSON value to render.
    value: Value,
    /// Number of spaces for indentation.
    indent: usize,
    /// Whether to sort object keys alphabetically.
    sort_keys: bool,
    /// Whether to apply syntax highlighting.
    highlight: bool,
    /// Theme for syntax highlighting.
    theme: JsonTheme,
}

impl Json {
    /// Create a new Json renderable from a `serde_json::Value`.
    #[must_use]
    pub fn new(value: Value) -> Self {
        Self {
            value,
            indent: 2,
            sort_keys: false,
            highlight: true,
            theme: JsonTheme::default(),
        }
    }

    /// Create a Json renderable from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not valid JSON.
    #[expect(
        clippy::should_implement_trait,
        reason = "returns Result with custom error, not FromStr pattern"
    )]
    pub fn from_str(s: &str) -> Result<Self, JsonError> {
        let value: Value = serde_json::from_str(s).map_err(JsonError::Parse)?;
        Ok(Self::new(value))
    }

    /// Set the number of spaces for indentation.
    #[must_use]
    pub fn indent(mut self, spaces: usize) -> Self {
        self.indent = spaces;
        self
    }

    /// Set whether to sort object keys alphabetically.
    #[must_use]
    pub fn sort_keys(mut self, sort: bool) -> Self {
        self.sort_keys = sort;
        self
    }

    /// Set whether to apply syntax highlighting.
    #[must_use]
    pub fn highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
    }

    /// Set a custom theme for syntax highlighting.
    #[must_use]
    pub fn theme(mut self, theme: JsonTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Get a style, or no style if highlighting is disabled.
    fn style(&self, style: &Style) -> Option<Style> {
        if self.highlight {
            Some(style.clone())
        } else {
            None
        }
    }

    /// Render a JSON value at the given depth.
    fn render_value(&self, value: &Value, depth: usize) -> Vec<Segment<'_>> {
        match value {
            Value::Null => vec![Segment::new("null", self.style(&self.theme.null))],
            Value::Bool(b) => {
                let text = if *b { "true" } else { "false" };
                vec![Segment::new(text, self.style(&self.theme.boolean))]
            }
            Value::Number(n) => {
                vec![Segment::new(n.to_string(), self.style(&self.theme.number))]
            }
            Value::String(s) => {
                // Escape and quote the string
                let escaped = escape_json_string(s);
                vec![Segment::new(
                    format!("\"{escaped}\""),
                    self.style(&self.theme.string),
                )]
            }
            Value::Array(arr) => self.render_array(arr, depth),
            Value::Object(obj) => self.render_object(obj, depth),
        }
    }

    /// Render an array.
    fn render_array(&self, arr: &[Value], depth: usize) -> Vec<Segment<'_>> {
        const MAX_DEPTH: usize = 20;
        if depth > MAX_DEPTH {
            return vec![Segment::new("[...]", self.style(&self.theme.bracket))];
        }

        if arr.is_empty() {
            return vec![Segment::new("[]", self.style(&self.theme.bracket))];
        }

        let mut segments = Vec::new();
        let indent_str = " ".repeat(self.indent * (depth + 1));
        let close_indent = " ".repeat(self.indent * depth);

        // Opening bracket
        segments.push(Segment::new("[", self.style(&self.theme.bracket)));
        segments.push(Segment::new("\n", None));

        for (i, item) in arr.iter().enumerate() {
            // Indent
            segments.push(Segment::new(indent_str.clone(), None));

            // Value
            segments.extend(self.render_value(item, depth + 1));

            // Comma (except for last item)
            if i < arr.len() - 1 {
                segments.push(Segment::new(",", self.style(&self.theme.punctuation)));
            }
            segments.push(Segment::new("\n", None));
        }

        // Closing bracket
        segments.push(Segment::new(close_indent, None));
        segments.push(Segment::new("]", self.style(&self.theme.bracket)));

        segments
    }

    /// Render an object.
    fn render_object(
        &self,
        obj: &serde_json::Map<String, Value>,
        depth: usize,
    ) -> Vec<Segment<'_>> {
        const MAX_DEPTH: usize = 20;
        if depth > MAX_DEPTH {
            return vec![Segment::new("{...}", self.style(&self.theme.bracket))];
        }

        if obj.is_empty() {
            return vec![Segment::new("{}", self.style(&self.theme.bracket))];
        }

        let mut segments = Vec::new();
        let indent_str = " ".repeat(self.indent * (depth + 1));
        let close_indent = " ".repeat(self.indent * depth);

        // Get keys, optionally sorted
        let keys: Vec<&String> = if self.sort_keys {
            let mut k: Vec<_> = obj.keys().collect();
            k.sort();
            k
        } else {
            obj.keys().collect()
        };

        // Opening brace
        segments.push(Segment::new("{", self.style(&self.theme.bracket)));
        segments.push(Segment::new("\n", None));

        for (i, key) in keys.iter().enumerate() {
            let value = &obj[*key];

            // Indent
            segments.push(Segment::new(indent_str.clone(), None));

            // Key (quoted)
            let escaped_key = escape_json_string(key);
            segments.push(Segment::new(
                format!("\"{escaped_key}\""),
                self.style(&self.theme.key),
            ));

            // Colon
            segments.push(Segment::new(": ", self.style(&self.theme.punctuation)));

            // Value
            segments.extend(self.render_value(value, depth + 1));

            // Comma (except for last item)
            if i < keys.len() - 1 {
                segments.push(Segment::new(",", self.style(&self.theme.punctuation)));
            }
            segments.push(Segment::new("\n", None));
        }

        // Closing brace
        segments.push(Segment::new(close_indent, None));
        segments.push(Segment::new("}", self.style(&self.theme.bracket)));

        segments
    }

    /// Render the JSON to segments.
    #[must_use]
    pub fn render(&self) -> Vec<Segment<'_>> {
        self.render_value(&self.value, 0)
    }

    /// Render to a plain string without ANSI codes.
    #[must_use]
    pub fn to_plain_string(&self) -> String {
        self.render().iter().map(|s| s.text.as_ref()).collect()
    }
}

/// Escape special characters in a JSON string.
fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(result, "\\u{:04x}", c as u32);
            }
            c => result.push(c),
        }
    }
    result
}

/// Error type for JSON parsing.
#[derive(Debug)]
pub enum JsonError {
    /// JSON parsing error.
    Parse(serde_json::Error),
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "JSON parse error: {e}"),
        }
    }
}

impl std::error::Error for JsonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(e) => Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_null() {
        let json = Json::new(Value::Null);
        let segments = json.render();
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();
        assert_eq!(text, "null");
    }

    #[test]
    fn test_json_bool_true() {
        let json = Json::new(Value::Bool(true));
        let segments = json.render();
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();
        assert_eq!(text, "true");
    }

    #[test]
    fn test_json_bool_false() {
        let json = Json::new(Value::Bool(false));
        let segments = json.render();
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();
        assert_eq!(text, "false");
    }

    #[test]
    fn test_json_number_int() {
        let json = Json::new(serde_json::json!(42));
        let segments = json.render();
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();
        assert_eq!(text, "42");
    }

    #[test]
    fn test_json_number_float() {
        let json = Json::new(serde_json::json!(1.23));
        let segments = json.render();
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();
        assert_eq!(text, "1.23");
    }

    #[test]
    fn test_json_string() {
        let json = Json::new(serde_json::json!("hello"));
        let segments = json.render();
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();
        assert_eq!(text, "\"hello\"");
    }

    #[test]
    fn test_json_string_escaped() {
        let json = Json::new(serde_json::json!("line1\nline2"));
        let segments = json.render();
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();
        assert_eq!(text, "\"line1\\nline2\"");
    }

    #[test]
    fn test_json_empty_array() {
        let json = Json::new(serde_json::json!([]));
        let segments = json.render();
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();
        assert_eq!(text, "[]");
    }

    #[test]
    fn test_json_simple_array() {
        let json = Json::new(serde_json::json!([1, 2, 3])).indent(2);
        let text = json.to_plain_string();
        assert!(text.contains("[\n"));
        assert!(text.contains("  1"));
        assert!(text.contains("  2"));
        assert!(text.contains("  3"));
        assert!(text.contains(']'));
    }

    #[test]
    fn test_json_empty_object() {
        let json = Json::new(serde_json::json!({}));
        let segments = json.render();
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();
        assert_eq!(text, "{}");
    }

    #[test]
    fn test_json_simple_object() {
        let json = Json::new(serde_json::json!({"name": "Alice"})).indent(2);
        let text = json.to_plain_string();
        assert!(text.contains("{\n"));
        assert!(text.contains("\"name\""));
        assert!(text.contains(": \"Alice\""));
        assert!(text.contains('}'));
    }

    #[test]
    fn test_json_nested_object() {
        let json = Json::new(serde_json::json!({
            "person": {
                "name": "Alice",
                "age": 30
            }
        }))
        .indent(2);
        let text = json.to_plain_string();
        assert!(text.contains("\"person\""));
        assert!(text.contains("\"name\""));
        assert!(text.contains("\"Alice\""));
        assert!(text.contains("\"age\""));
        assert!(text.contains("30"));
    }

    #[test]
    fn test_json_from_str() {
        let json = Json::from_str(r#"{"key": "value"}"#).unwrap();
        let text = json.to_plain_string();
        assert!(text.contains("\"key\""));
        assert!(text.contains("\"value\""));
    }

    #[test]
    fn test_json_from_str_invalid() {
        let result = Json::from_str("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_sort_keys() {
        let json = Json::new(serde_json::json!({"z": 1, "a": 2, "m": 3})).sort_keys(true);
        let text = json.to_plain_string();

        // Find positions of keys
        let pos_a = text.find("\"a\"").unwrap();
        let pos_m = text.find("\"m\"").unwrap();
        let pos_z = text.find("\"z\"").unwrap();

        // Keys should appear in sorted order
        assert!(pos_a < pos_m);
        assert!(pos_m < pos_z);
    }

    #[test]
    fn test_json_no_highlight() {
        let json = Json::new(serde_json::json!("test")).highlight(false);
        let segments = json.render();
        // Without highlighting, styles should be None
        assert!(segments.iter().all(|s| s.style.is_none()));
    }

    #[test]
    fn test_json_with_highlight() {
        let json = Json::new(serde_json::json!("test")).highlight(true);
        let segments = json.render();
        // With highlighting, string should have a style
        assert!(segments.iter().any(|s| s.style.is_some()));
    }

    #[test]
    fn test_json_custom_indent() {
        let json = Json::new(serde_json::json!([1])).indent(4);
        let text = json.to_plain_string();
        // Should have 4-space indentation
        assert!(text.contains("    1"));
    }

    #[test]
    fn test_json_mixed_array() {
        let json = Json::new(serde_json::json!([1, "two", true, null]));
        let text = json.to_plain_string();
        assert!(text.contains('1'));
        assert!(text.contains("\"two\""));
        assert!(text.contains("true"));
        assert!(text.contains("null"));
    }

    #[test]
    fn test_json_complex() {
        let json = Json::new(serde_json::json!({
            "users": [
                {"name": "Alice", "active": true},
                {"name": "Bob", "active": false}
            ],
            "count": 2,
            "meta": null
        }))
        .sort_keys(true);

        let text = json.to_plain_string();
        assert!(text.contains("\"users\""));
        assert!(text.contains("\"count\""));
        assert!(text.contains("\"meta\""));
        assert!(text.contains("\"Alice\""));
        assert!(text.contains("\"Bob\""));
        assert!(text.contains("true"));
        assert!(text.contains("false"));
        assert!(text.contains("null"));
        assert!(text.contains('2'));
    }

    #[test]
    fn test_escape_json_string() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_json_string("a\\b"), "a\\\\b");
        assert_eq!(escape_json_string("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_json_string("tab\there"), "tab\\there");
    }

    #[test]
    fn test_json_custom_theme() {
        let theme = JsonTheme {
            key: Style::new().color_str("red").unwrap_or_default(),
            string: Style::new().color_str("blue").unwrap_or_default(),
            number: Style::new().color_str("green").unwrap_or_default(),
            boolean: Style::new().color_str("yellow").unwrap_or_default(),
            null: Style::new().color_str("white").unwrap_or_default(),
            bracket: Style::new().color_str("cyan").unwrap_or_default(),
            punctuation: Style::new().color_str("magenta").unwrap_or_default(),
        };

        let json = Json::new(serde_json::json!({"key": "value"})).theme(theme);
        let segments = json.render();

        // Should have styled segments
        assert!(segments.iter().any(|s| s.style.is_some()));
    }
}
