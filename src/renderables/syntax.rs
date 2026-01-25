//! Syntax highlighting renderable.
//!
//! This module provides syntax highlighting for code using the syntect library.
//! It supports numerous programming languages and themes out of the box.
//!
//! # Feature Flag
//!
//! This module requires the `syntax` feature to be enabled:
//!
//! ```toml
//! [dependencies]
//! rich_rust = { version = "0.1", features = ["syntax"] }
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
//! Enabling this feature adds the [`syntect`](https://docs.rs/syntect) crate as a dependency,
//! which provides the underlying syntax definitions and theme support.
//!
//! # Basic Usage
//!
//! ```rust,ignore
//! use rich_rust::renderables::syntax::Syntax;
//!
//! // Create a syntax-highlighted code block
//! let code = r#"fn main() { println!("Hello"); }"#;
//! let syntax = Syntax::new(code, "rust");
//! let segments = syntax.render(None)?;
//! ```
//!
//! # Line Numbers and Themes
//!
//! ```rust,ignore
//! use rich_rust::renderables::syntax::Syntax;
//!
//! let code = "def hello():\n    print('world')";
//! let syntax = Syntax::new(code, "python")
//!     .line_numbers(true)
//!     .start_line(10)  // Start numbering from line 10
//!     .theme("base16-ocean.dark");
//!
//! let segments = syntax.render(None)?;
//! ```
//!
//! # Loading from Files
//!
//! ```rust,ignore
//! use rich_rust::renderables::syntax::Syntax;
//!
//! // Auto-detect language from file extension
//! let syntax = Syntax::from_path("src/main.rs")?
//!     .line_numbers(true)
//!     .theme("InspiredGitHub");
//! ```
//!
//! # Available Themes
//!
//! Call [`Syntax::available_themes()`] to list all built-in themes. Common themes include:
//! - `base16-ocean.dark` (default)
//! - `base16-ocean.light`
//! - `InspiredGitHub`
//! - `Solarized (dark)`
//! - `Solarized (light)`
//!
//! # Supported Languages
//!
//! Call [`Syntax::available_languages()`] to list all supported languages. Syntect includes
//! syntax definitions for 100+ languages including Rust, Python, JavaScript, TypeScript,
//! Go, Java, C/C++, Ruby, and many more.
//!
//! # Known Limitations
//!
//! - **Theme loading**: Only the default syntect themes are available. Custom `.tmTheme` files
//!   are not currently supported.
//! - **Syntax definitions**: Only default syntect syntax definitions are available. Custom
//!   `.sublime-syntax` files are not currently supported.
//! - **Large files**: Rendering very large files may be slow due to per-line highlighting.
//! - **Word wrap**: The `word_wrap` option is defined but not yet fully implemented.

use crate::cells;
use crate::color::Color;
use crate::segment::Segment;
use crate::style::Style;

