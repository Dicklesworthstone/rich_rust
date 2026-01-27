//! Theme support for named styles (Python Rich parity).
//!
//! Python Rich has a global style registry (`Theme`) containing many named styles
//! (e.g. `"rule.line"`, `"table.header"`). `Console.get_style()` will first consult
//! the active theme, and fall back to parsing a style definition if no named style
//! exists.
//!
//! This module ports `rich.theme` + `rich.default_styles` for `rich_rust`.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use crate::style::{Style, StyleParseError};

static DEFAULT_STYLES: LazyLock<HashMap<String, Style>> = LazyLock::new(|| {
    let mut styles = HashMap::new();

    for (line_no, line) in include_str!("default_styles.tsv").lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (name, definition) = line.split_once('\t').unwrap_or_else(|| {
            panic!(
                "src/default_styles.tsv:{}: expected TAB-separated name + style",
                line_no + 1
            )
        });

        let style = Style::parse(definition).unwrap_or_else(|err| {
            panic!(
                "src/default_styles.tsv:{}: failed to parse style {definition:?} for {name:?}: {err}",
                line_no + 1
            )
        });

        let prior = styles.insert(name.to_string(), style);
        assert!(
            prior.is_none(),
            "src/default_styles.tsv:{}: duplicate style key {name:?}",
            line_no + 1
        );
    }

    styles
});

/// A container for style information used by [`crate::console::Console`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    styles: HashMap<String, Style>,
}

impl Theme {
    /// Create a theme from a map of named styles.
    ///
    /// If `inherit` is true, the theme starts with Python Rich's built-in
    /// `DEFAULT_STYLES` and the provided styles override / extend them.
    #[must_use]
    pub fn new(styles: Option<HashMap<String, Style>>, inherit: bool) -> Self {
        let mut merged = if inherit {
            DEFAULT_STYLES.clone()
        } else {
            HashMap::new()
        };

        if let Some(styles) = styles {
            merged.extend(styles);
        }

        Self { styles: merged }
    }

    /// Build a theme from string style definitions (`"bold red"`, `"rule.line"`, etc).
    pub fn from_style_definitions<I, K, V>(styles: I, inherit: bool) -> Result<Self, ThemeError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: AsRef<str>,
    {
        let mut parsed = HashMap::new();
        for (name, definition) in styles {
            let name = name.into();
            let style =
                Style::parse(definition.as_ref()).map_err(|err| ThemeError::InvalidStyle {
                    name: name.clone(),
                    err,
                })?;
            parsed.insert(name, style);
        }
        Ok(Self::new(Some(parsed), inherit))
    }

    /// Get a style by its theme name (exact match).
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Style> {
        self.styles.get(name)
    }

    /// Get all styles in this theme.
    #[must_use]
    pub fn styles(&self) -> &HashMap<String, Style> {
        &self.styles
    }

    /// Get the contents of a `.ini` theme file for this theme (Python Rich compatible).
    #[must_use]
    pub fn config(&self) -> String {
        let mut names: Vec<&str> = self.styles.keys().map(String::as_str).collect();
        names.sort_unstable();

        let mut out = String::from("[styles]\n");
        for name in names {
            let style = self.styles.get(name).expect("key exists");
            out.push_str(name);
            out.push_str(" = ");
            out.push_str(&style.to_string());
            out.push('\n');
        }
        out
    }

    /// Parse a `.ini` theme file string (supports a `[styles]` section).
    ///
    /// This is intentionally minimal but matches the common subset used by Rich.
    pub fn from_ini_str(contents: &str, inherit: bool) -> Result<Self, ThemeError> {
        let mut in_styles = false;
        let mut seen_styles_section = false;
        let mut styles: HashMap<String, Style> = HashMap::new();

        for (line_no, raw_line) in contents.lines().enumerate() {
            let line = raw_line.trim();

            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                let section_name = line[1..line.len() - 1].trim();
                in_styles = section_name.eq_ignore_ascii_case("styles");
                if in_styles {
                    seen_styles_section = true;
                }
                continue;
            }

            if !in_styles {
                continue;
            }

            let (name, definition) = line
                .split_once('=')
                .or_else(|| line.split_once(':'))
                .ok_or_else(|| ThemeError::InvalidIniLine {
                    line_no: line_no + 1,
                    line: raw_line.to_string(),
                })?;

            // Match Python's configparser default behavior: option keys are lowercased.
            let name = name.trim().to_lowercase();
            if name.is_empty() {
                return Err(ThemeError::InvalidIniLine {
                    line_no: line_no + 1,
                    line: raw_line.to_string(),
                });
            }

            let definition = definition.trim();
            let style = Style::parse(definition).map_err(|err| ThemeError::InvalidStyle {
                name: name.clone(),
                err,
            })?;

            if styles.insert(name.clone(), style).is_some() {
                return Err(ThemeError::DuplicateIniKey {
                    line_no: line_no + 1,
                    name,
                });
            }
        }

