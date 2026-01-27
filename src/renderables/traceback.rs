//! Traceback rendering.
//!
//! This is a Rust-idiomatic approximation of Python Rich's `rich.traceback`.
//! For deterministic testing and conformance fixtures, this implementation
//! supports rendering from **synthetic frames** (function name + line number).
//!
//! A higher-fidelity Rust backtrace capture layer can be built on top by
//! converting a backtrace to [`TracebackFrame`]s.

use crate::console::{Console, ConsoleOptions};
use crate::renderables::Renderable;
use crate::segment::Segment;
use crate::text::Text;

use super::panel::Panel;

/// A single traceback frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracebackFrame {
    pub name: String,
    pub line: usize,
}

impl TracebackFrame {
    #[must_use]
    pub fn new(name: impl Into<String>, line: usize) -> Self {
        Self {
            name: name.into(),
            line,
        }
    }
}

/// A rendered traceback, inspired by Python Rich.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Traceback {
    frames: Vec<TracebackFrame>,
    exception_type: String,
    exception_message: String,
    title: Text,
}

impl Traceback {
    /// Create a new traceback from frames and exception info.
    #[must_use]
    pub fn new(
        frames: impl Into<Vec<TracebackFrame>>,
        exception_type: impl Into<String>,
        exception_message: impl Into<String>,
    ) -> Self {
        Self {
            frames: frames.into(),
            exception_type: exception_type.into(),
            exception_message: exception_message.into(),
            title: Text::new("Traceback (most recent call last)"),
        }
    }

    /// Override the title shown in the traceback panel.
    #[must_use]
    pub fn title(mut self, title: impl Into<Text>) -> Self {
        self.title = title.into();
        self
    }

    /// Push a frame (builder-style).
    pub fn push_frame(&mut self, frame: TracebackFrame) {
        self.frames.push(frame);
    }
}

impl Renderable for Traceback {
    fn render<'a>(&'a self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        let width = options.max_width.max(1);

        let content_lines: Vec<Vec<Segment<'static>>> = self
            .frames
            .iter()
            .map(|frame| {
                vec![Segment::new(
                    format!("in {}:{}", frame.name, frame.line),
                    None,
                )]
            })
            .collect();

        let panel = Panel::new(content_lines)
            .title(self.title.clone())
            .width(width);
        let mut segments: Vec<Segment<'static>> = panel.render(width);

        segments.push(Segment::new(
            format!("{}: {}", self.exception_type, self.exception_message),
            None,
        ));
        segments.push(Segment::line());

        segments.into_iter().collect()
    }
}

/// Convenience helper mirroring Python Rich's `Console.print_exception`.
///
/// Rust doesn't have a standardized structured backtrace API across error
/// types; for now, this prints a provided [`Traceback`] renderable.
pub fn print_exception(console: &Console, traceback: &Traceback) {
    console.print_exception(traceback);
}
