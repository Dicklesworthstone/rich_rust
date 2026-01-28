//! Group renderable for combining multiple renderables.
//!
//! A `Group` combines multiple renderables into a single unit that can be
//! passed to containers like Panel or Layout. This is useful when you want
//! to pass multiple renderables as panel content.
//!
//! # Examples
//!
//! ```rust,ignore
//! use rich_rust::renderables::{Group, Panel, Rule};
//! use rich_rust::text::Text;
//!
//! // Combine multiple renderables into a group
//! let group = Group::new()
//!     .push("First paragraph of text")
//!     .push(Rule::new())
//!     .push("Second paragraph");
//!
//! // Use the group as panel content
//! let panel = Panel::from_renderable(&group, 80)
//!     .title("Grouped Content");
//!
//! // Or render directly
//! console.print_renderable(&group);
//! ```
//!
//! # Fit Option
//!
//! By default, each renderable is rendered on its own lines. Use `fit(true)`
//! to attempt to render items inline when they fit.

use crate::console::{Console, ConsoleOptions};
use crate::segment::Segment;

use super::Renderable;

/// A container that groups multiple renderables together.
///
/// Group implements the Renderable trait, allowing you to combine
/// multiple renderables into a single unit. Each child is rendered
/// in sequence with optional separators.
#[derive(Default)]
pub struct Group<'a> {
    /// The renderables in this group.
    children: Vec<Box<dyn Renderable + 'a>>,
    /// Whether to fit items inline when possible.
    fit: bool,
}

impl<'a> Group<'a> {
    /// Create a new empty group.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a renderable to the group.
    ///
    /// The renderable is boxed and stored. You can add any type that
    /// implements the Renderable trait.
    #[must_use]
    pub fn push<R: Renderable + 'a>(mut self, renderable: R) -> Self {
        self.children.push(Box::new(renderable));
        self
    }

    /// Add a boxed renderable to the group.
    #[must_use]
    pub fn push_boxed(mut self, renderable: Box<dyn Renderable + 'a>) -> Self {
        self.children.push(renderable);
        self
    }

    /// Set whether to fit items inline when possible.
    ///
    /// When `fit` is true, items that would fit on the same line are
    /// rendered together. When false (default), each item gets its own line.
    #[must_use]
    pub fn fit(mut self, fit: bool) -> Self {
        self.fit = fit;
        self
    }

    /// Check if the group is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Return the number of items in the group.
    #[must_use]
    pub fn len(&self) -> usize {
        self.children.len()
    }
}

impl Renderable for Group<'_> {
    fn render(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment<'_>> {
        let mut segments = Vec::new();

        for (i, child) in self.children.iter().enumerate() {
            // Add newline between items unless fit mode is on
            if i > 0 && !self.fit {
                segments.push(Segment::new("\n".to_string(), None));
            }

            // Render the child
            let child_segments = child.render(console, options);
            segments.extend(child_segments.into_iter().map(Segment::into_owned));
        }

        segments
    }
}

/// Create a group from an iterator of renderables.
///
/// This is a convenience function for creating groups from iterators.
///
/// # Examples
///
/// ```rust,ignore
/// use rich_rust::renderables::group::group;
///
/// let items = vec!["Item 1", "Item 2", "Item 3"];
/// let g = group(items.into_iter());
/// ```
pub fn group<'a, I, R>(iter: I) -> Group<'a>
where
    I: IntoIterator<Item = R>,
    R: Renderable + 'a,
{
    let mut g = Group::new();
    for item in iter {
        g = g.push(item);
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;

    #[test]
    fn test_group_new() {
        let g: Group = Group::new();
        assert!(g.is_empty());
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn test_group_add_strings() {
        let g = Group::new()
            .push("First")
            .push("Second")
            .push("Third");
        assert_eq!(g.len(), 3);
    }

    #[test]
    fn test_group_render() {
        let g = Group::new()
            .push("Line 1")
            .push("Line 2");

        let console = Console::builder()
            .force_terminal(false)
            .markup(false)
            .build();
        let options = console.options();

        let segments = g.render(&console, &options);
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();

        assert!(text.contains("Line 1"));
        assert!(text.contains("Line 2"));
        assert!(text.contains('\n'), "should have newline between items");
    }

    #[test]
    fn test_group_render_fit_mode() {
        let g = Group::new()
            .push("Part1")
            .push("Part2")
            .fit(true);

        let console = Console::builder()
            .force_terminal(false)
            .markup(false)
            .build();
        let options = console.options();

        let segments = g.render(&console, &options);
        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();

        assert!(text.contains("Part1"));
        assert!(text.contains("Part2"));
        // In fit mode, no newlines between items
        assert!(!text.contains('\n'));
    }

    #[test]
    fn test_group_function() {
        let items = vec!["A", "B", "C"];
        let g = group(items);
        assert_eq!(g.len(), 3);
    }
}
