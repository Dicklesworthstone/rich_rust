//! Typography and spacing patterns for demo_showcase.
//!
//! This module provides consistent visual patterns across all scenes:
//! - Section headers (Rule + title combinations)
//! - Standard padding and spacing
//! - Hero/title alignment utilities
//!
//! These helpers ensure the demo output feels designed and cohesive.

// Many helpers are provided for future scene implementations
#![allow(dead_code)]

use rich_rust::console::Console;
use rich_rust::console::PrintOptions;
use rich_rust::renderables::rule::Rule;
use rich_rust::style::Style;

/// Standard vertical spacing between major sections.
pub const SECTION_SPACING: usize = 1;

/// Standard padding inside panels (top, right, bottom, left).
pub const PANEL_PADDING: (usize, usize, usize, usize) = (1, 2, 1, 2);

/// Standard margin around major blocks (top, right, bottom, left).
pub const BLOCK_MARGIN: (usize, usize, usize, usize) = (0, 1, 0, 1);

/// Print a styled section header with a rule and title.
///
/// Creates a visually distinct section break with:
/// - A horizontal rule styled with section.rule
/// - A styled title
/// - A blank line after for spacing
///
/// # Example
/// ```ignore
/// section_header(&console, "Table Showcase", false);
/// ```
pub fn section_header(console: &Console, title: &str, centered: bool) {
    let rule_style = console.get_style("section.rule");
    console.print_renderable(&Rule::new().style(rule_style));

    let styled_title = format!("[section.title]{}[/]", title);
    if centered {
        // Use justify for centering
        console.print_with_options(
            &styled_title,
            &PrintOptions::new()
                .with_markup(true)
                .with_justify(rich_rust::text::JustifyMethod::Center),
        );
    } else {
        console.print_with_options(&styled_title, &PrintOptions::new().with_markup(true));
    }

    console.print_plain(""); // Blank line after title
}

/// Print a styled scene header with prominent title and subtitle.
///
/// Used at the start of major scenes for hero-style presentation.
pub fn scene_header(console: &Console, title: &str, subtitle: Option<&str>) {
    let title_markup = format!("[brand.title]{}[/]", title);
    console.print_with_options(
        &title_markup,
        &PrintOptions::new()
            .with_markup(true)
            .with_justify(rich_rust::text::JustifyMethod::Center),
    );

    if let Some(sub) = subtitle {
        let sub_markup = format!("[brand.subtitle]{}[/]", sub);
        console.print_with_options(
            &sub_markup,
            &PrintOptions::new()
                .with_markup(true)
                .with_justify(rich_rust::text::JustifyMethod::Center),
        );
    }

    console.print_plain("");
}

/// Print a muted hint/instruction line.
pub fn hint(console: &Console, text: &str) {
    console.print_with_options(
        &format!("[hint]{}[/]", text),
        &PrintOptions::new().with_markup(true),
    );
}

/// Print a blank line for vertical spacing.
pub fn spacer(console: &Console) {
    console.print_plain("");
}

/// Print multiple blank lines for larger vertical gaps.
pub fn spacer_n(console: &Console, n: usize) {
    for _ in 0..n {
        console.print_plain("");
    }
}

/// Create a thin divider rule (less prominent than section header).
pub fn divider() -> Rule {
    let dim_style = Style::parse("dim").unwrap_or_default();
    Rule::new().style(dim_style)
}

/// Print a divider directly to the console.
pub fn print_divider(console: &Console) {
    console.print_renderable(&divider());
}

/// Create a styled status badge text.
///
/// Returns markup that can be used inline:
/// - `status_badge("OK", "ok")` → `"[status.ok.badge] OK [/]"`
/// - `status_badge("FAIL", "err")` → `"[status.err.badge] FAIL [/]"`
#[must_use]
pub fn status_badge(text: &str, status: &str) -> String {
    format!("[status.{}.badge] {} [/]", status, text)
}

