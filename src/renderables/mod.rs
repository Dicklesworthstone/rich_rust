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

pub mod align;
pub mod columns;
pub mod padding;
pub mod panel;
pub mod progress;
pub mod rule;
pub mod table;
pub mod tree;

// Re-export commonly used types
pub use align::{Align, AlignLines, AlignMethod, VerticalAlignMethod, align_text};
pub use columns::Columns;
pub use padding::{Padding, PaddingDimensions};
pub use panel::Panel;
pub use progress::{BarStyle, ProgressBar, Spinner};
pub use rule::Rule;
pub use table::{Cell, Column, Row, Table, VerticalAlign};
pub use tree::{Tree, TreeGuides, TreeNode};

// Phase 3+: Syntax highlighting (requires "syntax" feature)
#[cfg(feature = "syntax")]
pub mod syntax;

#[cfg(feature = "syntax")]
pub use syntax::{Syntax, SyntaxError};

// Phase 3+: Markdown rendering (requires "markdown" feature)
#[cfg(feature = "markdown")]
pub mod markdown;

#[cfg(feature = "markdown")]
pub use markdown::Markdown;

// Phase 4: JSON rendering (requires "json" feature)
#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "json")]
pub use json::{Json, JsonError, JsonTheme};
