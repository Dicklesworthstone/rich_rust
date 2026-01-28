//! Traceback rendering.
//!
//! This is a Rust-idiomatic approximation of Python Rich's `rich.traceback`.
//! For deterministic testing and conformance fixtures, this implementation
//! supports rendering from **synthetic frames** (function name + line number).
//!
//! # Automatic Capture (requires `backtrace` feature)
//!
//! When the `backtrace` feature is enabled, you can capture the current
//! call stack automatically:
//!
//! ```ignore
//! use rich_rust::renderables::{Traceback, TracebackFrame};
//!
//! // Capture current backtrace
//! let traceback = Traceback::capture("MyError", "something went wrong");
//! console.print_exception(&traceback);
//! ```

use crate::console::{Console, ConsoleOptions};
use crate::renderables::Renderable;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

use super::panel::Panel;

#[cfg(feature = "backtrace")]
use backtrace::Backtrace as BT;

/// A single traceback frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracebackFrame {
    pub filename: Option<String>,
    pub name: String,
    pub line: usize,
    /// Optional source code snippet for this frame.
    ///
    /// When provided, this is used instead of reading from the filesystem.
    /// This enables deterministic testing and rendering without file access.
    /// The snippet should contain lines around the error, with the error line
    /// being at position `line` (1-indexed relative to the start of the snippet's
    /// first line number, specified by `source_first_line`).
    pub source_context: Option<String>,
    /// The line number of the first line in `source_context`.
    /// Defaults to 1 if not specified.
    pub source_first_line: usize,
}

impl TracebackFrame {
    #[must_use]
    pub fn new(name: impl Into<String>, line: usize) -> Self {
        Self {
            filename: None,
            name: name.into(),
            line,
            source_context: None,
            source_first_line: 1,
        }
    }

