//! Tree renderable.
//!
//! This module provides tree components for displaying hierarchical data
//! in the terminal with configurable guide characters and styles.

use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

/// Guide character styles for tree rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreeGuides {
    /// ASCII guides using | ` - characters.
    Ascii,
    /// Unicode box-drawing characters (default).
    #[default]
    Unicode,
    /// Bold Unicode box-drawing characters.
    Bold,
    /// Double-line Unicode characters.
    Double,
    /// Rounded Unicode characters.
    Rounded,
}

impl TreeGuides {
    /// Vertical continuation guide (for items that have siblings below).
    #[must_use]
    pub const fn vertical(&self) -> &str {
        match self {
            Self::Ascii => "|   ",
            Self::Unicode | Self::Rounded => "\u{2502}   ", // │
            Self::Bold => "\u{2503}   ",                    // ┃
            Self::Double => "\u{2551}   ",                  // ║
        }
    }

    /// Branch guide (for items with siblings below).
    #[must_use]
    pub const fn branch(&self) -> &str {
        match self {
            Self::Ascii => "+-- ",
            Self::Unicode => "\u{251C}\u{2500}\u{2500} ", // ├──
            Self::Bold => "\u{2523}\u{2501}\u{2501} ",    // ┣━━
            Self::Double => "\u{2560}\u{2550}\u{2550} ",  // ╠══
            Self::Rounded => "\u{251C}\u{2500}\u{2500} ", // ├──
        }
    }

    /// Last item guide (for items without siblings below).
    #[must_use]
    pub const fn last(&self) -> &str {
        match self {
            Self::Ascii => "`-- ",
            Self::Unicode => "\u{2514}\u{2500}\u{2500} ", // └──
            Self::Bold => "\u{2517}\u{2501}\u{2501} ",    // ┗━━
            Self::Double => "\u{255A}\u{2550}\u{2550} ",  // ╚══
            Self::Rounded => "\u{2570}\u{2500}\u{2500} ", // ╰──
        }
    }

    /// Empty space (for indentation where no guide is needed).
    #[must_use]
    pub const fn space(&self) -> &str {
        "    "
    }
}

/// A node in the tree.
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// The label for this node.
    label: Text,
    /// Child nodes.
    children: Vec<TreeNode>,
    /// Whether this node is expanded (children visible).
    expanded: bool,
    /// Optional icon to display before the label.
    icon: Option<String>,
    /// Style for the icon.
    icon_style: Style,
}

impl TreeNode {
    /// Create a new tree node with a label.
    #[must_use]
    pub fn new(label: impl Into<Text>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            expanded: true,
            icon: None,
            icon_style: Style::new(),
        }
    }

    /// Create a new tree node with an icon and label.
    #[must_use]
    pub fn with_icon(icon: impl Into<String>, label: impl Into<Text>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            expanded: true,
            icon: Some(icon.into()),
            icon_style: Style::new(),
        }
    }

    /// Add a child node.
    #[must_use]
    pub fn child(mut self, node: TreeNode) -> Self {
        self.children.push(node);
        self
    }

    /// Add multiple child nodes.
    #[must_use]
    pub fn children(mut self, nodes: impl IntoIterator<Item = TreeNode>) -> Self {
        self.children.extend(nodes);
        self
    }

    /// Set the icon for this node.
    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the icon style.
    #[must_use]
    pub fn icon_style(mut self, style: Style) -> Self {
        self.icon_style = style;
        self
    }

    /// Set whether this node is expanded.
    #[must_use]
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Collapse this node (hide children).
    #[must_use]
    pub fn collapsed(self) -> Self {
        self.expanded(false)
    }

    /// Get the label text.
    #[must_use]
    pub fn label(&self) -> &Text {
        &self.label
    }

    /// Get the children.
    #[must_use]
    pub fn children_nodes(&self) -> &[TreeNode] {
        &self.children
    }

    /// Check if this node has children.
    #[must_use]
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Check if this node is expanded.
    #[must_use]
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Get the icon if set.
    #[must_use]
    pub fn get_icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }
}

/// A tree for displaying hierarchical data.
#[derive(Debug, Clone)]
pub struct Tree {
    /// The root node.
    root: TreeNode,
    /// Guide style.
    guides: TreeGuides,
    /// Style for the guide characters.
    guide_style: Style,
    /// Whether to show the root node.
    show_root: bool,
    /// Style for highlighted nodes.
    highlight_style: Option<Style>,
    /// Maximum depth to display (-1 for unlimited).
    max_depth: isize,
}

