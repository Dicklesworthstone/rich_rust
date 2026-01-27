use crate::console::{Console, ConsoleOptions};
use crate::emoji::{EmojiVariant, get as get_emoji};
use crate::renderables::Renderable;
use crate::segment::Segment;
use crate::style::Style;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoEmoji {
    name: String,
}

impl std::fmt::Display for NoEmoji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no emoji called {:?}", self.name)
    }
}

impl std::error::Error for NoEmoji {}

/// A single emoji character (Rich-style), optionally with a presentation variant.
#[derive(Debug, Clone)]
pub struct Emoji {
    name: String,
    style: Style,
    variant: Option<EmojiVariant>,
}

impl Emoji {
    /// Construct an Emoji by name (e.g. `"smile"`).
    ///
    /// Returns an error if the emoji name is not known.
    pub fn new(name: impl Into<String>) -> Result<Self, NoEmoji> {
        let name = name.into();
        if get_emoji(&name).is_none() {
            return Err(NoEmoji { name });
        }
        Ok(Self {
            name,
            style: Style::null(),
            variant: None,
        })
    }

    /// Set the style for this emoji.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the emoji presentation variant.
    #[must_use]
    pub fn variant(mut self, variant: Option<EmojiVariant>) -> Self {
        self.variant = variant;
        self
    }

    /// Get the emoji name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Renderable for Emoji {
    fn render<'a>(&'a self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment<'a>> {
        let Some(emoji) = get_emoji(&self.name) else {
            return vec![Segment::plain(format!(":{name}:", name = self.name))];
        };

        let selector = self.variant.map_or("", EmojiVariant::selector);
        let glyph = if selector.is_empty() {
            emoji.to_string()
        } else {
            let mut s = String::with_capacity(emoji.len() + selector.len());
            s.push_str(emoji);
            s.push_str(selector);
            s
        };

        let segment = if self.style.is_null() {
            Segment::plain(glyph)
        } else {
            Segment::styled(glyph, self.style.clone())
        };

        vec![segment]
    }
}