use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Error type for syntax highlighting operations.
#[derive(Debug, Clone)]
pub enum SyntaxError {
    /// The specified language is not supported.
    UnknownLanguage(String),
    /// The specified theme is not found.
    UnknownTheme(String),
    /// Failed to read the file.
    IoError(String),
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownLanguage(lang) => write!(f, "Unknown language: {lang}"),
            Self::UnknownTheme(theme) => write!(f, "Unknown theme: {theme}"),
            Self::IoError(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for SyntaxError {}

/// A syntax-highlighted code block renderable.
///
/// Uses syntect for syntax highlighting with support for themes,
/// line numbers, and background colors.
#[derive(Debug, Clone)]
pub struct Syntax {
    /// The source code to highlight.
    code: String,
    /// The language for syntax highlighting.
    language: String,
    /// Whether to show line numbers.
    line_numbers: bool,
    /// The starting line number (for excerpts).
    start_line: usize,
    /// The theme name to use.
    theme_name: String,
    /// Optional background color override.
    background_color: Option<Color>,
    /// Whether to show indentation guides.
    indent_guides: bool,
    /// Tab size for rendering.
    tab_size: usize,
    /// Optional word wrap width.
    word_wrap: Option<usize>,
    /// Style for the line number column.
    line_number_style: Style,
    /// Padding around the code block.
    padding: (usize, usize),
}

impl Default for Syntax {
    fn default() -> Self {
        Self {
            code: String::new(),
            language: String::from("text"),
            line_numbers: false,
            start_line: 1,
            theme_name: String::from("base16-ocean.dark"),
            background_color: None,
            indent_guides: false,
            tab_size: 4,
            word_wrap: None,
            line_number_style: Style::new().color_str("bright_black").unwrap_or_default(),
            padding: (0, 0),
        }
    }
}

impl Syntax {
    /// Create a new syntax highlighted code block.
    ///
    /// # Arguments
    ///
    /// * `code` - The source code to highlight
    /// * `language` - The programming language (e.g., "rust", "python", "javascript")
    #[must_use]
    pub fn new(code: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            language: language.into(),
            ..Default::default()
        }
    }

    /// Load syntax from a file path, auto-detecting the language.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, SyntaxError> {
        let path = path.as_ref();
        let code = fs::read_to_string(path).map_err(|e| SyntaxError::IoError(e.to_string()))?;

        // Auto-detect language from extension
        let language = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map_or_else(|| String::from("text"), Self::extension_to_language);

        Ok(Self::new(code, language))
    }