/// Create styled status text (without badge background).
#[must_use]
pub fn status_text(text: &str, status: &str) -> String {
    format!("[status.{}]{}[/]", status, text)
}

/// Create a brand accent markup string.
#[must_use]
pub fn brand_accent(text: &str) -> String {
    format!("[brand.accent]{}[/]", text)
}

/// Create a muted text markup string.
#[must_use]
pub fn muted(text: &str) -> String {
    format!("[brand.muted]{}[/]", text)
}

/// Create a key-value row suitable for panels or lists.
///
/// Returns a formatted string with the key styled as a label and value as-is:
/// `"[dim]key:[/] value"`
///
/// # Example
/// ```ignore
/// let row = kv_row("Version", "1.2.3");
/// // Returns: "[dim]Version:[/] 1.2.3"
/// ```
#[must_use]
pub fn kv_row(key: &str, value: &str) -> String {
    format!("[dim]{}:[/] {}", key, value)
}

/// Create a key-value row with custom key and value styles.
///
/// # Example
/// ```ignore
/// let row = kv_row_styled("Status", "status.ok", "Running", "status.ok");
/// ```
#[must_use]
pub fn kv_row_styled(key: &str, key_style: &str, value: &str, value_style: &str) -> String {
    format!("[{}]{}:[/] [{}]{}[/]", key_style, key, value_style, value)
}

/// Generic badge helper that wraps text with a style.
///
/// # Example
/// ```ignore
/// let badge = badge("NEW", "brand.accent");
/// // Returns: "[brand.accent] NEW [/]"
/// ```
#[must_use]
pub fn badge(text: &str, style: &str) -> String {
    format!("[{}] {} [/]", style, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_badge_formatting() {
        assert_eq!(status_badge("OK", "ok"), "[status.ok.badge] OK [/]");
        assert_eq!(status_badge("WARN", "warn"), "[status.warn.badge] WARN [/]");
        assert_eq!(status_badge("FAIL", "err"), "[status.err.badge] FAIL [/]");
    }

    #[test]
    fn test_status_text_formatting() {
        assert_eq!(status_text("Passed", "ok"), "[status.ok]Passed[/]");
        assert_eq!(status_text("Warning", "warn"), "[status.warn]Warning[/]");
    }

    #[test]
    fn test_brand_accent_formatting() {
        assert_eq!(brand_accent("highlight"), "[brand.accent]highlight[/]");
    }

    #[test]
    fn test_muted_formatting() {
        assert_eq!(muted("subtle"), "[brand.muted]subtle[/]");
    }

    #[test]
    fn test_padding_constants_are_reasonable() {
        let (top, right, bottom, left) = PANEL_PADDING;
        assert!(top <= 3, "panel padding top should be modest");
        assert!(right <= 4, "panel padding right should be modest");
        assert!(bottom <= 3, "panel padding bottom should be modest");
        assert!(left <= 4, "panel padding left should be modest");
    }

    #[test]
    fn test_section_spacing_is_reasonable() {
        assert!(SECTION_SPACING >= 1);
        assert!(SECTION_SPACING <= 3);
    }

    #[test]
    fn test_divider_creates_rule() {
        let rule = divider();
        // Just verify it compiles and creates a Rule
        let _ = rule;
    }

    #[test]
    fn test_kv_row_formatting() {
        assert_eq!(kv_row("Name", "Alice"), "[dim]Name:[/] Alice");
        assert_eq!(kv_row("Version", "1.0.0"), "[dim]Version:[/] 1.0.0");
    }

    #[test]
    fn test_kv_row_styled_formatting() {
        assert_eq!(
            kv_row_styled("Status", "bold", "Running", "status.ok"),
            "[bold]Status:[/] [status.ok]Running[/]"
        );
    }

    #[test]
    fn test_badge_formatting() {
        assert_eq!(badge("NEW", "brand.accent"), "[brand.accent] NEW [/]");
        assert_eq!(badge("INFO", "status.info"), "[status.info] INFO [/]");
    }
}