impl Default for Tree {
    fn default() -> Self {
        Self {
            root: TreeNode::new("root"),
            guides: TreeGuides::default(),
            guide_style: Style::new().color_str("bright_black").unwrap_or_default(),
            show_root: true,
            highlight_style: None,
            max_depth: -1,
        }
    }
}

impl Tree {
    /// Create a new tree with the given root node.
    #[must_use]
    pub fn new(root: TreeNode) -> Self {
        Self {
            root,
            ..Self::default()
        }
    }

    /// Create a new tree with just a label for the root.
    #[must_use]
    pub fn with_label(label: impl Into<Text>) -> Self {
        Self::new(TreeNode::new(label))
    }

    /// Set the guide style.
    #[must_use]
    pub fn guides(mut self, guides: TreeGuides) -> Self {
        self.guides = guides;
        self
    }

    /// Set the style for guide characters.
    #[must_use]
    pub fn guide_style(mut self, style: Style) -> Self {
        self.guide_style = style;
        self
    }

    /// Set whether to show the root node.
    #[must_use]
    pub fn show_root(mut self, show: bool) -> Self {
        self.show_root = show;
        self
    }

    /// Hide the root node (only show children).
    #[must_use]
    pub fn hide_root(self) -> Self {
        self.show_root(false)
    }

    /// Set the highlight style for nodes.
    #[must_use]
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = Some(style);
        self
    }

    /// Set the maximum depth to display.
    #[must_use]
    pub fn max_depth(mut self, depth: isize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Add a child node to the root.
    #[must_use]
    pub fn child(mut self, node: TreeNode) -> Self {
        self.root.children.push(node);
        self
    }

    /// Add multiple children to the root.
    #[must_use]
    pub fn children(mut self, nodes: impl IntoIterator<Item = TreeNode>) -> Self {
        self.root.children.extend(nodes);
        self
    }

    /// Render the tree to segments.
    #[must_use]
    pub fn render(&self) -> Vec<Segment> {
        let mut segments = Vec::new();
        let mut prefix_stack: Vec<bool> = Vec::new(); // true = has more siblings

        if self.show_root {
            self.render_node(&self.root, &mut segments, &prefix_stack, true, 0);
        } else {
            // Render children directly
            let children = &self.root.children;
            for (i, child) in children.iter().enumerate() {
                let is_last = i == children.len() - 1;
                self.render_node(child, &mut segments, &mut prefix_stack, is_last, 0);
            }
        }

        segments
    }

    /// Render a single node and its children recursively.
    fn render_node(
        &self,
        node: &TreeNode,
        segments: &mut Vec<Segment>,
        prefix_stack: &[bool],
        is_last: bool,
        depth: usize,
    ) {
        // Check depth limit
        if self.max_depth >= 0 && depth as isize > self.max_depth {
            return;
        }

        // Build the prefix (guides from ancestors)
        for &has_more_siblings in prefix_stack {
            let guide = if has_more_siblings {
                self.guides.vertical()
            } else {
                self.guides.space()
            };
            segments.push(Segment::new(guide, Some(self.guide_style.clone())));
        }

        // Add the branch guide for this node (if not root at depth 0)
        if !prefix_stack.is_empty() || !self.show_root {
            let guide = if is_last {
                self.guides.last()
            } else {
                self.guides.branch()
            };
            segments.push(Segment::new(guide, Some(self.guide_style.clone())));
        }

        // Add icon if present
        if let Some(icon) = node.get_icon() {
            segments.push(Segment::new(
                &format!("{icon} "),
                Some(node.icon_style.clone()),
            ));
        }

        // Add the label
        let label_style = self
            .highlight_style
            .clone()
            .unwrap_or_else(|| node.label.style().clone());
        segments.push(Segment::new(node.label.plain(), Some(label_style)));

        // Add collapse indicator if has children but collapsed
        if node.has_children() && !node.is_expanded() {
            segments.push(Segment::new(" [...]", Some(self.guide_style.clone())));
        }

        segments.push(Segment::line());

        // Render children if expanded
        if node.is_expanded() {
            let children = &node.children;
            let mut new_prefix_stack = prefix_stack.to_vec();
            new_prefix_stack.push(!is_last);

            for (i, child) in children.iter().enumerate() {
                let child_is_last = i == children.len() - 1;
                self.render_node(child, segments, &new_prefix_stack, child_is_last, depth + 1);
            }
        }
    }

    /// Render the tree as a plain string.
    #[must_use]
    pub fn render_plain(&self) -> String {
        self.render().into_iter().map(|seg| seg.text).collect()
    }
}

