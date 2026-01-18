//! Console - the central entry point for styled terminal output.
//!
//! The [`Console`] handles rendering styled content to the terminal,
//! including color detection, width calculation, and ANSI code generation.
//!
//! # Examples
//!
//! ## Basic Printing with Markup
//!
//! ```rust,ignore
//! use rich_rust::Console;
//!
//! let console = Console::new();
//!
//! // Print with markup syntax
//! console.print("[bold red]Error:[/] Something went wrong");
//! console.print("[green]Success![/] Operation completed");
//!
//! // Markup supports colors, attributes, and combinations
//! console.print("[bold italic #ff8800 on blue]Custom styling[/]");
//! ```
//!
//! ## Console Builder
//!
//! ```rust,ignore
//! use rich_rust::console::{Console, ConsoleBuilder};
//! use rich_rust::color::ColorSystem;
//!
//! let console = Console::builder()
//!     .color_system(ColorSystem::EightBit)  // Force 256 colors
//!     .width(80)                            // Fixed width
//!     .markup(true)                         // Enable markup parsing
//!     .build();
//! ```
//!
//! ## Print Options
//!
//! ```rust,ignore
//! use rich_rust::console::{Console, PrintOptions};
//! use rich_rust::style::Style;
//! use rich_rust::text::JustifyMethod;
//!
//! let console = Console::new();
//!
//! let options = PrintOptions::new()
//!     .with_style(Style::new().bold())
//!     .with_justify(JustifyMethod::Center)
//!     .with_markup(true);
//!
//! console.print_with_options("Centered bold text", &options);
//! ```
//!
//! ## Capturing Output
//!
//! ```rust,ignore
//! use rich_rust::Console;
//!
//! let mut console = Console::new();
//!
//! // Start capturing
//! console.begin_capture();
//! console.print("[bold]Hello[/]");
//!
//! // Get captured segments
//! let segments = console.end_capture();
//! for seg in &segments {
//!     println!("Text: {:?}, Style: {:?}", seg.text, seg.style);
//! }
//! ```
//!
//! # Terminal Detection
//!
//! The Console automatically detects terminal capabilities:
//!
//! - **Color system**: `TrueColor` (24-bit), 256 colors, or 16 colors
//! - **Terminal dimensions**: Width and height in character cells
//! - **TTY status**: Whether output is to an interactive terminal
//!
//! You can override these with the builder pattern or by setting explicit values.

use std::cell::{Cell, RefCell};
use std::io::{self, Write};

use crate::color::ColorSystem;
use crate::markup;
use crate::segment::Segment;
use crate::style::Style;
use crate::terminal;
use crate::text::{JustifyMethod, OverflowMethod, Text};

/// Console dimensions in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsoleDimensions {
    /// Width in cells.
    pub width: usize,
    /// Height in rows.
    pub height: usize,
}

impl Default for ConsoleDimensions {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
        }
    }
}

/// Options for rendering.
#[derive(Debug, Clone)]
pub struct ConsoleOptions {
    /// Terminal dimensions.
    pub size: ConsoleDimensions,
    /// Using legacy Windows console.
    pub legacy_windows: bool,
    /// Minimum width constraint.
    pub min_width: usize,
    /// Maximum width constraint.
    pub max_width: usize,
    /// Output is a terminal (vs file/pipe).
    pub is_terminal: bool,
    /// Output encoding.
    pub encoding: String,
    /// Maximum height for rendering.
    pub max_height: usize,
    /// Default justification.
    pub justify: Option<JustifyMethod>,
    /// Default overflow handling.
    pub overflow: Option<OverflowMethod>,
    /// Default `no_wrap` setting.
    pub no_wrap: Option<bool>,
    /// Enable highlighting.
    pub highlight: Option<bool>,
    /// Parse markup in strings.
    pub markup: Option<bool>,
    /// Explicit height override.
    pub height: Option<usize>,
}

