//! Hero scene for demo_showcase.
//!
//! Introduces the Nebula Deploy brand and demonstrates rich_rust capabilities.
//! Content: branded title, capability detection panel, palette preview, hyperlink CTAs.

use std::sync::Arc;

use rich_rust::console::Console;
use rich_rust::renderables::panel::Panel;
use rich_rust::renderables::table::{Column, Table};
use rich_rust::style::Style;
use rich_rust::text::Text;

use crate::Config;
use crate::scenes::{Scene, SceneError};

/// Hero scene: branding, capabilities, palette preview.
pub struct HeroScene;

impl HeroScene {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Scene for HeroScene {
    fn name(&self) -> &'static str {
        "hero"
    }

    fn summary(&self) -> &'static str {
        "Introduce Nebula Deploy and the visual brand."
    }

    fn run(&self, console: &Arc<Console>, _cfg: &Config) -> Result<(), SceneError> {
        // Big branded title
        render_brand_title(console);

        console.print("");

        // Capability panel
        render_capabilities_panel(console);

        console.print("");

        // Palette preview
        render_palette_preview(console);

        console.print("");

        // Hyperlink CTAs
        render_ctas(console);

        Ok(())
    }
}

/// Render the big branded title with tagline.
fn render_brand_title(console: &Console) {
    let width = console.width();

    // Use compact layout for narrow terminals
    if width < 50 {
        // Narrow layout: simple centered text
        console.print("[brand.title]┌────────────────────────┐[/]");
        console.print("[brand.title]│[/] [bold #a78bfa]✦ NEBULA DEPLOY ✦[/] [brand.title]│[/]");
        console.print("[brand.title]└────────────────────────┘[/]");
        console.print("");
        console.print("[brand.subtitle]Beautiful terminal output[/]");
        console.print("[brand.muted]    powered by rich_rust[/]");
    } else {
        // Full-width layout with spaced letters
        console.print("[brand.title]╭───────────────────────────────────────────╮[/]");
        console
            .print("[brand.title]│[/]                                           [brand.title]│[/]");
        console.print(
            "[brand.title]│[/]     [bold #a78bfa]✦  N E B U L A   D E P L O Y  ✦[/]     [brand.title]│[/]",
        );
        console
            .print("[brand.title]│[/]                                           [brand.title]│[/]");
        console.print("[brand.title]╰───────────────────────────────────────────╯[/]");
        console.print("");
        console.print("[brand.subtitle]      Beautiful terminal output for Rust[/]");
        console.print("[brand.muted]             powered by rich_rust[/]");
    }
}

/// Render the capabilities detection panel.
fn render_capabilities_panel(console: &Console) {
    let width = console.width();
    let height = console.height();
    let is_terminal = console.is_terminal();
    let is_interactive = console.is_interactive();
    let color_system = console.color_system();
    let emoji_enabled = true; // Default for demo

    // Format color system name
    let color_name = match color_system {
        Some(cs) => format!("{cs:?}"),
        None => "None (no color)".to_string(),
    };

    // Build capability lines
    let lines = [
        format!(
            "[dim]Terminal size:[/] [brand.accent]{width}[/] × [brand.accent]{height}[/] cells"
        ),
        format!("[dim]Color system:[/]  [brand.accent]{color_name}[/]"),
        format!(
            "[dim]Is terminal:[/]   {}",
            if is_terminal {
                "[status.ok]yes[/]"
            } else {
                "[status.warn]no (piped)[/]"
            }
        ),
        format!(
            "[dim]Interactive:[/]   {}",
            if is_interactive {
                "[status.ok]yes[/]"
            } else {
                "[status.warn]no[/]"
            }
        ),
        format!(
            "[dim]Emoji:[/]         {}",
            if emoji_enabled {
                "[status.ok]enabled[/] ✨"
            } else {
                "[status.warn]disabled[/]"
            }
        ),
    ];

    // Create panel content
    let content: Vec<Vec<rich_rust::segment::Segment>> = lines
        .iter()
        .map(|line| {
            let text = rich_rust::markup::render_or_plain(line);
            text.render("")
                .into_iter()
                .map(rich_rust::segment::Segment::into_owned)
                .collect()
        })
        .collect();

    let panel = Panel::new(content)
        .title(Text::new("Environment Detection"))
        .border_style(Style::parse("dim #38bdf8").unwrap_or_default())
        .expand(false);

    console.print_renderable(&panel);
}

/// Render the color palette preview.
fn render_palette_preview(console: &Console) {
    let mut table = Table::new().title("Color Palette");
    table.add_column(Column::new("Category").style(Style::parse("dim").unwrap_or_default()));
    table.add_column(Column::new("Preview"));

    // Brand colors
    table.add_row_markup([
        "Brand",
        "[#a78bfa]████[/] [#c4b5fd]████[/] [#38bdf8]████[/]",
    ]);

    // Status colors
    table.add_row_markup([
        "Status",
        "[green]████[/] [yellow]████[/] [red]████[/] [cyan]████[/]",
    ]);

    // Badges
    table.add_row_markup([
        "Badges",
        "[bold white on green] OK [/] [bold black on yellow] WARN [/] [bold white on red] ERR [/]",
    ]);

    // Dim/muted
    table.add_row_markup(["Muted", "[dim #94a3b8]████[/] [dim #64748b]████[/]"]);

    console.print_renderable(&table);
}

/// Render call-to-action hyperlinks.
fn render_ctas(console: &Console) {
    console.print("[section.title]Get Started[/]");
    console.print("");

    // Documentation link
    console.print("  [dim]📖[/] Documentation: [link=https://docs.rs/rich_rust][brand.accent]docs.rs/rich_rust[/][/link]");

    // Repository link
    console.print("  [dim]📦[/] Repository:    [link=https://github.com/Dicklesworthstone/rich_rust][brand.accent]github.com/Dicklesworthstone/rich_rust[/][/link]");

    // Crates.io link
    console.print("  [dim]🦀[/] Crates.io:     [link=https://crates.io/crates/rich_rust][brand.accent]crates.io/crates/rich_rust[/][/link]");

    console.print("");
    console.print("[hint]Press any key to continue, or run with --scene <name> to jump to a specific demo.[/]");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hero_scene_has_correct_name() {
        let scene = HeroScene::new();
        assert_eq!(scene.name(), "hero");
    }

    #[test]
    fn hero_scene_has_summary() {
        let scene = HeroScene::new();
        assert!(!scene.summary().is_empty());
    }

    #[test]
    fn hero_scene_runs_without_error() {
        let scene = HeroScene::new();
        let console = Console::builder()
            .force_terminal(false)
            .markup(true)
            .build()
            .shared();
        let cfg = Config::with_defaults();

        let result = scene.run(&console, &cfg);
        assert!(result.is_ok());
    }

    #[test]
    fn hero_scene_produces_output() {
        let scene = HeroScene::new();
        let console = Console::builder()
            .force_terminal(false)
            .markup(true)
            .width(80)
            .build()
            .shared();
        let cfg = Config::with_defaults();

        console.begin_capture();
        let _ = scene.run(&console, &cfg);
        let segments = console.end_capture();

        // Collect all text into a string for easier assertion
        let output: String = segments.iter().map(|s| s.text.as_ref()).collect();

        // Should contain brand title (N E B U L A with spaces in the hero)
        assert!(
            output.contains("N E B U L A") || output.contains("D E P L O Y"),
            "output should contain brand title"
        );
        // Should contain capability info
        assert!(
            output.contains("Terminal size") || output.contains("Color system"),
            "output should contain capability info"
        );
    }
}