        if !seen_styles_section {
            return Err(ThemeError::MissingStylesSection);
        }

        Ok(Self::new(Some(styles), inherit))
    }

    /// Read a `.ini` theme file from disk.
    pub fn read(path: impl AsRef<Path>, inherit: bool) -> Result<Self, ThemeError> {
        let contents = fs::read_to_string(&path).map_err(|err| ThemeError::Io {
            path: path.as_ref().to_path_buf(),
            err,
        })?;
        Self::from_ini_str(&contents, inherit)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(None, true)
    }
}

/// Errors returned by Theme / `ThemeStack` operations.
#[derive(Debug)]
pub enum ThemeError {
    Io {
        path: std::path::PathBuf,
        err: std::io::Error,
    },
    MissingStylesSection,
    InvalidIniLine {
        line_no: usize,
        line: String,
    },
    DuplicateIniKey {
        line_no: usize,
        name: String,
    },
    InvalidStyle {
        name: String,
        err: StyleParseError,
    },
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, err } => {
                write!(f, "failed to read theme file {}: {err}", path.display())
            }
            Self::MissingStylesSection => write!(f, "theme ini is missing a [styles] section"),
            Self::InvalidIniLine { line_no, line } => {
                write!(f, "invalid theme ini line {line_no}: {line:?}")
            }
            Self::DuplicateIniKey { line_no, name } => {
                write!(f, "duplicate theme key {name:?} at line {line_no}")
            }
            Self::InvalidStyle { name, err } => {
                write!(f, "invalid style definition for theme key {name:?}: {err}")
            }
        }
    }
}

impl std::error::Error for ThemeError {}

/// Base exception for theme stack errors (Python Rich parity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeStackError;

impl fmt::Display for ThemeStackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to pop base theme")
    }
}

impl std::error::Error for ThemeStackError {}

/// A stack of themes (Python Rich parity).
#[derive(Debug, Clone)]
pub struct ThemeStack {
    entries: Vec<HashMap<String, Style>>,
}

impl ThemeStack {
    /// Create a theme stack with a base theme.
    #[must_use]
    pub fn new(theme: Theme) -> Self {
        Self {
            entries: vec![theme.styles],
        }
    }

    /// Get a style by name from the top-most theme.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Style> {
        self.entries.last().and_then(|styles| styles.get(name))
    }

    /// Push a theme on top of the stack.
    pub fn push_theme(&mut self, theme: Theme, inherit: bool) {
        let styles = if inherit {
            let mut merged = self.entries.last().cloned().unwrap_or_else(HashMap::new);
            merged.extend(theme.styles);
            merged
        } else {
            theme.styles
        };
        self.entries.push(styles);
    }

    /// Pop (and discard) the top-most theme.
    pub fn pop_theme(&mut self) -> Result<(), ThemeStackError> {
        if self.entries.len() == 1 {
            return Err(ThemeStackError);
        }
        self.entries.pop();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_contains_rule_line() {
        let theme = Theme::default();
        assert!(theme.get("rule.line").is_some());
        assert_eq!(theme.get("rule.line").unwrap().to_string(), "bright_green");
    }

    #[test]
    fn theme_from_style_definitions_overrides_defaults() {
        let theme =
            Theme::from_style_definitions([("rule.line", "bold red")], true).expect("theme");
        assert_eq!(theme.get("rule.line").unwrap().to_string(), "bold red");
    }

    #[test]
    fn theme_config_roundtrip_has_styles_section() {
        let theme = Theme::from_style_definitions([("warning", "bold red")], false).expect("theme");
        let config = theme.config();
        assert!(config.starts_with("[styles]\n"));
        assert!(config.contains("warning = bold red\n"));
    }

    #[test]
    fn theme_from_ini_str_parses_styles_section() {
        let ini = "[styles]\nwarning = bold red\n";
        let theme = Theme::from_ini_str(ini, false).expect("theme");
        assert_eq!(theme.get("warning").unwrap().to_string(), "bold red");
    }

    #[test]
    fn theme_stack_pop_base_errors() {
        let mut stack = ThemeStack::new(Theme::default());
        let err = stack.pop_theme().expect_err("expected error");
        assert_eq!(err.to_string(), "Unable to pop base theme");
    }

    #[test]
    fn theme_stack_push_and_pop() {
        let mut stack = ThemeStack::new(Theme::default());
        let theme = Theme::from_style_definitions([("warning", "bold red")], false).expect("theme");
        stack.push_theme(theme, true);
        assert_eq!(stack.get("warning").unwrap().to_string(), "bold red");
        stack.pop_theme().expect("pop theme");
    }
}