impl Default for ConsoleOptions {
    fn default() -> Self {
        Self {
            size: ConsoleDimensions::default(),
            legacy_windows: false,
            min_width: 1,
            max_width: 80,
            is_terminal: true,
            encoding: String::from("utf-8"),
            max_height: usize::MAX,
            justify: None,
            overflow: None,
            no_wrap: None,
            highlight: None,
            markup: None,
            height: None,
        }
    }
}

impl ConsoleOptions {
    /// Create options with a different `max_width`.
    #[must_use]
    pub fn update_width(&self, width: usize) -> Self {
        Self {
            max_width: width.min(self.max_width),
            ..self.clone()
        }
    }

    /// Create options with a different height.
    #[must_use]
    pub fn update_height(&self, height: usize) -> Self {
        Self {
            height: Some(height),
            ..self.clone()
        }
    }
}

/// Print options for controlling output.
#[derive(Debug, Clone, Default)]
pub struct PrintOptions {
    /// String to separate multiple objects.
    pub sep: String,
    /// String to append at end.
    pub end: String,
    /// Apply style to output.
    pub style: Option<Style>,
    /// Override justification.
    pub justify: Option<JustifyMethod>,
    /// Override overflow handling.
    pub overflow: Option<OverflowMethod>,
    /// Override `no_wrap`.
    pub no_wrap: Option<bool>,
    /// Suppress newline.
    pub no_newline: bool,
    /// Parse markup.
    pub markup: Option<bool>,
    /// Enable highlighting.
    pub highlight: bool,
    /// Override width.
    pub width: Option<usize>,
    /// Crop output to width.
    pub crop: bool,
    /// Soft wrap at width.
    pub soft_wrap: bool,
}

impl PrintOptions {
    /// Create new print options with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sep: String::from(" "),
            end: String::from("\n"),
            ..Default::default()
        }
    }

    /// Set markup parsing.
    #[must_use]
    pub fn with_markup(mut self, markup: bool) -> Self {
        self.markup = Some(markup);
        self
    }

    /// Set style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Set the separator between objects.
    #[must_use]
    pub fn with_sep(mut self, sep: impl Into<String>) -> Self {
        self.sep = sep.into();
        self
    }

    /// Set the end string appended after output.
    #[must_use]
    pub fn with_end(mut self, end: impl Into<String>) -> Self {
        self.end = end.into();
        self
    }

    /// Override justification.
    #[must_use]
    pub fn with_justify(mut self, justify: JustifyMethod) -> Self {
        self.justify = Some(justify);
        self
    }

    /// Override overflow handling.
    #[must_use]
    pub fn with_overflow(mut self, overflow: OverflowMethod) -> Self {
        self.overflow = Some(overflow);
        self
    }

    /// Override `no_wrap`.
    #[must_use]
    pub fn with_no_wrap(mut self, no_wrap: bool) -> Self {
        self.no_wrap = Some(no_wrap);
        self
    }

    /// Suppress newline at end.
    #[must_use]
    pub fn with_no_newline(mut self, no_newline: bool) -> Self {
        self.no_newline = no_newline;
        self
    }

    /// Enable/disable highlighting.
    #[must_use]
    pub fn with_highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
    }

    /// Override width.
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Crop output to width.
    #[must_use]
    pub fn with_crop(mut self, crop: bool) -> Self {
        self.crop = crop;
        self
    }

    /// Soft wrap at width.
    #[must_use]
    pub fn with_soft_wrap(mut self, soft_wrap: bool) -> Self {
        self.soft_wrap = soft_wrap;
        self
    }
}

