//! # rich_rust
//!
//! A Rust port of Python's Rich library for beautiful terminal output.
//!
//! This library provides an abstraction over ANSI escape codes to render styled text,
//! tables, panels, progress bars, trees, and more in the terminal.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rich_rust::prelude::*;
//!
//! let console = Console::new();
//! console.print("[bold red]Hello[/] [green]World[/]!");
//! ```
//!
//! ## Core Concepts
//!
//! - **[`Console`]**: The central entry point for printing styled output. Handles color
//!   detection, terminal dimensions, and ANSI code generation.
//! - **[`Style`]**: Visual attributes including foreground/background colors, text
//!   decorations (bold, italic, underline, etc.), and hyperlinks.
//! - **[`Text`]**: Rich text with overlapping style spans that can be justified, wrapped,
//!   and rendered to the terminal.
//! - **[`Segment`]**: The atomic rendering unit combining text content with a style.
//! - **[`Color`]**: Terminal colors supporting 4-bit ANSI, 8-bit (256), and 24-bit
//!   true color with automatic downgrading.
//!
//! ## Renderables
//!
//! The library provides several high-level renderables for structured output:
//!
//! - **[`renderables::Table`]**: Display data in rows and columns with borders
//! - **[`renderables::Panel`]**: Frame content with a title and border
//! - **[`renderables::Tree`]**: Hierarchical data with guide lines
//! - **[`renderables::ProgressBar`]**: Visual progress indicators
//! - **[`renderables::Rule`]**: Horizontal divider lines
//! - **[`renderables::Columns`]**: Multi-column text layout
//!
//! ## Markup Syntax
//!
//! The Console supports a simple markup syntax for inline styling:
//!
//! ```rust,ignore
//! // Basic styling
//! console.print("[bold]Bold text[/bold]");
//! console.print("[italic red]Red italic[/]");  // [/] closes any open tag
//!
//! // Colors
//! console.print("[green]Green[/] [#ff8800]Orange[/] [rgb(100,150,200)]Blue[/]");
//!
//! // Combinations
//! console.print("[bold underline magenta on white]Styled text[/]");
//!
//! // Hyperlinks
//! console.print("[link=https://example.com]Click here[/link]");
//! ```
//!
//! ## Features
//!
//! Optional features can be enabled in `Cargo.toml`:
//!
//! - **`syntax`**: Syntax highlighting for source code via syntect
//! - **`markdown`**: Markdown rendering via pulldown-cmark
//! - **`json`**: JSON formatting with syntax highlighting
//!
//! ```toml
//! [dependencies]
//! rich_rust = { version = "0.1", features = ["syntax", "markdown", "json"] }
//! ```
//!
//! ## Examples
//!
//! ### Styled Text
//!
//! ```rust,ignore
//! use rich_rust::prelude::*;
//!
//! // Build text programmatically
//! let mut text = Text::new("Hello, ");
//! text.append("World", Style::new().bold().color(Color::parse("red").unwrap()));
//! text.append("!", Style::new().italic());
//!
//! let console = Console::new();
//! console.print_text(&text);
//! ```
//!
//! ### Tables
//!
//! ```rust,ignore
//! use rich_rust::prelude::*;
//!
//! let table = Table::new()
//!     .add_column(Column::new("Name").style(Style::new().bold()))
//!     .add_column(Column::new("Age").justify(JustifyMethod::Right))
//!     .add_row(Row::new().cell("Alice").cell("30"))
//!     .add_row(Row::new().cell("Bob").cell("25"));
//!
//! for segment in table.render(80) {
//!     print!("{}", segment.text);
//! }
//! ```
//!
//! ### Panels
//!
//! ```rust,ignore
//! use rich_rust::prelude::*;
//!
//! let panel = Panel::new("Content inside a box")
//!     .title("My Panel")
//!     .border_style(Style::new().color(Color::parse("blue").unwrap()));
//!
//! for segment in panel.render(60) {
//!     print!("{}", segment.text);
//! }
//! ```
//!
//! ### Trees
//!
//! ```rust,ignore
//! use rich_rust::prelude::*;
//!
//! let tree = Tree::new(
//!     TreeNode::new("Project")
//!         .child(TreeNode::new("src")
//!             .child(TreeNode::new("main.rs"))
//!             .child(TreeNode::new("lib.rs")))
//!         .child(TreeNode::new("Cargo.toml")),
//! );
//!
//! for segment in tree.render() {
//!     print!("{}", segment.text);
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod r#box;
pub mod cells;
pub mod color;
pub mod console;
pub mod markup;
pub mod measure;
pub mod renderables;
pub mod segment;
pub mod style;
pub mod terminal;
pub mod text;

/// Re-exports for convenient usage
pub mod prelude {
    pub use crate::r#box::BoxChars;
    pub use crate::color::{Color, ColorSystem, ColorTriplet, ColorType};
    pub use crate::console::{Console, ConsoleOptions};
    pub use crate::measure::Measurement;
    pub use crate::renderables::{
        Align, AlignLines, AlignMethod, BarStyle, Cell, Column, Columns, PaddingDimensions, Panel,
        ProgressBar, Row, Rule, Spinner, Table, Tree, TreeGuides, TreeNode, VerticalAlign,
        VerticalAlignMethod, align_text,
    };
    pub use crate::segment::Segment;
    pub use crate::style::{Attributes, Style};
    pub use crate::text::{JustifyMethod, OverflowMethod, Span, Text};

    #[cfg(feature = "syntax")]
    pub use crate::renderables::{Syntax, SyntaxError};

    #[cfg(feature = "markdown")]
    pub use crate::renderables::Markdown;

    #[cfg(feature = "json")]
    pub use crate::renderables::{Json, JsonError, JsonTheme};
}

// Re-export key types at crate root
pub use color::{Color, ColorSystem, ColorTriplet, ColorType};
pub use console::Console;
pub use segment::Segment;
pub use style::{Attributes, Style};
pub use text::{Span, Text};