    /// Map file extension to language name.
    fn extension_to_language(ext: &str) -> String {
        match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "jsx" => "javascript",
            "tsx" => "typescript",
            "rb" => "ruby",
            "go" => "go",
            "java" => "java",
            "c" => "c",
            "cpp" | "cxx" | "cc" => "c++",
            "h" | "hpp" => "c++",
            "cs" => "c#",
            "php" => "php",
            "swift" => "swift",
            "kt" | "kts" => "kotlin",
            "scala" => "scala",
            "sh" | "bash" => "bash",
            "zsh" => "zsh",
            "fish" => "fish",
            "ps1" => "powershell",
            "sql" => "sql",
            "html" | "htm" => "html",
            "css" => "css",
            "scss" => "scss",
            "less" => "less",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "xml" => "xml",
            "md" | "markdown" => "markdown",
            "r" => "r",
            "lua" => "lua",
            "perl" | "pl" => "perl",
            "vim" => "vim",
            "dockerfile" => "dockerfile",
            "makefile" => "makefile",
            _ => ext,
        }
        .to_string()
    }

    /// Enable or disable line numbers.
    #[must_use]
    pub fn line_numbers(mut self, enabled: bool) -> Self {
        self.line_numbers = enabled;
        self
    }

    /// Set the starting line number (useful for code excerpts).
    #[must_use]
    pub fn start_line(mut self, line: usize) -> Self {
        self.start_line = line.max(1);
        self
    }

    /// Set the theme for syntax highlighting.
    ///
    /// Common themes: "base16-ocean.dark", "base16-ocean.light",
    /// `InspiredGitHub`, `Solarized (dark)`, `Solarized (light)`
    #[must_use]
    pub fn theme(mut self, theme_name: impl Into<String>) -> Self {
        self.theme_name = theme_name.into();
        self
    }

    /// Override the background color.
    #[must_use]
    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Enable or disable indentation guides.
    #[must_use]
    pub fn indent_guides(mut self, enabled: bool) -> Self {
        self.indent_guides = enabled;
        self
    }

    /// Set the tab size.
    #[must_use]
    pub fn tab_size(mut self, size: usize) -> Self {
        self.tab_size = size.max(1);
        self
    }

    /// Set word wrap width.
    #[must_use]
    pub fn word_wrap(mut self, width: Option<usize>) -> Self {
        self.word_wrap = width;
        self
    }

    /// Set the style for line numbers.
    #[must_use]
    pub fn line_number_style(mut self, style: Style) -> Self {
        self.line_number_style = style;
        self
    }

    /// Set padding around the code block (top/bottom, left/right).
    #[must_use]
    pub fn padding(mut self, vertical: usize, horizontal: usize) -> Self {
        self.padding = (vertical, horizontal);
        self
    }

    /// Get the list of available themes.
    #[must_use]
    pub fn available_themes() -> Vec<String> {
        THEME_SET.themes.keys().cloned().collect()
    }

    /// Get the list of available languages.
    #[must_use]
    pub fn available_languages() -> Vec<String> {
        SYNTAX_SET
            .syntaxes()
            .iter()
            .map(|s| s.name.clone())
            .collect()
    }

    /// Render the syntax-highlighted code to segments.
    ///
    /// # Errors
    ///
    /// Returns an error if the theme or language is not found.
    pub fn render(&self, max_width: Option<usize>) -> Result<Vec<Segment<'_>>, SyntaxError> {
        let ps = &*SYNTAX_SET;
        let ts = &*THEME_SET;

        // Find the syntax definition
        let syntax = ps
            .find_syntax_by_token(&self.language)
            .or_else(|| ps.find_syntax_by_extension(&self.language))
            .ok_or_else(|| SyntaxError::UnknownLanguage(self.language.clone()))?;

        // Get the theme
        let theme = ts
            .themes
            .get(&self.theme_name)
            .ok_or_else(|| SyntaxError::UnknownTheme(self.theme_name.clone()))?;

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut segments = Vec::new();

        // Calculate line number width
        let line_count = self.code.lines().count();
        let last_line = self.start_line.saturating_add(line_count.saturating_sub(1));
        let line_num_width = last_line.to_string().len();

        // Add top padding
        for _ in 0..self.padding.0 {
            segments.push(Segment::new("\n", None));
        }

        // Process each line
        for (idx, line) in LinesWithEndings::from(&self.code).enumerate() {
            let line_num = self.start_line + idx;

            // Add horizontal padding
            if self.padding.1 > 0 {
                segments.push(Segment::new(" ".repeat(self.padding.1), None));
            }

            // Add line number if enabled
            if self.line_numbers {
                let num_str = format!("{line_num:>line_num_width$} │ ");
                segments.push(Segment::new(num_str, Some(self.line_number_style.clone())));
            }

            // Expand tabs
            let line_expanded = line
                .replace('\t', &" ".repeat(self.tab_size))
                .replace("\r\n", "\n");

            // Add indentation guides if enabled
            if self.indent_guides {
                let leading_spaces = line_expanded.len() - line_expanded.trim_start().len();
                let guide_positions: Vec<usize> =
                    (0..leading_spaces).step_by(self.tab_size).skip(1).collect();

                if !guide_positions.is_empty() {
                    // We'll handle guides during highlight processing
                    // For now, just highlight the line
                }
            }

            // Highlight the line
            let ranges = highlighter
                .highlight_line(&line_expanded, ps)
                .unwrap_or_else(|_| {
                    vec![(
                        syntect::highlighting::Style::default(),
                        line_expanded.as_str(),
                    )]
                });

            let mut saw_newline = false;
            for (style, text) in ranges {
                let rich_style = self.syntect_style_to_rich(style, theme);
                if text.contains('\n') {
                    let mut parts = text.split('\n').peekable();
                    while let Some(part) = parts.next() {
                        if !part.is_empty() {
                            // Clone string to own it (Segment<'static>)
                            segments.push(Segment::new(part.to_string(), Some(rich_style.clone())));
                        }
                        if parts.peek().is_some() {
                            if self.padding.1 > 0 {
                                segments.push(Segment::new(" ".repeat(self.padding.1), None));
                            }
                            segments.push(Segment::new("\n", None));
                            saw_newline = true;
                        }
                    }
                } else if !text.is_empty() {
                    // Clone string to own it
                    segments.push(Segment::new(text.to_string(), Some(rich_style)));
                }
            }

            // Add horizontal padding at end
            if self.padding.1 > 0 && !saw_newline {
                segments.push(Segment::new(" ".repeat(self.padding.1), None));
            }
        }

        // Add bottom padding
        for _ in 0..self.padding.0 {
            segments.push(Segment::new("\n", None));
        }

        if let Some(width) = max_width.filter(|value| *value > 0) {
            Ok(pad_segments_to_width(segments, width))
        } else {
            Ok(segments)
        }
    }

    /// Convert syntect style to rich Style.
    fn syntect_style_to_rich(&self, style: syntect::highlighting::Style, theme: &Theme) -> Style {
        let fg = Color::from_rgb(style.foreground.r, style.foreground.g, style.foreground.b);

        let bg = if let Some(ref override_bg) = self.background_color {
            override_bg.clone()
        } else {
            let bg_color = theme
                .settings
                .background
                .unwrap_or(syntect::highlighting::Color::BLACK);
            Color::from_rgb(bg_color.r, bg_color.g, bg_color.b)
        };

        let mut rich_style = Style::new().color(fg).bgcolor(bg);

        // Apply font style modifiers
        if style
            .font_style
            .contains(syntect::highlighting::FontStyle::BOLD)
        {
            rich_style = rich_style.bold();
        }
        if style
            .font_style
            .contains(syntect::highlighting::FontStyle::ITALIC)
        {
            rich_style = rich_style.italic();
        }
        if style
            .font_style
            .contains(syntect::highlighting::FontStyle::UNDERLINE)
        {
            rich_style = rich_style.underline();
        }

        rich_style
    }

    /// Get the highlighted code as a concatenated string (for testing/preview).
    #[must_use]
    pub fn plain_text(&self) -> String {
        self.code.clone()
    }
}