/// The main Console for rendering styled output.
pub struct Console {
    /// Color system to use (None = auto-detect).
    color_system: Option<ColorSystem>,
    /// Force terminal mode.
    force_terminal: Option<bool>,
    /// Tab expansion size.
    tab_size: usize,
    /// Buffer output for export.
    record: Cell<bool>,
    /// Parse markup by default.
    markup: bool,
    /// Enable emoji rendering.
    emoji: bool,
    /// Enable syntax highlighting.
    highlight: bool,
    /// Override width.
    width: Option<usize>,
    /// Override height.
    height: Option<usize>,
    /// Use ASCII-safe box characters.
    safe_box: bool,
    /// Output stream (defaults to stdout).
    file: RefCell<Box<dyn Write + Send>>,
    /// Recording buffer.
    buffer: RefCell<Vec<Segment>>,
    /// Cached terminal detection.
    is_terminal: bool,
    /// Detected/configured color system.
    detected_color_system: Option<ColorSystem>,
}

impl std::fmt::Debug for Console {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Console")
            .field("color_system", &self.color_system)
            .field("force_terminal", &self.force_terminal)
            .field("tab_size", &self.tab_size)
            .field("record", &self.record.get())
            .field("markup", &self.markup)
            .field("emoji", &self.emoji)
            .field("highlight", &self.highlight)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("safe_box", &self.safe_box)
            .field("file", &"<dyn Write>")
            .field("buffer_len", &self.buffer.borrow().len())
            .field("is_terminal", &self.is_terminal)
            .field("detected_color_system", &self.detected_color_system)
            .finish()
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

impl Console {
    /// Create a new console with default settings.
    #[must_use]
    pub fn new() -> Self {
        let is_terminal = terminal::is_terminal();
        let detected_color_system = if is_terminal {
            terminal::detect_color_system()
        } else {
            None
        };

        Self {
            color_system: None,
            force_terminal: None,
            tab_size: 8,
            record: Cell::new(false),
            markup: true,
            emoji: true,
            highlight: true,
            width: None,
            height: None,
            safe_box: false,
            file: RefCell::new(Box::new(io::stdout())),
            buffer: RefCell::new(Vec::new()),
            is_terminal,
            detected_color_system,
        }
    }

    /// Create a console builder for custom configuration.
    #[must_use]
    pub fn builder() -> ConsoleBuilder {
        ConsoleBuilder::default()
    }

    /// Get the console width.
    #[must_use]
    pub fn width(&self) -> usize {
        self.width.unwrap_or_else(terminal::get_terminal_width)
    }

    /// Get the console height.
    #[must_use]
    pub fn height(&self) -> usize {
        self.height.unwrap_or_else(terminal::get_terminal_height)
    }

    /// Get the console dimensions.
    #[must_use]
    pub fn size(&self) -> ConsoleDimensions {
        ConsoleDimensions {
            width: self.width(),
            height: self.height(),
        }
    }

    /// Check if this console outputs to a terminal.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.force_terminal.unwrap_or(self.is_terminal)
    }

    /// Get the color system in use.
    #[must_use]
    pub fn color_system(&self) -> Option<ColorSystem> {
        self.color_system.or(self.detected_color_system)
    }

    /// Check if colors are enabled.
    #[must_use]
    pub fn is_color_enabled(&self) -> bool {
        self.color_system().is_some()
    }

    /// Get the tab size.
    #[must_use]
    pub const fn tab_size(&self) -> usize {
        self.tab_size
    }

    /// Create console options for rendering.
    #[must_use]
    pub fn options(&self) -> ConsoleOptions {
        ConsoleOptions {
            size: self.size(),
            legacy_windows: false,
            min_width: 1,
            max_width: self.width(),
            is_terminal: self.is_terminal(),
            encoding: String::from("utf-8"),
            max_height: self.height(),
            justify: None,
            overflow: None,
            no_wrap: None,
            highlight: Some(self.highlight),
            markup: Some(self.markup),
            height: None,
        }
    }

    /// Enable recording mode.
    pub fn begin_capture(&mut self) {
        self.record.set(true);
        self.buffer.borrow_mut().clear();
    }

