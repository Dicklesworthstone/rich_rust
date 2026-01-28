//! Layout/composition scene for demo_showcase.
//!
//! Demonstrates rich_rust layout and composition capabilities:
//! - Columns for side-by-side content
//! - Align for horizontal positioning
//! - Padding for spacing and "card" feel

use std::sync::Arc;

use rich_rust::cells::cell_len;
use rich_rust::console::Console;
use rich_rust::renderables::align::{Align, AlignMethod};
use rich_rust::renderables::columns::Columns;
use rich_rust::renderables::padding::{Padding, PaddingDimensions};
use rich_rust::renderables::panel::Panel;
use rich_rust::segment::Segment;

use crate::Config;
use crate::scenes::{Scene, SceneError};

/// Layout/composition scene: demonstrates Columns, Align, and Padding.
pub struct LayoutScene;

impl LayoutScene {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Scene for LayoutScene {
    fn name(&self) -> &'static str {
        "layout"
    }

    fn summary(&self) -> &'static str {
        "Layout tools: Columns, Align, and Padding for polished UI composition."
    }

    fn run(&self, console: &Arc<Console>, cfg: &Config) -> Result<(), SceneError> {
        console.print("[section.title]Layout & Composition: Building Polished UIs[/]");
        console.print("");
        console.print("[dim]Combine Columns, Align, and Padding for professional layouts.[/]");
        console.print("");

        // Demo 1: Alignment showcase
        render_alignment_demo(console);

        console.print("");

        // Demo 2: Columns layout
        render_columns_demo(console);

        console.print("");

        // Demo 3: Padding for card-like containers
        render_padding_demo(console);

        console.print("");

        // Demo 4: Practical composition example
        render_composition_demo(console, cfg);

        Ok(())
    }
}

/// Render alignment demonstration.
fn render_alignment_demo(console: &Console) {
    console.print("[brand.accent]Horizontal Alignment[/]");
    console.print("");

    let width = 50;

    // Left aligned (default)
    let left = Align::from_str("Left-aligned text", width).left().render();
    let left_text: String = left.iter().map(|s| s.text.as_ref()).collect();
    console.print(&format!("|{}|", left_text));

    // Center aligned
    let center = Align::from_str("Centered text", width).center().render();
    let center_text: String = center.iter().map(|s| s.text.as_ref()).collect();
    console.print(&format!("|{}|", center_text));

    // Right aligned
    let right = Align::from_str("Right-aligned text", width)
        .right()
        .render();
    let right_text: String = right.iter().map(|s| s.text.as_ref()).collect();
    console.print(&format!("|{}|", right_text));

    console.print("");

    // Centered hero block
    let hero_lines = [
        "[bold cyan]Nebula Deploy[/]",
        "[dim]Production-ready in minutes[/]",
    ];

    for line in hero_lines {
        let aligned = Align::from_str(line, 60).center().render();
        let text: String = aligned.iter().map(|s| s.text.as_ref()).collect();
        console.print(&text);
    }

    console.print("");
    console.print(
        "[hint]Align wraps content to position it left, center, or right within a width.[/]",
    );
}

/// Render columns layout demonstration.
fn render_columns_demo(console: &Console) {
    console.print("[brand.accent]Multi-Column Layout[/]");
    console.print("");

    // Feature cards in columns
    let features = [
        "Tables", "Panels", "Trees", "Progress", "Syntax", "Markdown",
    ];

    let cols = Columns::from_strings(&features)
        .column_count(3)
        .gutter(4)
        .equal_width(true)
        .align(AlignMethod::Center);

    console.print_renderable(&cols);
    console.print("");

    // Descriptive cards (longer content)
    let cards = [
        "Tables: Structured data",
        "Panels: Bordered content",
        "Trees: Hierarchical views",
        "Progress: Live updates",
    ];

    let card_cols = Columns::from_strings(&cards)
        .column_count(2)
        .gutter(4)
        .equal_width(true);

    console.print_renderable(&card_cols);

    console.print("");
    console.print(
        "[hint]Columns arrange items in newspaper-style layout with configurable gutters.[/]",
    );
}

