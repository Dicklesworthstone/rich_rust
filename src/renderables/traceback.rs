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
    pub filename: Option<String>,
    pub name: String,
    pub line: usize,
}

impl TracebackFrame {
    #[must_use]
    pub fn new(name: impl Into<String>, line: usize) -> Self {
        Self {
            filename: None,
            name: name.into(),
            line,
        }
    }

    #[must_use]
    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }
}

/// A rendered traceback, inspired by Python Rich.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Traceback {
    frames: Vec<TracebackFrame>,
    exception_type: String,
    exception_message: String,
    title: Text,
    extra_lines: usize,
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
            extra_lines: 0,
        }
    }

    /// Override the title shown in the traceback panel.
    #[must_use]
    pub fn title(mut self, title: impl Into<Text>) -> Self {
        self.title = title.into();
        self
    }

    #[must_use]
    pub fn extra_lines(mut self, extra_lines: usize) -> Self {
        self.extra_lines = extra_lines;
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

        let mut content_lines: Vec<Vec<Segment<'static>>> = Vec::new();
        for frame in &self.frames {
            if let Some(filename) = frame.filename.as_deref() {
                if let Ok(source) = std::fs::read_to_string(filename) {
                    content_lines.push(vec![Segment::new(
                        format!("{filename}:{} in {}", frame.line, frame.name),
                        None,
                    )]);
                    content_lines.push(vec![Segment::new(String::new(), None)]);

                    let source_lines: Vec<&str> = source.lines().collect();
                    if frame.line > 0 && frame.line <= source_lines.len() {
                        let start = frame.line.saturating_sub(self.extra_lines).max(1);
                        let end = (frame.line + self.extra_lines).min(source_lines.len());
                        let line_number_width = end.to_string().len() + 5;

                        for line_no in start..=end {
                            let code = source_lines[line_no - 1].trim_start();
                            let indicator = if line_no == frame.line { "❱" } else { " " };
                            let line = format!("{indicator} {line_no:<line_number_width$}{code}");
                            content_lines.push(vec![Segment::new(line, None)]);
                        }
                    }

                    continue;
                }
            }

            content_lines.push(vec![Segment::new(
                format!("in {}:{}", frame.name, frame.line),
                None,
            )]);
        }

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