    /// End recording and return captured segments.
    pub fn end_capture(&mut self) -> Vec<Segment> {
        self.record.set(false);
        std::mem::take(&mut *self.buffer.borrow_mut())
    }

    /// Print styled text to the console.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rich_rust::Console;
    ///
    /// let console = Console::new();
    /// console.print("[bold red]Hello[/] World!");
    /// ```
    pub fn print(&self, content: &str) {
        self.print_with_options(content, &PrintOptions::new().with_markup(self.markup));
    }

    /// Print a prepared Text object.
    pub fn print_text(&self, text: &Text) {
        let mut file = self.file.borrow_mut();
        let _ = self.print_text_to(&mut *file, text);
    }

    /// Print a prepared Text object to a specific writer.
    pub fn print_text_to<W: Write>(&self, writer: &mut W, text: &Text) -> io::Result<()> {
        let segments = text.render(&text.end);
        self.write_segments(writer, &segments)
    }

    /// Print prepared segments.
    pub fn print_segments(&self, segments: &[Segment]) {
        let mut file = self.file.borrow_mut();
        let _ = self.print_segments_to(&mut *file, segments);
    }

    /// Print prepared segments to a specific writer.
    pub fn print_segments_to<W: Write>(
        &self,
        writer: &mut W,
        segments: &[Segment],
    ) -> io::Result<()> {
        self.write_segments(writer, segments)
    }

    /// Print with custom options.
    pub fn print_with_options(&self, content: &str, options: &PrintOptions) {
        let mut file = self.file.borrow_mut();
        self.print_to(&mut *file, content, options)
            .expect("failed to write to output stream");
    }

    /// Print to a specific writer.
    pub fn print_to<W: Write>(
        &self,
        writer: &mut W,
        content: &str,
        options: &PrintOptions,
    ) -> io::Result<()> {
        // Parse markup if enabled
        let parse_markup = options.markup.unwrap_or(self.markup);
        let mut text = if parse_markup {
            markup::render_or_plain(content)
        } else {
            Text::new(content)
        };

        if let Some(justify) = options.justify {
            text.justify = justify;
        }
        if let Some(overflow) = options.overflow {
            text.overflow = overflow;
        }
        if let Some(no_wrap) = options.no_wrap {
            text.no_wrap = no_wrap;
        }
        if options.crop {
            text.overflow = OverflowMethod::Crop;
        }

        let width = options.width.or_else(|| {
            if options.justify.is_some()
                || options.overflow.is_some()
                || options.no_wrap.is_some()
                || options.crop
                || options.soft_wrap
            {
                Some(self.width())
            } else {
                None
            }
        });

        let end = if options.no_newline { "" } else { &options.end };
        let segments = if let Some(width) = width {
            let mut rendered = Vec::new();
            let lines = if text.no_wrap {
                text.split_lines()
            } else {
                text.wrap(width)
            };
            let last_index = lines.len().saturating_sub(1);
            let justify = match text.justify {
                JustifyMethod::Default => JustifyMethod::Left,
                other => other,
            };

            for (index, mut line) in lines.into_iter().enumerate() {
                if text.no_wrap && line.cell_len() > width {
                    line.truncate(width, line.overflow, false);
                }

                if matches!(
                    justify,
                    JustifyMethod::Center | JustifyMethod::Right | JustifyMethod::Full
                ) && line.cell_len() < width
                {
                    line.pad(width, justify);
                }

                let line_end = if index == last_index { end } else { "\n" };
                rendered.extend(line.render(line_end));
            }

            rendered
        } else {
            text.render(end)
        };

        // Apply any overall style
        let segments: Vec<Segment> = if let Some(ref style) = options.style {
            segments
                .into_iter()
                .map(|mut seg| {
                    if !seg.is_control() {
                        seg.style = Some(match seg.style {
                            Some(s) => style.combine(&s),
                            None => style.clone(),
                        });
                    }
                    seg
                })
                .collect()
        } else {
            segments
        };

        // Write segments
        self.write_segments(writer, &segments)
    }