    #[must_use]
    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Provide source context lines directly instead of reading from filesystem.
    ///
    /// This is useful for:
    /// - Deterministic testing without filesystem dependencies
    /// - Rendering tracebacks when source files are not available
    /// - Embedding source snippets from memory
    ///
    /// # Arguments
    /// * `source` - The source code snippet (may contain multiple lines)
    /// * `first_line` - The line number of the first line in the snippet
    ///
    /// # Example
    /// ```
    /// use rich_rust::renderables::TracebackFrame;
    ///
    /// let frame = TracebackFrame::new("my_function", 5)
    ///     .source_context("fn my_function() {\n    let x = 1;\n    panic!(\"oops\");\n}", 3);
    /// ```
    #[must_use]
    pub fn source_context(mut self, source: impl Into<String>, first_line: usize) -> Self {
        self.source_context = Some(source.into());
        self.source_first_line = first_line.max(1);
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

    /// Capture the current call stack and create a traceback.
    ///
    /// This is the primary way to create a Traceback from an actual runtime
    /// error. It captures the current backtrace and converts it to frames.
    ///
    /// # Arguments
    /// * `exception_type` - The type/name of the exception (e.g., `PanicError`)
    /// * `exception_message` - The error message
    ///
    /// # Example
    /// ```ignore
    /// let traceback = Traceback::capture("ConnectionError", "failed to connect");
    /// console.print_exception(&traceback);
    /// ```
    ///
    /// Requires the `backtrace` feature.
    #[cfg(feature = "backtrace")]
    #[must_use]
    pub fn capture(
        exception_type: impl Into<String>,
        exception_message: impl Into<String>,
    ) -> Self {
        let bt = BT::new();
        Self::from_backtrace(&bt, exception_type, exception_message)
    }

    /// Create a Traceback from an existing `backtrace::Backtrace`.
    ///
    /// This is useful when you have a backtrace from a panic handler or
    /// error type that provides its own backtrace.
    ///
    /// # Arguments
    /// * `bt` - The backtrace to convert
    /// * `exception_type` - The type/name of the exception
    /// * `exception_message` - The error message
    ///
    /// Requires the `backtrace` feature.
    #[cfg(feature = "backtrace")]
    #[must_use]
    pub fn from_backtrace(
        bt: &BT,
        exception_type: impl Into<String>,
        exception_message: impl Into<String>,
    ) -> Self {
        let frames = Self::parse_backtrace(bt);
        Self::new(frames, exception_type, exception_message)
    }

    /// Parse a backtrace into `TracebackFrame` list.
    ///
    /// Filters out runtime/std frames to show only relevant user code.
    #[cfg(feature = "backtrace")]
    fn parse_backtrace(bt: &BT) -> Vec<TracebackFrame> {
        let mut frames = Vec::new();
        let mut seen_user_code = false;

        for frame in bt.frames() {
            // Get symbols for this frame
            let symbols: Vec<_> = {
                let mut syms = Vec::new();
                backtrace::resolve(frame.ip(), |symbol| {
                    syms.push((
                        symbol.name().map(|n| n.to_string()),
                        symbol.filename().map(std::path::Path::to_path_buf),
                        symbol.lineno(),
                    ));
                });
                syms
            };

            for (name, filename, lineno) in symbols {
                let Some(name) = name else {
                    continue;
                };

                // Filter out internal/runtime frames
                if Self::is_internal_frame(&name) {
                    // Once we've seen user code, internal frames mark the end
                    if seen_user_code {
                        continue;
                    }
                    continue;
                }

                seen_user_code = true;

                let mut frame =
                    TracebackFrame::new(Self::demangle_name(&name), lineno.unwrap_or(0) as usize);

                if let Some(ref path) = filename {
                    frame = frame.filename(path.display().to_string());
                }

                frames.push(frame);
            }
        }

        // Reverse so most recent call is last (like Python)
        frames.reverse();
        frames
    }

    /// Check if a frame name is internal/runtime that should be filtered.
    #[cfg(feature = "backtrace")]
    fn is_internal_frame(name: &str) -> bool {
        // Filter common runtime prefixes
        let internal_prefixes = [
            "std::",
            "core::",
            "alloc::",
            "backtrace::",
            "<alloc::",
            "<core::",
            "<std::",
            "rust_begin_unwind",
            "__rust_",
            "_start",
            "main",
            "__libc_",
            "clone",
        ];

        for prefix in internal_prefixes {
            if name.starts_with(prefix) {
                return true;
            }
        }

        // Filter Traceback's own capture functions
        if name.contains("Traceback::capture") || name.contains("Traceback::from_backtrace") {
            return true;
        }

        false
    }

    /// Simplify/demangle a function name for display.
    #[cfg(feature = "backtrace")]
    fn demangle_name(name: &str) -> String {
        // The backtrace crate already demangles, but we can simplify further
        let name = name.to_string();

        // Remove hash suffixes like ::h1234567890abcdef
        if let Some(pos) = name.rfind("::h")
            && name[pos + 3..].chars().all(|c| c.is_ascii_hexdigit())
        {
            return name[..pos].to_string();
        }

        name
    }

    /// Get source for a frame, preferring provided context over filesystem.
    ///
    /// Returns `Some((source, first_line))` if source is available,
    /// `None` if no source can be obtained.
    fn get_frame_source(&self, frame: &TracebackFrame) -> Option<(String, usize)> {
        // Priority 1: Use provided source context
        if let Some(ref source) = frame.source_context {
            return Some((source.clone(), frame.source_first_line));
        }

        // Priority 2: Read from filesystem if filename is provided
        if let Some(ref filename) = frame.filename
            && let Ok(source) = std::fs::read_to_string(filename)
        {
            return Some((source, 1));
        }

        None
    }
}

impl Renderable for Traceback {
    fn render<'a>(&'a self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment<'a>> {
        let width = options.max_width.max(1);

        // Define styles for traceback components
        let file_style = Style::parse("cyan").ok();
        let lineno_style = Style::parse("yellow").ok();
        let func_style = Style::parse("green").ok();
        let dim_style = Style::parse("dim").ok();
        let error_line_style = Style::parse("bold").ok();
        let exception_type_style = Style::parse("bold red").ok();
        let exception_msg_style = Style::parse("red").ok();