/// Create a tree from a file system-like structure.
///
/// Takes a root path and builds a tree showing the directory structure.
#[must_use]
pub fn file_tree(root: &str, entries: &[(&str, bool)]) -> Tree {
    let mut root_node = TreeNode::with_icon("📁", root);

    for (path, is_dir) in entries {
        let icon = if *is_dir { "📁" } else { "📄" };
        root_node = root_node.child(TreeNode::with_icon(icon, *path));
    }

    Tree::new(root_node)
}

/// Create an ASCII-style tree.
#[must_use]
pub fn ascii_tree(root: TreeNode) -> Tree {
    Tree::new(root).guides(TreeGuides::Ascii)
}

/// Create a rounded-style tree.
#[must_use]
pub fn rounded_tree(root: TreeNode) -> Tree {
    Tree::new(root).guides(TreeGuides::Rounded)
}

/// Create a bold-style tree.
#[must_use]
pub fn bold_tree(root: TreeNode) -> Tree {
    Tree::new(root).guides(TreeGuides::Bold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_new() {
        let node = TreeNode::new("test");
        assert_eq!(node.label().plain(), "test");
        assert!(node.children_nodes().is_empty());
        assert!(node.is_expanded());
    }

    #[test]
    fn test_tree_node_with_icon() {
        let node = TreeNode::with_icon("📁", "folder");
        assert_eq!(node.label().plain(), "folder");
        assert_eq!(node.get_icon(), Some("📁"));
    }

    #[test]
    fn test_tree_node_children() {
        let node = TreeNode::new("root")
            .child(TreeNode::new("child1"))
            .child(TreeNode::new("child2"));
        assert_eq!(node.children_nodes().len(), 2);
        assert!(node.has_children());
    }

    #[test]
    fn test_tree_node_collapsed() {
        let node = TreeNode::new("test").collapsed();
        assert!(!node.is_expanded());
    }

    #[test]
    fn test_tree_new() {
        let tree = Tree::with_label("root");
        assert!(tree.show_root);
        assert_eq!(tree.guides, TreeGuides::Unicode);
    }

    #[test]
    fn test_tree_guides_ascii() {
        let guides = TreeGuides::Ascii;
        assert_eq!(guides.vertical(), "|   ");
        assert_eq!(guides.branch(), "+-- ");
        assert_eq!(guides.last(), "`-- ");
        assert_eq!(guides.space(), "    ");
    }

    #[test]
    fn test_tree_guides_unicode() {
        let guides = TreeGuides::Unicode;
        assert!(guides.vertical().starts_with('\u{2502}')); // │
        assert!(guides.branch().starts_with('\u{251C}')); // ├
        assert!(guides.last().starts_with('\u{2514}')); // └
    }

    #[test]
    fn test_tree_render_simple() {
        let tree = Tree::with_label("root")
            .child(TreeNode::new("child1"))
            .child(TreeNode::new("child2"));

        let segments = tree.render();
        assert!(!segments.is_empty());

        let plain = tree.render_plain();
        assert!(plain.contains("root"));
        assert!(plain.contains("child1"));
        assert!(plain.contains("child2"));
    }

    #[test]
    fn test_tree_render_nested() {
        let tree =
            Tree::with_label("root").child(TreeNode::new("parent").child(TreeNode::new("child")));

        let plain = tree.render_plain();
        assert!(plain.contains("root"));
        assert!(plain.contains("parent"));
        assert!(plain.contains("child"));
    }

    #[test]
    fn test_tree_hide_root() {
        let tree = Tree::with_label("root")
            .hide_root()
            .child(TreeNode::new("visible"));

        let plain = tree.render_plain();
        assert!(!plain.contains("root"));
        assert!(plain.contains("visible"));
    }

    #[test]
    fn test_tree_collapsed_node() {
        let tree = Tree::with_label("root").child(
            TreeNode::new("collapsed")
                .collapsed()
                .child(TreeNode::new("hidden")),
        );

        let plain = tree.render_plain();
        assert!(plain.contains("collapsed"));
        assert!(plain.contains("[...]"));
        assert!(!plain.contains("hidden"));
    }

    #[test]
    fn test_tree_max_depth() {
        let tree = Tree::with_label("root")
            .max_depth(1)
            .child(TreeNode::new("level1").child(TreeNode::new("level2")));

        let plain = tree.render_plain();
        assert!(plain.contains("root"));
        assert!(plain.contains("level1"));
        assert!(!plain.contains("level2"));
    }

    #[test]
    fn test_tree_ascii_style() {
        let tree = ascii_tree(TreeNode::new("root").child(TreeNode::new("child")));

        let plain = tree.render_plain();
        assert!(plain.contains("+--") || plain.contains("`--"));
    }

    #[test]
    fn test_tree_with_icons() {
        let tree = Tree::with_label("project")
            .child(TreeNode::with_icon("📁", "src"))
            .child(TreeNode::with_icon("📄", "README.md"));

        let plain = tree.render_plain();
        assert!(plain.contains("📁"));
        assert!(plain.contains("📄"));
        assert!(plain.contains("src"));
        assert!(plain.contains("README.md"));
    }

    #[test]
    fn test_file_tree() {
        let tree = file_tree("project", &[("src", true), ("Cargo.toml", false)]);

        let plain = tree.render_plain();
        assert!(plain.contains("project"));
        assert!(plain.contains("src"));
        assert!(plain.contains("Cargo.toml"));
    }

    #[test]
    fn test_tree_complex_structure() {
        let tree = Tree::with_label("root")
            .child(
                TreeNode::new("branch1")
                    .child(TreeNode::new("leaf1"))
                    .child(TreeNode::new("leaf2")),
            )
            .child(
                TreeNode::new("branch2")
                    .child(TreeNode::new("sub-branch").child(TreeNode::new("deep-leaf"))),
            )
            .child(TreeNode::new("leaf3"));

        let plain = tree.render_plain();

        // Verify all nodes are present
        assert!(plain.contains("root"));
        assert!(plain.contains("branch1"));
        assert!(plain.contains("branch2"));
        assert!(plain.contains("leaf1"));
        assert!(plain.contains("leaf2"));
        assert!(plain.contains("leaf3"));
        assert!(plain.contains("sub-branch"));
        assert!(plain.contains("deep-leaf"));
    }

    #[test]
    fn test_tree_empty_root() {
        // Tree with just an empty root
        let tree = Tree::with_label("");
        let plain = tree.render_plain();
        // Should render without panic
        // Just verify it doesn't panic - the test passing is proof enough
        let _ = plain;
    }

    #[test]
    fn test_tree_single_node() {
        let tree = Tree::with_label("single");
        let plain = tree.render_plain();
        assert!(plain.contains("single"));
        // Should have no guide characters at root
        assert!(!plain.contains("├──"));
        assert!(!plain.contains("└──"));
    }

    #[test]
    fn test_tree_wide_unicode_labels() {
        // Test with CJK characters (each is 2 cells wide)
        let tree = Tree::with_label("项目") // "project" in Chinese
            .child(TreeNode::new("源代码")) // "source code"
            .child(TreeNode::new("文档")); // "documentation"

        let plain = tree.render_plain();
        assert!(plain.contains("项目"));
        assert!(plain.contains("源代码"));
        assert!(plain.contains("文档"));
    }

    #[test]
    fn test_tree_emoji_labels() {
        let tree = Tree::with_label("📁 Root")
            .child(TreeNode::new("📄 File"))
            .child(TreeNode::new("🔧 Config"));

        let plain = tree.render_plain();
        assert!(plain.contains("📁"));
        assert!(plain.contains("📄"));
        assert!(plain.contains("🔧"));
    }

    #[test]
    fn test_tree_guides_bold() {
        let guides = TreeGuides::Bold;
        assert_eq!(guides.vertical(), "┃   ");
        assert_eq!(guides.branch(), "┣━━ ");
        assert_eq!(guides.last(), "┗━━ ");
        assert_eq!(guides.space(), "    ");
    }

    #[test]
    fn test_tree_guides_double() {
        let guides = TreeGuides::Double;
        assert_eq!(guides.vertical(), "║   ");
        assert_eq!(guides.branch(), "╠══ ");
        assert_eq!(guides.last(), "╚══ ");
        assert_eq!(guides.space(), "    ");
    }

    #[test]
    fn test_tree_guides_rounded() {
        let guides = TreeGuides::Rounded;
        assert_eq!(guides.vertical(), "│   ");
        assert_eq!(guides.branch(), "├── ");
        assert_eq!(guides.last(), "╰── "); // Rounded uses ╰
        assert_eq!(guides.space(), "    ");
    }
}