/// Render padding demonstration.
fn render_padding_demo(console: &Console) {
    console.print("[brand.accent]Padding for Visual Hierarchy[/]");
    console.print("");

    // Show different padding styles
    console
        .print("[dim]CSS-style padding: (vertical, horizontal) or (top, right, bottom, left)[/]");
    console.print("");

    // No padding
    let content_no_pad = vec![vec![Segment::new("No padding", None)]];
    let no_pad = Padding::new(content_no_pad, PaddingDimensions::zero(), 20);
    let no_pad_lines = no_pad.render();
    for line in no_pad_lines {
        let text: String = line.iter().map(|s| s.text.as_ref()).collect();
        console.print(&format!("[{}]", text));
    }

    // Symmetric padding
    let content_sym = vec![vec![Segment::new("Padding (1, 2)", None)]];
    let sym_pad = Padding::new(content_sym, (1, 2), 24);
    let sym_lines = sym_pad.render();
    for line in sym_lines {
        let text: String = line.iter().map(|s| s.text.as_ref()).collect();
        console.print(&format!("[{}]", text));
    }

    console.print("");

    // Card-like padding with background
    let content_card = vec![
        vec![Segment::new("[bold]Feature Card[/]", None)],
        vec![Segment::new("", None)],
        vec![Segment::new("Add spacing and structure", None)],
        vec![Segment::new("to make content stand out.", None)],
    ];
    let card_pad = Padding::new(content_card, (1, 3), 40);
    let card_lines = card_pad.render();
    for line in card_lines {
        let text: String = line.iter().map(|s| s.text.as_ref()).collect();
        console.print(&format!("|{}|", text));
    }

    console.print("");
    console.print("[hint]Padding creates breathing room around content for a polished look.[/]");
}

/// Render panels side by side by combining their rendered lines horizontally.
fn render_panels_side_by_side(
    console: &Console,
    panels: &[Panel<'_>],
    panel_width: usize,
    gutter: usize,
) {
    // Render each panel to plain text and split into lines
    let rendered: Vec<Vec<String>> = panels
        .iter()
        .map(|p| {
            let text = p.render_plain(panel_width);
            text.lines().map(String::from).collect()
        })
        .collect();

    // Find max height (number of lines)
    let max_lines = rendered.iter().map(|r| r.len()).max().unwrap_or(0);

    // Find width of each panel (by measuring actual rendered content)
    let widths: Vec<usize> = rendered
        .iter()
        .map(|lines| lines.iter().map(|l| cell_len(l)).max().unwrap_or(0))
        .collect();

    // Build combined lines
    for line_idx in 0..max_lines {
        let mut combined = String::new();
        for (panel_idx, lines) in rendered.iter().enumerate() {
            if panel_idx > 0 {
                combined.push_str(&" ".repeat(gutter));
            }
            let line = lines.get(line_idx).map(|s| s.as_str()).unwrap_or("");
            let line_width = cell_len(line);
            let this_panel_width = widths[panel_idx];
            combined.push_str(line);
            // Pad to panel width for alignment
            if line_width < this_panel_width {
                combined.push_str(&" ".repeat(this_panel_width - line_width));
            }
        }
        console.print(&combined);
    }
}

/// Render practical composition demonstration.
fn render_composition_demo(console: &Console, cfg: &Config) {
    console.print("[brand.accent]Composition: Putting It Together[/]");
    console.print("");

    // Create a multi-card layout using panels side by side
    let card1 = Panel::from_text(
        "[bold green]Production[/]\n\n\
         Status: Healthy\n\
         Uptime: 99.9%\n\
         Latency: 12ms",
    )
    .title("[green]us-west-2[/]")
    .width(28)
    .safe_box(cfg.is_safe_box());

    let card2 = Panel::from_text(
        "[bold green]Production[/]\n\n\
         Status: Healthy\n\
         Uptime: 99.8%\n\
         Latency: 45ms",
    )
    .title("[green]eu-west-1[/]")
    .width(28)
    .safe_box(cfg.is_safe_box());

    let card3 = Panel::from_text(
        "[bold yellow]Degraded[/]\n\n\
         Status: Elevated\n\
         Uptime: 98.5%\n\
         Latency: 120ms",
    )
    .title("[yellow]ap-south-1[/]")
    .width(28)
    .safe_box(cfg.is_safe_box());

    // Render cards side by side
    render_panels_side_by_side(console, &[card1, card2, card3], 28, 2);

    console.print("");

    // Centered summary
    let summary = Align::from_str(
        "[bold]3 regions | 99.4% avg uptime | 59ms avg latency[/]",
        80,
    )
    .center()
    .render();
    let summary_text: String = summary.iter().map(|s| s.text.as_ref()).collect();
    console.print(&summary_text);

    console.print("");
    console.print("[hint]Combine layout primitives to create dashboard-quality output.[/]");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_scene_has_correct_name() {
        let scene = LayoutScene::new();
        assert_eq!(scene.name(), "layout");
    }

    #[test]
    fn layout_scene_runs_without_error() {
        let scene = LayoutScene::new();
        let console = Console::builder()
            .force_terminal(false)
            .markup(true)
            .build()
            .shared();
        let cfg = Config::with_defaults();

        let result = scene.run(&console, &cfg);
        assert!(result.is_ok());
    }
}