        let mut content_lines: Vec<Vec<Segment<'static>>> = Vec::new();
        for frame in &self.frames {
            // Try to get source: first from provided context, then from filesystem
            let source_result = self.get_frame_source(frame);

            if let Some((source, first_line)) = source_result {
                // Render frame header with location info using styled segments
                if let Some(filename) = frame.filename.as_deref() {
                    content_lines.push(vec![
                        Segment::new(filename.to_string(), file_style.clone()),
                        Segment::new(":", dim_style.clone()),
                        Segment::new(frame.line.to_string(), lineno_style.clone()),
                        Segment::new(" in ", dim_style.clone()),
                        Segment::new(frame.name.clone(), func_style.clone()),
                    ]);
                } else {
                    content_lines.push(vec![
                        Segment::new("in ", dim_style.clone()),
                        Segment::new(frame.name.clone(), func_style.clone()),
                        Segment::new(":", dim_style.clone()),
                        Segment::new(frame.line.to_string(), lineno_style.clone()),
                    ]);
                }
                content_lines.push(vec![Segment::new(String::new(), None)]);

                // Render source context with line numbers
                let source_lines: Vec<&str> = source.lines().collect();
                let last_line = first_line + source_lines.len().saturating_sub(1);

                // Calculate which lines to show based on extra_lines
                let start = frame.line.saturating_sub(self.extra_lines).max(first_line);
                let end = (frame.line + self.extra_lines).min(last_line);

                if start <= end && frame.line >= first_line && frame.line <= last_line {
                    let line_number_width = end.to_string().len() + 5;

                    for line_no in start..=end {
                        let source_idx = line_no.saturating_sub(first_line);
                        if source_idx < source_lines.len() {
                            let code = source_lines[source_idx].trim_start();
                            let is_error_line = line_no == frame.line;
                            let indicator = if is_error_line { "❱" } else { " " };
                            let line_style = if is_error_line {
                                error_line_style.clone()
                            } else {
                                None
                            };

                            content_lines.push(vec![
                                Segment::new(
                                    indicator.to_string(),
                                    if is_error_line {
                                        Some(Style::parse("bold red").unwrap_or_default())
                                    } else {
                                        dim_style.clone()
                                    },
                                ),
                                Segment::new(" ", None),
                                Segment::new(
                                    format!("{line_no:<line_number_width$}"),
                                    lineno_style.clone(),
                                ),
                                Segment::new(code.to_string(), line_style),
                            ]);
                        }
                    }
                }

                continue;
            }

            // Fallback: no source available, just show frame info
            content_lines.push(vec![
                Segment::new("in ", dim_style.clone()),
                Segment::new(frame.name.clone(), func_style.clone()),
                Segment::new(":", dim_style.clone()),
                Segment::new(frame.line.to_string(), lineno_style.clone()),
            ]);
        }

        let panel = Panel::new(content_lines)
            .title(self.title.clone())
            .border_style(Style::parse("red").unwrap_or_default())
            .width(width);
        let mut segments: Vec<Segment<'static>> = panel.render(width);