fn pad_segments_to_width<'a>(segments: Vec<Segment<'a>>, width: usize) -> Vec<Segment<'a>> {
    let mut padded = Vec::new();
    let mut line_width = 0usize;

    for segment in segments {
        if segment.is_control() {
            padded.push(segment);
            continue;
        }

        let style = segment.style.clone();
        let text = segment.text;
        let text_ref = text.as_ref();
        let mut start = 0usize;

        for (idx, ch) in text_ref.char_indices() {
            if ch == '\n' {
                let part = &text_ref[start..idx];
                if !part.is_empty() {
                    padded.push(Segment::new(part.to_string(), style.clone()));
                    line_width += cells::cell_len(part);
                }
                if line_width < width {
                    padded.push(Segment::new(" ".repeat(width - line_width), None));
                }
                padded.push(Segment::line());
                line_width = 0;
                start = idx + 1;
            }
        }

        let tail = &text_ref[start..];
        if !tail.is_empty() {
            padded.push(Segment::new(tail.to_string(), style));
            line_width += cells::cell_len(tail);
        }
    }

    if line_width > 0 {
        if line_width < width {
            padded.push(Segment::new(" ".repeat(width - line_width), None));
        }
        padded.push(Segment::line());
    }

    padded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_new() {
        let syntax = Syntax::new("let x = 1;", "rust");
        assert_eq!(syntax.code, "let x = 1;");
        assert_eq!(syntax.language, "rust");
        assert!(!syntax.line_numbers);
    }

    #[test]
    fn test_syntax_builder() {
        let syntax = Syntax::new("print('hello')", "python")
            .line_numbers(true)
            .start_line(10)
            .theme("base16-ocean.dark")
            .indent_guides(true)
            .tab_size(2)
            .padding(1, 2);

        assert!(syntax.line_numbers);
        assert_eq!(syntax.start_line, 10);
        assert_eq!(syntax.theme_name, "base16-ocean.dark");
        assert!(syntax.indent_guides);
        assert_eq!(syntax.tab_size, 2);
        assert_eq!(syntax.padding, (1, 2));
    }

    #[test]
    fn test_extension_to_language() {
        assert_eq!(Syntax::extension_to_language("rs"), "rust");
        assert_eq!(Syntax::extension_to_language("py"), "python");
        assert_eq!(Syntax::extension_to_language("js"), "javascript");
        assert_eq!(Syntax::extension_to_language("ts"), "typescript");
        assert_eq!(Syntax::extension_to_language("go"), "go");
        assert_eq!(Syntax::extension_to_language("unknown"), "unknown");
    }

    #[test]
    fn test_available_themes() {
        let themes = Syntax::available_themes();
        assert!(!themes.is_empty());
        assert!(themes.iter().any(|t| t.contains("base16")));
    }

    #[test]
    fn test_available_languages() {
        let langs = Syntax::available_languages();
        assert!(!langs.is_empty());
    }

    #[test]
    fn test_render_simple() {
        let code = r#"fn main() {
    println!("Hello, world!");
}"#;
        let syntax = Syntax::new(code, "rust");
        let result = syntax.render(None);
        assert!(result.is_ok());
        let segments = result.unwrap();
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_render_with_line_numbers() {
        let code = "x = 1\ny = 2";
        let syntax = Syntax::new(code, "python").line_numbers(true);
        let result = syntax.render(None);
        assert!(result.is_ok());
        let segments = result.unwrap();
        // Should contain line number segments
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();
        assert!(text.contains('1'));
        assert!(text.contains('2'));
    }

    #[test]
    fn test_render_unknown_language() {
        let syntax = Syntax::new("code", "nonexistent_lang_xyz");
        let result = syntax.render(None);
        assert!(result.is_err());
        if let Err(SyntaxError::UnknownLanguage(lang)) = result {
            assert_eq!(lang, "nonexistent_lang_xyz");
        } else {
            panic!("Expected UnknownLanguage error");
        }
    }

    #[test]
    fn test_render_unknown_theme() {
        let syntax = Syntax::new("let x = 1", "rust").theme("nonexistent_theme_xyz");
        let result = syntax.render(None);
        assert!(result.is_err());
        if let Err(SyntaxError::UnknownTheme(theme)) = result {
            assert_eq!(theme, "nonexistent_theme_xyz");
        } else {
            panic!("Expected UnknownTheme error");
        }
    }

    #[test]
    fn test_plain_text() {
        let code = "fn main() {}";
        let syntax = Syntax::new(code, "rust");
        assert_eq!(syntax.plain_text(), code);
    }

    #[test]
    fn test_background_color_override() {
        let syntax = Syntax::new("code", "text").background_color(Color::parse("red").unwrap());
        assert!(syntax.background_color.is_some());
    }

    #[test]
    fn test_start_line_minimum() {
        let syntax = Syntax::new("code", "text").start_line(0);
        assert_eq!(syntax.start_line, 1); // Should be at minimum 1
    }

    #[test]
    fn test_tab_size_minimum() {
        let syntax = Syntax::new("code", "text").tab_size(0);
        assert_eq!(syntax.tab_size, 1); // Should be at minimum 1
    }

    #[test]
    fn test_padding_does_not_shift_lines() {
        // Use "rust" as a valid language (syntect doesn't have a "text" syntax)
        let syntax = Syntax::new("a\nb", "rust").padding(0, 2);
        let text = syntax
            .render(None)
            .expect("render should succeed")
            .iter()
            .map(|s| s.text.as_ref())
            .collect::<String>();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines, vec!["  a  ", "  b  "]);
    }

    #[test]
    fn test_render_crlf_strips_carriage_returns() {
        let code = "let x = 1;\r\nlet y = 2;\r\n";
        let syntax = Syntax::new(code, "rust");
        let text = syntax
            .render(None)
            .expect("render should succeed")
            .iter()
            .map(|s| s.text.as_ref())
            .collect::<String>();
        assert!(!text.contains('\r'));
        assert!(text.contains("let x = 1;"));
        assert!(text.contains("let y = 2;"));
    }
}