    /// Write segments to a writer.
    fn write_segments<W: Write>(&self, writer: &mut W, segments: &[Segment]) -> io::Result<()> {
        if self.record.get() {
            self.buffer.borrow_mut().extend(segments.iter().cloned());
        }

        let color_system = self.color_system();

        for segment in segments {
            if segment.is_control() {
                self.write_control_codes(writer, segment)?;
                continue;
            }

            // Get ANSI codes for style
            let (prefix, suffix) = if let Some(ref style) = segment.style {
                if let Some(cs) = color_system {
                    style.render_ansi(cs)
                } else {
                    (String::new(), String::new())
                }
            } else {
                (String::new(), String::new())
            };

            // Write styled text
            write!(writer, "{prefix}{}{suffix}", segment.text)?;
        }

        writer.flush()
    }

    fn write_control_codes<W: Write>(&self, writer: &mut W, segment: &Segment) -> io::Result<()> {
        let Some(ref controls) = segment.control else {
            return Ok(());
        };

        for control in controls {
            match control.control_type {
                crate::segment::ControlType::Bell => {
                    write!(writer, "\x07")?;
                }
                crate::segment::ControlType::CarriageReturn => {
                    write!(writer, "\r")?;
                }
                crate::segment::ControlType::Home => {
                    write!(writer, "\x1b[H")?;
                }
                crate::segment::ControlType::Clear => {
                    write!(writer, "\x1b[2J")?;
                }
                crate::segment::ControlType::ShowCursor => {
                    write!(writer, "\x1b[?25h")?;
                }
                crate::segment::ControlType::HideCursor => {
                    write!(writer, "\x1b[?25l")?;
                }
                crate::segment::ControlType::EnableAltScreen => {
                    write!(writer, "\x1b[?1049h")?;
                }
                crate::segment::ControlType::DisableAltScreen => {
                    write!(writer, "\x1b[?1049l")?;
                }
                crate::segment::ControlType::CursorUp => {
                    let n = control_param(&control.params, 0, 1);
                    write!(writer, "\x1b[{n}A")?;
                }
                crate::segment::ControlType::CursorDown => {
                    let n = control_param(&control.params, 0, 1);
                    write!(writer, "\x1b[{n}B")?;
                }
                crate::segment::ControlType::CursorForward => {
                    let n = control_param(&control.params, 0, 1);
                    write!(writer, "\x1b[{n}C")?;
                }
                crate::segment::ControlType::CursorBackward => {
                    let n = control_param(&control.params, 0, 1);
                    write!(writer, "\x1b[{n}D")?;
                }
                crate::segment::ControlType::CursorMoveToColumn => {
                    let column = control_param(&control.params, 0, 1);
                    write!(writer, "\x1b[{column}G")?;
                }
                crate::segment::ControlType::CursorMoveTo => {
                    let row = control_param(&control.params, 0, 1);
                    let column = control_param(&control.params, 1, 1);
                    write!(writer, "\x1b[{row};{column}H")?;
                }
                crate::segment::ControlType::EraseInLine => {
                    let mode = erase_in_line_mode(&control.params);
                    write!(writer, "\x1b[{mode}K")?;
                }
                crate::segment::ControlType::SetWindowTitle => {
                    if let Some(title) = control_title(segment, control) {
                        write!(writer, "\x1b]0;{title}\x07")?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Print a blank line.
    pub fn line(&self) {
        let mut file = self.file.borrow_mut();
        let _ = writeln!(file);
    }

    /// Print a rule (horizontal line).
    pub fn rule(&self, title: Option<&str>) {
        let width = self.width();
        let line_char = if self.safe_box { '-' } else { '\u{2500}' };

        let mut file = self.file.borrow_mut();
        if let Some(title) = title {
            let title_len = crate::cells::cell_len(title);
            let available = width.saturating_sub(title_len + 2);
            let left_pad = available / 2;
            let right_pad = available - left_pad;
            let left = line_char.to_string().repeat(left_pad);
            let right = line_char.to_string().repeat(right_pad);
            let _ = writeln!(file, "{left} {title} {right}");
        } else {
            let _ = writeln!(file, "{}", line_char.to_string().repeat(width));
        }
    }

    /// Clear the screen.
    pub fn clear(&self) {
        let mut file = self.file.borrow_mut();
        let _ = terminal::control::clear_screen(&mut *file);
    }

    /// Clear the current line.
    pub fn clear_line(&self) {
        let mut file = self.file.borrow_mut();
        let _ = terminal::control::clear_line(&mut *file);
    }

    /// Set the terminal title.
    pub fn set_title(&self, title: &str) {
        let mut file = self.file.borrow_mut();
        let _ = terminal::control::set_title(&mut *file, title);
    }

    /// Ring the terminal bell.
    pub fn bell(&self) {
        let mut file = self.file.borrow_mut();
        let _ = terminal::control::bell(&mut *file);
    }

    /// Print text without parsing markup.
    pub fn print_plain(&self, content: &str) {
        self.print_with_options(content, &PrintOptions::new().with_markup(false));
    }

    /// Print a styled message.
    pub fn print_styled(&self, content: &str, style: Style) {
        self.print_with_options(
            content,
            &PrintOptions::new()
                .with_markup(self.markup)
                .with_style(style),
        );
    }

    /// Print a log message with a level indicator.
    pub fn log(&self, message: &str, level: LogLevel) {
        let (prefix, style) = match level {
            LogLevel::Debug => ("[DEBUG]", Style::parse("cyan").unwrap_or_default()),
            LogLevel::Info => ("[INFO]", Style::parse("green").unwrap_or_default()),
            LogLevel::Warning => ("[WARNING]", Style::parse("yellow").unwrap_or_default()),
            LogLevel::Error => ("[ERROR]", Style::parse("bold red").unwrap_or_default()),
        };

        let mut file = self.file.borrow_mut();
        let _ = self.print_to(
            &mut *file,
            prefix,
            &PrintOptions::new().with_markup(false).with_style(style),
        );
        let _ = write!(file, " ");
        let _ = self.print_to(
            &mut *file,
            message,
            &PrintOptions::new().with_markup(self.markup),
        );
    }
}

fn control_param(params: &[i32], index: usize, default: i32) -> i32 {
    params
        .get(index)
        .copied()
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn erase_in_line_mode(params: &[i32]) -> i32 {
    if let Some(value) = params.first().copied()
        && (0..=2).contains(&value)
    {
        return value;
    }
    2
}

fn control_title(segment: &Segment, control: &crate::segment::ControlCode) -> Option<String> {
    if !segment.text.is_empty() {
        return Some(segment.text.clone());
    }

    if control.params.is_empty() {
        return None;
    }

    let mut title = String::with_capacity(control.params.len());
    for param in &control.params {
        if let Ok(byte) = u8::try_from(*param) {
            title.push(byte as char);
        }
    }

    if title.is_empty() { None } else { Some(title) }
}

/// Log level for `console.log()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// Builder for creating a Console with custom settings.
#[derive(Default)]
pub struct ConsoleBuilder {
    color_system: Option<ColorSystem>,
    force_terminal: Option<bool>,
    tab_size: Option<usize>,
    markup: Option<bool>,
    emoji: Option<bool>,
    highlight: Option<bool>,
    width: Option<usize>,
    height: Option<usize>,
    safe_box: Option<bool>,
    file: Option<Box<dyn Write + Send>>,
}

impl std::fmt::Debug for ConsoleBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsoleBuilder")
            .field("color_system", &self.color_system)
            .field("force_terminal", &self.force_terminal)
            .field("tab_size", &self.tab_size)
            .field("markup", &self.markup)
            .field("emoji", &self.emoji)
            .field("highlight", &self.highlight)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("safe_box", &self.safe_box)
            .field("file", &self.file.as_ref().map(|_| "<dyn Write>"))
            .finish()
    }
}

impl ConsoleBuilder {
    /// Set the color system.
    #[must_use]
    pub fn color_system(mut self, system: ColorSystem) -> Self {
        self.color_system = Some(system);
        self
    }

    /// Disable colors.
    #[must_use]
    pub fn no_color(mut self) -> Self {
        self.color_system = None;
        self
    }

    /// Force terminal mode.
    #[must_use]
    pub fn force_terminal(mut self, force: bool) -> Self {
        self.force_terminal = Some(force);
        self
    }

    /// Set tab size.
    #[must_use]
    pub fn tab_size(mut self, size: usize) -> Self {
        self.tab_size = Some(size);
        self
    }

    /// Enable/disable markup parsing.
    #[must_use]
    pub fn markup(mut self, enabled: bool) -> Self {
        self.markup = Some(enabled);
        self
    }

    /// Enable/disable emoji.
    #[must_use]
    pub fn emoji(mut self, enabled: bool) -> Self {
        self.emoji = Some(enabled);
        self
    }

    /// Enable/disable highlighting.
    #[must_use]
    pub fn highlight(mut self, enabled: bool) -> Self {
        self.highlight = Some(enabled);
        self
    }

    /// Set console width.
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set console height.
    #[must_use]
    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    /// Use ASCII-safe box characters.
    #[must_use]
    pub fn safe_box(mut self, safe: bool) -> Self {
        self.safe_box = Some(safe);
        self
    }

    /// Set the output stream.
    #[must_use]
    pub fn file(mut self, writer: Box<dyn Write + Send>) -> Self {
        self.file = Some(writer);
        self
    }

    /// Build the console.
    #[must_use]
    pub fn build(self) -> Console {
        let mut console = Console::new();

        if let Some(cs) = self.color_system {
            console.color_system = Some(cs);
        }
        if let Some(ft) = self.force_terminal {
            console.force_terminal = Some(ft);
        }
        if let Some(ts) = self.tab_size {
            console.tab_size = ts;
        }
        if let Some(m) = self.markup {
            console.markup = m;
        }
        if let Some(e) = self.emoji {
            console.emoji = e;
        }
        if let Some(h) = self.highlight {
            console.highlight = h;
        }
        if let Some(w) = self.width {
            console.width = Some(w);
        }
        if let Some(h) = self.height {
            console.height = Some(h);
        }
        if let Some(sb) = self.safe_box {
            console.safe_box = sb;
        }
        if let Some(f) = self.file {
            console.file = RefCell::new(f);
        }

        console
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_new() {
        let console = Console::new();
        assert!(console.width() > 0);
        assert!(console.height() > 0);
    }

    #[test]
    fn test_console_builder() {
        let console = Console::builder()
            .width(100)
            .height(50)
            .markup(false)
            .build();

        assert_eq!(console.width(), 100);
        assert_eq!(console.height(), 50);
        assert!(!console.markup);
    }

    #[test]
    fn test_console_options() {
        let console = Console::builder().width(80).build();
        let options = console.options();

        assert_eq!(options.max_width, 80);
        assert_eq!(options.size.width, 80);
    }

    #[test]
    fn test_print_options() {
        let options = PrintOptions::new()
            .with_markup(true)
            .with_style(Style::new().bold());

        assert_eq!(options.markup, Some(true));
        assert!(options.style.is_some());
    }

    #[test]
    fn test_capture() {
        let mut console = Console::new();
        console.begin_capture();

        // Print would add to buffer
        // For testing, we just verify the mechanism
        let segments = console.end_capture();
        assert!(segments.is_empty()); // Nothing captured in this test
    }

    #[test]
    fn test_capture_collects_segments() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

        impl Write for SharedBuffer {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.0.lock().unwrap().write(buf)
            }
            fn flush(&mut self) -> io::Result<()> {
                self.0.lock().unwrap().flush()
            }
        }

        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let mut console = Console::builder()
            .width(40)
            .markup(false)
            .file(Box::new(buffer))
            .build();

        console.begin_capture();
        console.print_plain("Hello");
        let segments = console.end_capture();

        let captured: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(captured.contains("Hello"));
    }

    #[test]
    fn test_dimensions() {
        let dims = ConsoleDimensions::default();
        assert_eq!(dims.width, 80);
        assert_eq!(dims.height, 24);
    }

    #[test]
    fn test_custom_output_stream() {
        use std::sync::{Arc, Mutex};

        // Thread-safe buffer that implements Write + Send
        #[derive(Clone)]
        struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

        impl Write for SharedBuffer {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.0.lock().unwrap().write(buf)
            }
            fn flush(&mut self) -> io::Result<()> {
                self.0.lock().unwrap().flush()
            }
        }

        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .width(80)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build();

        console.print_plain("Hello, World!");

        let output = buffer.0.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(
            text.contains("Hello, World!"),
            "Expected 'Hello, World!' in output, got: {text}"
        );
    }

    #[test]
    fn test_print_plain_disables_markup() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

        impl Write for SharedBuffer {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.0.lock().unwrap().write(buf)
            }
            fn flush(&mut self) -> io::Result<()> {
                self.0.lock().unwrap().flush()
            }
        }

        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .markup(true)
            .file(Box::new(buffer.clone()))
            .build();

        console.print_plain("[bold]Hello[/]");

        let output = buffer.0.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(
            text.contains("[bold]Hello[/]"),
            "Expected literal markup in output, got: {text}"
        );
        assert!(
            !text.contains("\x1b["),
            "Did not expect ANSI sequences in output, got: {text}"
        );
    }