        // Exception info with styling
        segments.push(Segment::new(
            self.exception_type.clone(),
            exception_type_style.clone(),
        ));
        segments.push(Segment::new(": ".to_string(), None));
        segments.push(Segment::new(
            self.exception_message.clone(),
            exception_msg_style.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn render_to_text(traceback: &Traceback, width: usize) -> String {
        let console = Console::new();
        let options = ConsoleOptions {
            max_width: width,
            ..Default::default()
        };
        let segments = traceback.render(&console, &options);
        segments.iter().map(|s| s.text.as_ref()).collect()
    }

    #[test]
    fn frame_without_source_shows_minimal_info() {
        let frame = TracebackFrame::new("my_func", 42);
        let traceback = Traceback::new(vec![frame], "Error", "something went wrong");

        let output = render_to_text(&traceback, 80);
        assert!(output.contains("my_func"));
        assert!(output.contains("42"));
        assert!(output.contains("Error: something went wrong"));
    }

    #[test]
    fn frame_with_source_context_renders_code() {
        let source = "fn main() {\n    let x = 1;\n    panic!(\"oops\");\n    let y = 2;\n}";
        let frame = TracebackFrame::new("main", 3).source_context(source, 1);
        let traceback = Traceback::new(vec![frame], "PanicError", "oops").extra_lines(1);

        let output = render_to_text(&traceback, 80);

        // Should show the error line with indicator
        assert!(output.contains("❱"));
        assert!(output.contains("panic!"));

        // Should show context lines (extra_lines=1)
        assert!(output.contains("let x = 1"));
        assert!(output.contains("let y = 2"));

        // Should show exception info
        assert!(output.contains("PanicError: oops"));
    }

    #[test]
    fn source_context_with_offset_first_line() {
        // Simulating a snippet from lines 10-14 of a larger file
        let source = "    fn helper() {\n        do_thing();\n        crash_here();\n    }\n";
        let frame = TracebackFrame::new("helper", 12).source_context(source, 10);
        let traceback = Traceback::new(vec![frame], "Error", "crashed").extra_lines(1);

        let output = render_to_text(&traceback, 80);

        // Should show line 12 with indicator
        assert!(output.contains("❱"));
        assert!(output.contains("12"));
        assert!(output.contains("crash_here"));

        // Should show context (lines 11 and 13)
        assert!(output.contains("11"));
        assert!(output.contains("do_thing"));
    }

    #[test]
    fn source_context_takes_priority_over_filename() {
        // Even if filename is set, source_context should be used
        let source = "custom source line";
        let frame = TracebackFrame::new("func", 1)
            .filename("/nonexistent/file.rs")
            .source_context(source, 1);
        let traceback = Traceback::new(vec![frame], "Error", "test");

        let output = render_to_text(&traceback, 80);

        // Should render the provided source, not try to read file
        assert!(output.contains("custom source line"));
        // Should still show filename in header
        assert!(output.contains("/nonexistent/file.rs"));
    }

    #[test]
    fn extra_lines_zero_shows_only_error_line() {
        let source = "line1\nline2\nline3\nline4\nline5";
        let frame = TracebackFrame::new("func", 3).source_context(source, 1);
        let traceback = Traceback::new(vec![frame], "Error", "test").extra_lines(0);

        let output = render_to_text(&traceback, 80);

        // Should show only line 3
        assert!(output.contains("line3"));
        assert!(output.contains("❱"));
        // Should not show other lines
        assert!(!output.contains("line1"));
        assert!(!output.contains("line5"));
    }

    #[test]
    fn multiple_frames_with_source_context() {
        let frame1 =
            TracebackFrame::new("outer", 2).source_context("fn outer() {\n    inner();\n}", 1);
        let frame2 =
            TracebackFrame::new("inner", 2).source_context("fn inner() {\n    panic!();\n}", 1);

        let traceback = Traceback::new(vec![frame1, frame2], "PanicError", "boom");

        let output = render_to_text(&traceback, 80);

        // Both frames should be rendered
        assert!(output.contains("outer"));
        assert!(output.contains("inner"));
        assert!(output.contains("PanicError: boom"));
    }

    #[test]
    fn frame_builder_methods() {
        let frame = TracebackFrame::new("test", 10)
            .filename("src/lib.rs")
            .source_context("test code", 5);

        assert_eq!(frame.name, "test");
        assert_eq!(frame.line, 10);
        assert_eq!(frame.filename, Some("src/lib.rs".to_string()));
        assert_eq!(frame.source_context, Some("test code".to_string()));
        assert_eq!(frame.source_first_line, 5);
    }

    #[test]
    fn source_first_line_minimum_is_one() {
        let frame = TracebackFrame::new("test", 1).source_context("code", 0);
        assert_eq!(frame.source_first_line, 1);
    }

    #[cfg(feature = "backtrace")]
    mod backtrace_tests {
        use super::*;

        fn inner_function() -> Traceback {
            Traceback::capture("TestError", "test message")
        }

        fn outer_function() -> Traceback {
            inner_function()
        }

        #[test]
        fn capture_creates_traceback_with_frames() {
            let traceback = outer_function();

            // Should have at least some frames (our functions)
            assert!(!traceback.frames.is_empty(), "should capture frames");

            // Exception info should be set
            assert_eq!(traceback.exception_type, "TestError");
            assert_eq!(traceback.exception_message, "test message");
        }

        #[test]
        fn capture_filters_internal_frames() {
            let traceback = Traceback::capture("Error", "test");

            // Should not contain std/core frames
            for frame in &traceback.frames {
                assert!(
                    !frame.name.starts_with("std::"),
                    "should filter std:: frames: {}",
                    frame.name
                );
                assert!(
                    !frame.name.starts_with("core::"),
                    "should filter core:: frames: {}",
                    frame.name
                );
            }
        }

        #[test]
        fn is_internal_frame_detects_runtime() {
            assert!(Traceback::is_internal_frame("std::rt::lang_start"));
            assert!(Traceback::is_internal_frame("core::ops::function::FnOnce"));
            assert!(Traceback::is_internal_frame("__libc_start_main"));
            assert!(Traceback::is_internal_frame("main"));
            assert!(!Traceback::is_internal_frame("my_crate::my_function"));
            assert!(!Traceback::is_internal_frame("app::handler::process"));
        }

        #[test]
        fn demangle_removes_hash_suffix() {
            assert_eq!(
                Traceback::demangle_name("my_crate::func::h1234567890abcdef"),
                "my_crate::func"
            );
            assert_eq!(Traceback::demangle_name("my_crate::func"), "my_crate::func");
        }

        #[test]
        fn capture_renders_without_panic() {
            let traceback = Traceback::capture("PanicError", "something went wrong");
            let output = render_to_text(&traceback, 100);

            assert!(output.contains("PanicError: something went wrong"));
            assert!(output.contains("Traceback"));
        }
    }
}
