//! Renderable components for rich terminal output.
//!
//! This module provides high-level components for structured terminal output:
//!
//! - [`Table`]: Display data in rows and columns with borders
//! - [`Panel`]: Frame content with a title and border
//! - [`Tree`]: Hierarchical data with guide lines
//! - [`ProgressBar`] / [`Spinner`]: Visual progress indicators
//! - [`Rule`]: Horizontal divider lines
//! - [`Columns`]: Multi-column text layout
//! - [`Align`]: Text alignment utilities
//!
//! # Examples
//!
//! ## Tables
//!
//! ```rust,ignore
//! use rich_rust::prelude::*;
//!
//! let table = Table::new()
//!     .title("Users")
//!     .add_column(Column::new("Name").style(Style::new().bold()))
//!     .add_column(Column::new("Email"))
//!     .add_row(Row::new().cell("Alice").cell("alice@example.com"))
//!     .add_row(Row::new().cell("Bob").cell("bob@example.com"));
//!
//! // Render to segments
//! for segment in table.render(80) {
//!     print!("{}", segment.text);
//! }
//! ```
//!
//! ## Panels
//!
//! ```rust,ignore
//! use rich_rust::prelude::*;
//!
//! let panel = Panel::new("Important message!")
//!     .title("Notice")
//!     .border_style(Style::new().color(Color::parse("yellow").unwrap()));
//!
//! for segment in panel.render(60) {
//!     print!("{}", segment.text);
//! }
//! ```
//!
//! ## Trees
//!
//! ```rust,ignore
//! use rich_rust::prelude::*;
//!
//! let tree = Tree::new(
//!     TreeNode::new("Root")
//!         .child(TreeNode::new("Branch A")
//!             .child(TreeNode::new("Leaf 1")))
//!         .child(TreeNode::new("Branch B")),
//! )
//! .guides(TreeGuides::Unicode);
//!
//! for segment in tree.render() {
//!     print!("{}", segment.text);
//! }
//! ```
//!
//! ## Rules (Dividers)
//!
//! ```rust,ignore
//! use rich_rust::prelude::*;
//!
//! let rule = Rule::new()
//!     .style(Style::new().color(Color::parse("blue").unwrap()));
//!
//! let titled_rule = Rule::with_title("Section Title");
//!
//! for segment in rule.render(80) {
//!     print!("{}", segment.text);
//! }
//! ```
//!
//! # Optional Features
//!
//! Additional renderables are available with feature flags:
//!
//! - **`syntax`**: [`Syntax`] - Syntax-highlighted source code
//! - **`markdown`**: [`Markdown`] - Markdown document rendering
//! - **`json`**: [`Json`] - JSON formatting with syntax highlighting

use crate::console::{Console, ConsoleOptions};
use crate::markup;
use crate::segment::Segment;
use crate::text::Text;

/// Trait for objects that can be rendered to the console.
pub trait Renderable {
    /// Render the object to a list of segments.
    fn render<'a>(&'a self, console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>>;
}

pub mod align;
pub mod columns;
pub mod layout;
pub mod padding;
pub mod panel;
pub mod progress;
pub mod rule;
pub mod table;
pub mod tree;

// Re-export commonly used types
pub use align::{Align, AlignLines, AlignMethod, VerticalAlignMethod, align_text};
pub use columns::Columns;
pub use layout::{Layout, LayoutSplitter, Region};
pub use padding::{Padding, PaddingDimensions};
pub use panel::Panel;
pub use progress::{BarStyle, ProgressBar, Spinner};
pub use rule::Rule;
pub use table::{Cell, Column, Row, Table, VerticalAlign};
pub use tree::{Tree, TreeGuides, TreeNode};

impl Renderable for Table {
    fn render<'a>(&'a self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        // Table::render currently returns Vec<Segment<'static>>
        // We can cast to Vec<Segment<'a>> via coercion?
        // No, Vec is invariant.
        // We need to convert.
        // Or change Table::render to return Vec<Segment<'a>>?
        // Table::render implementation creates owned segments.
        // So it returns Vec<Segment<'static>>.
        // This is a subtype of Vec<Segment<'a>>?
        // No, Vec<T> is invariant in T.
        // So Vec<Segment<'static>> is NOT Vec<Segment<'a>>.
        // We must return Vec<Segment<'a>> which can hold static segments.
        // But we cannot simply cast the Vec.
        // We have to map? expensive.
        // Or change Table::render signature.
        self.render(options.max_width).into_iter().collect()
    }
}

impl Renderable for str {
    fn render<'a>(&'a self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        // Honor the markup setting from ConsoleOptions
        let text = if options.markup.unwrap_or(true) {
            markup::render_or_plain(self)
        } else {
            Text::new(self)
        };
        text.render("")
            .into_iter()
            .map(Segment::into_owned)
            .collect()
    }
}

impl Renderable for String {
    fn render<'a>(&'a self, console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        self.as_str().render(console, options)
    }
}

// Phase 3+: Syntax highlighting (requires "syntax" feature)
#[cfg(feature = "syntax")]
pub mod syntax;

#[cfg(feature = "syntax")]
pub use syntax::{Syntax, SyntaxError};

#[cfg(feature = "syntax")]
impl Renderable for Syntax {
    fn render<'a>(&'a self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        self.render(Some(options.max_width))
            .unwrap_or_default()
            .into_iter()
            .map(Segment::into_owned) // Ensure static/owned segments
            .collect()
    }
}

// Phase 3+: Markdown rendering (requires "markdown" feature)
#[cfg(feature = "markdown")]
pub mod markdown;

#[cfg(feature = "markdown")]
pub use markdown::Markdown;

#[cfg(feature = "markdown")]
impl Renderable for Markdown {
    fn render<'a>(&'a self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        self.render(options.max_width).into_iter().collect()
    }
}

// Phase 4: JSON rendering (requires "json" feature)
#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "json")]
pub use json::{Json, JsonError, JsonTheme};

#[cfg(feature = "json")]
impl Renderable for Json {
    fn render<'a>(&'a self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment<'a>> {
        self.render().into_iter().map(Segment::into_owned).collect()
    }
}