    #[test]
    fn test_print_options_justify_uses_console_width() {
        let console = Console::builder().width(10).markup(false).build();
        let mut output = Vec::new();
        let mut options = PrintOptions::new().with_justify(JustifyMethod::Center);
        options.no_newline = true;

        console
            .print_to(&mut output, "Hi", &options)
            .expect("failed to render");

        let text = String::from_utf8(output).expect("invalid utf8");
        assert_eq!(text, "    Hi    ");
    }

    #[test]
    fn test_print_options_width_wraps() {
        let console = Console::builder().width(80).markup(false).build();
        let mut output = Vec::new();
        let mut options = PrintOptions::new();
        options.width = Some(4);

        console
            .print_to(&mut output, "Hello", &options)
            .expect("failed to render");

        let text = String::from_utf8(output).expect("invalid utf8");
        assert_eq!(text, "Hell\no\n");
    }

    #[test]
    fn test_print_options_no_wrap_ellipsis() {
        let console = Console::builder().width(80).markup(false).build();
        let mut output = Vec::new();
        let mut options = PrintOptions::new()
            .with_no_wrap(true)
            .with_overflow(OverflowMethod::Ellipsis);
        options.width = Some(4);
        options.no_newline = true;

        console
            .print_to(&mut output, "Hello", &options)
            .expect("failed to render");

        let text = String::from_utf8(output).expect("invalid utf8");
        assert_eq!(text, "H...");
    }

    #[test]
    fn test_custom_output_stream_line() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

        impl Write for SharedBuffer {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.0.lock().unwrap().write(buf)
            }
            fn flush(&mut self) -> io::Result<()> {
                self.0.lock().unwrap().flush()
            }
        }

        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .width(80)
            .file(Box::new(buffer.clone()))
            .build();

        console.line();

        let output = buffer.0.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert_eq!(text, "\n", "Expected single newline, got: {text:?}");
    }
}
