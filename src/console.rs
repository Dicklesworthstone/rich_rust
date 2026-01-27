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

use std::fmt::Write as FmtWrite;
use std::io::{self, Write};
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};

use crate::color::ColorSystem;
use crate::emoji;
use crate::live::LiveInner;
use crate::markup;
use crate::renderables::Renderable;
use crate::segment::{ControlCode, ControlType, Segment};
use crate::style::{Attributes, Style};
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

    /// Create options with updated width and height.
    #[must_use]
    pub fn update_dimensions(&self, width: usize, height: usize) -> Self {
        Self {
            size: ConsoleDimensions { width, height },
            max_width: width,
            max_height: height,
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

/// Hook for intercepting rendered segments before output.
pub trait RenderHook: Send + Sync {
    fn process(&self, console: &Console, segments: &[Segment<'static>]) -> Vec<Segment<'static>>;
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
    record: AtomicBool,
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
    file: Mutex<Box<dyn Write + Send>>,
    /// Recording buffer.
    buffer: Mutex<Vec<Segment<'static>>>,
    /// Cached terminal detection.
    is_terminal: bool,
    /// Detected/configured color system.
    detected_color_system: Option<ColorSystem>,
    /// Render hooks (Live uses this).
    render_hooks: Mutex<Vec<Arc<dyn RenderHook>>>,
    /// Active Live stack for nested Live handling.
    live_stack: Mutex<Vec<Weak<LiveInner>>>,
}

impl std::fmt::Debug for Console {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Console")
            .field("color_system", &self.color_system)
            .field("force_terminal", &self.force_terminal)
            .field("tab_size", &self.tab_size)
            .field("record", &self.record.load(Ordering::Relaxed))
            .field("markup", &self.markup)
            .field("emoji", &self.emoji)
            .field("highlight", &self.highlight)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("safe_box", &self.safe_box)
            .field("file", &"<dyn Write>")
            .field("buffer_len", &self.buffer.lock().map_or(0, |b| b.len()))
            .field("is_terminal", &self.is_terminal)
            .field("detected_color_system", &self.detected_color_system)
            .finish_non_exhaustive()
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
            record: AtomicBool::new(false),
            markup: true,
            emoji: true,
            highlight: true,
            width: None,
            height: None,
            safe_box: false,
            file: Mutex::new(Box::new(io::stdout())),
            buffer: Mutex::new(Vec::new()),
            is_terminal,
            detected_color_system,
            render_hooks: Mutex::new(Vec::new()),
            live_stack: Mutex::new(Vec::new()),
        }
    }

    /// Create a console builder for custom configuration.
    #[must_use]
    pub fn builder() -> ConsoleBuilder {
        ConsoleBuilder::default()
    }

    /// Convert this Console into a shared reference-counted handle.
    #[must_use]
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
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

    /// Check if Rich-style emoji code replacement is enabled.
    #[must_use]
    pub const fn emoji(&self) -> bool {
        self.emoji
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

    /// Check if the terminal is "dumb".
    #[must_use]
    pub fn is_dumb_terminal(&self) -> bool {
        terminal::is_dumb_terminal()
    }

    /// Check if the console is interactive (TTY and not dumb).
    #[must_use]
    pub fn is_interactive(&self) -> bool {
        self.is_terminal() && !self.is_dumb_terminal()
    }

    pub(crate) fn push_render_hook(&self, hook: Arc<dyn RenderHook>) {
        if let Ok(mut hooks) = self.render_hooks.lock() {
            hooks.push(hook);
        }
    }

    pub(crate) fn pop_render_hook(&self) -> Option<Arc<dyn RenderHook>> {
        self.render_hooks
            .lock()
            .ok()
            .and_then(|mut hooks| hooks.pop())
    }

    pub(crate) fn set_live(&self, live: &Arc<LiveInner>) -> bool {
        if let Ok(mut stack) = self.live_stack.lock() {
            stack.push(Arc::downgrade(live));
            return stack.len() == 1;
        }
        true
    }

    pub(crate) fn clear_live(&self) {
        if let Ok(mut stack) = self.live_stack.lock()
            && !stack.is_empty()
        {
            stack.pop();
        }
    }

    pub(crate) fn live_stack_snapshot(&self) -> Vec<Arc<LiveInner>> {
        let mut result = Vec::new();
        if let Ok(mut stack) = self.live_stack.lock() {
            stack.retain(|entry| entry.strong_count() > 0);
            for entry in stack.iter() {
                if let Some(live) = entry.upgrade() {
                    result.push(live);
                }
            }
        }
        result
    }

    pub(crate) fn write_control_codes(&self, control_codes: Vec<ControlCode>) -> io::Result<()> {
        if control_codes.is_empty() {
            return Ok(());
        }
        let segment = Segment::control(control_codes);
        let mut file = self
            .file
            .lock()
            .map_err(|_| io::Error::other("console output lock poisoned"))?;
        self.write_segments_raw(&mut *file, &[segment])
    }

    /// Show or hide the cursor.
    pub fn show_cursor(&self, show: bool) -> io::Result<()> {
        let control = if show {
            ControlCode::new(ControlType::ShowCursor)
        } else {
            ControlCode::new(ControlType::HideCursor)
        };
        self.write_control_codes(vec![control])
    }

    /// Enable or disable the alternate screen buffer.
    pub fn set_alt_screen(&self, enable: bool) -> io::Result<()> {
        let control = if enable {
            ControlCode::new(ControlType::EnableAltScreen)
        } else {
            ControlCode::new(ControlType::DisableAltScreen)
        };
        self.write_control_codes(vec![control])
    }

    /// Enable recording mode.
    pub fn begin_capture(&mut self) {
        self.record.store(true, Ordering::Relaxed);
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.clear();
        }
    }

    /// End recording and return captured segments.
    pub fn end_capture(&mut self) -> Vec<Segment<'static>> {
        self.record.store(false, Ordering::Relaxed);
        self.buffer
            .lock()
            .map(|mut buffer| std::mem::take(&mut *buffer))
            .unwrap_or_default()
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
        if let Ok(mut file) = self.file.lock() {
            let _ = self.print_text_to(&mut *file, text);
        }
    }

    /// Print a prepared Text object to a specific writer.
    pub fn print_text_to<W: Write>(&self, writer: &mut W, text: &Text) -> io::Result<()> {
        let segments: Vec<Segment<'static>> = text
            .render(&text.end)
            .into_iter()
            .map(Segment::into_owned)
            .collect();
        let segments = self.apply_render_hooks(segments);
        self.write_segments_raw(writer, &segments)
    }

    /// Print prepared segments.
    pub fn print_segments(&self, segments: &[Segment<'_>]) {
        if let Ok(mut file) = self.file.lock() {
            let _ = self.print_segments_to(&mut *file, segments);
        }
    }

    /// Print prepared segments to a specific writer.
    pub fn print_segments_to<W: Write>(
        &self,
        writer: &mut W,
        segments: &[Segment<'_>],
    ) -> io::Result<()> {
        let owned: Vec<Segment<'static>> =
            segments.iter().cloned().map(Segment::into_owned).collect();
        let processed = self.apply_render_hooks(owned);
        self.write_segments_raw(writer, &processed)
    }

    /// Print any object implementing the Renderable trait.
    pub fn print_renderable(&self, renderable: &impl Renderable) {
        let options = self.options();
        let segments = renderable.render(self, &options);
        self.print_segments(&segments);
    }

    /// Print with custom options.
    pub fn print_with_options(&self, content: &str, options: &PrintOptions) {
        if let Ok(mut file) = self.file.lock() {
            self.print_to(&mut *file, content, options)
                .expect("failed to write to output stream");
        }
    }

    /// Export rendered text (no ANSI) using default print options.
    #[must_use]
    pub fn export_text(&self, content: &str) -> String {
        self.export_text_with_options(content, &PrintOptions::new().with_markup(self.markup))
    }

    /// Export rendered text (no ANSI) using custom print options.
    #[must_use]
    pub fn export_text_with_options(&self, content: &str, options: &PrintOptions) -> String {
        let segments = self.render_str_segments(content, options);
        Self::segments_to_plain(&segments)
    }

    /// Export a renderable to plain text (no ANSI).
    #[must_use]
    pub fn export_renderable_text(&self, renderable: &impl Renderable) -> String {
        let options = self.options();
        let segments = renderable.render(self, &options);
        Self::segments_to_plain(&segments)
    }

    /// Export recorded output to HTML.
    #[must_use]
    pub fn export_html(&self, clear: bool) -> String {
        let segments = self.recorded_segments(clear);
        export_segments_to_html(&segments)
    }

    /// Export recorded output to SVG.
    #[must_use]
    pub fn export_svg(&self, clear: bool) -> String {
        let segments = self.recorded_segments(clear);
        export_segments_to_svg(&segments)
    }

    /// Print to a specific writer.
    pub fn print_to<W: Write>(
        &self,
        writer: &mut W,
        content: &str,
        options: &PrintOptions,
    ) -> io::Result<()> {
        let segments = self.render_str_segments(content, options);
        let segments = self.apply_render_hooks(segments);
        self.write_segments_raw(writer, &segments)
    }

    fn render_str_segments(&self, content: &str, options: &PrintOptions) -> Vec<Segment<'static>> {
        let content = if self.emoji {
            emoji::replace(content, None)
        } else {
            std::borrow::Cow::Borrowed(content)
        };

        // Parse markup if enabled
        let parse_markup = options.markup.unwrap_or(self.markup);
        let mut text = if parse_markup {
            markup::render_or_plain(content.as_ref())
        } else {
            Text::new(content.as_ref())
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
        // soft_wrap enables wrapping by overriding text's no_wrap setting
        if options.soft_wrap {
            text.no_wrap = false;
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
        let mut segments: Vec<Segment<'static>> = if let Some(width) = width {
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
                rendered.extend(line.render(line_end).into_iter().map(Segment::into_owned));
            }

            rendered
        } else {
            text.render(end)
                .into_iter()
                .map(Segment::into_owned)
                .collect()
        };

        // Apply any overall style
        if let Some(ref style) = options.style {
            for segment in &mut segments {
                if !segment.is_control() {
                    segment.style = Some(match segment.style {
                        Some(ref s) => style.combine(s),
                        None => style.clone(),
                    });
                }
            }
        }

        segments
    }

    fn segments_to_plain(segments: &[Segment<'_>]) -> String {
        let capacity: usize = segments
            .iter()
            .filter(|segment| !segment.is_control())
            .map(|segment| segment.text.len())
            .sum();
        let mut output = String::with_capacity(capacity);
        for segment in segments {
            if !segment.is_control() {
                output.push_str(segment.text.as_ref());
            }
        }
        output
    }

    fn recorded_segments(&self, clear: bool) -> Vec<Segment<'static>> {
        let Ok(mut buffer) = self.buffer.lock() else {
            return Vec::new();
        };
        let segments = buffer.clone();
        if clear {
            buffer.clear();
        }
        segments
    }

    fn apply_render_hooks(&self, segments: Vec<Segment<'static>>) -> Vec<Segment<'static>> {
        let hooks = self
            .render_hooks
            .lock()
            .map(|hooks| hooks.clone())
            .unwrap_or_default();
        if hooks.is_empty() {
            return segments;
        }
        let mut current = segments;
        for hook in hooks {
            current = hook.process(self, &current);
        }
        current
    }

    /// Write segments to a writer without invoking render hooks.
    fn write_segments_raw<W: Write>(
        &self,
        writer: &mut W,
        segments: &[Segment<'_>],
    ) -> io::Result<()> {
        if self.record.load(Ordering::Relaxed)
            && let Ok(mut buffer) = self.buffer.lock()
        {
            buffer.extend(segments.iter().cloned().map(Segment::into_owned));
        }

        let color_system = self.color_system();

        for segment in segments {
            if segment.is_control() {
                self.write_control_segment(writer, segment)?;
                continue;
            }

            // Get ANSI codes for style
            let ansi_codes;
            let (prefix, suffix) = if let Some(ref style) = segment.style {
                if let Some(cs) = color_system {
                    ansi_codes = style.render_ansi(cs);
                    (&ansi_codes.0, &ansi_codes.1)
                } else {
                    static EMPTY: (String, String) = (String::new(), String::new());
                    (&EMPTY.0, &EMPTY.1)
                }
            } else {
                static EMPTY: (String, String) = (String::new(), String::new());
                (&EMPTY.0, &EMPTY.1)
            };

            // Write styled text
            write!(writer, "{prefix}{}{suffix}", segment.text)?;
        }

        writer.flush()
    }

    fn write_control_segment<W: Write>(
        &self,
        writer: &mut W,
        segment: &Segment<'_>,
    ) -> io::Result<()> {
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
        if let Ok(mut file) = self.file.lock() {
            let _ = writeln!(file);
        }
    }

    /// Print a rule (horizontal line).
    pub fn rule(&self, title: Option<&str>) {
        let width = self.width();
        let line_char = if self.safe_box { '-' } else { '\u{2500}' };

        let Ok(mut file) = self.file.lock() else {
            return;
        };
        if let Some(title) = title {
            // Ensure title fits within width, accounting for 2 spaces padding
            let max_title_width = width.saturating_sub(2);
            let title_len = crate::cells::cell_len(title);

            let display_title = if title_len > max_title_width {
                let mut t = Text::new(title);
                t.truncate(max_title_width, OverflowMethod::Ellipsis, false);
                t.plain().to_string()
            } else {
                title.to_string()
            };

            let display_len = crate::cells::cell_len(&display_title);
            let available = width.saturating_sub(display_len + 2);
            let left_pad = available / 2;
            let right_pad = available - left_pad;
            let left = line_char.to_string().repeat(left_pad);
            let right = line_char.to_string().repeat(right_pad);
            let _ = writeln!(file, "{left} {display_title} {right}");
        } else {
            let _ = writeln!(file, "{}", line_char.to_string().repeat(width));
        }
    }

    /// Clear the screen.
    pub fn clear(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = terminal::control::clear_screen(&mut *file);
        }
    }

    /// Clear the current line.
    pub fn clear_line(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = terminal::control::clear_line(&mut *file);
        }
    }

    /// Set the terminal title.
    pub fn set_title(&self, title: &str) {
        if let Ok(mut file) = self.file.lock() {
            let _ = terminal::control::set_title(&mut *file, title);
        }
    }

    /// Ring the terminal bell.
    pub fn bell(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = terminal::control::bell(&mut *file);
        }
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

        if let Ok(mut file) = self.file.lock() {
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

fn control_title(segment: &Segment<'_>, control: &crate::segment::ControlCode) -> Option<String> {
    if !segment.text.is_empty() {
        return Some(segment.text.to_string());
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

fn export_segments_to_html(segments: &[Segment<'_>]) -> String {
    let body = export_segments_to_html_body(segments);
    format!("<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body>{body}</body></html>")
}

fn export_segments_to_svg(segments: &[Segment<'_>]) -> String {
    let (width_cells, height_cells) = segments_shape(segments);
    let cell_width = 8usize;
    let cell_height = 16usize;
    let width_px = width_cells.saturating_mul(cell_width);
    let height_px = height_cells.saturating_mul(cell_height);
    let body = export_segments_to_html_body(segments);

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width_px}\" height=\"{height_px}\">\
<foreignObject width=\"100%\" height=\"100%\">{body}</foreignObject></svg>"
    )
}

fn export_segments_to_html_body(segments: &[Segment<'_>]) -> String {
    let mut html = String::new();
    html.push_str("<pre style=\"margin:0; font-family: monospace;\">");
    for segment in segments {
        if segment.is_control() {
            continue;
        }
        let text = escape_html(segment.text.as_ref());
        if let Some(style) = &segment.style {
            let css = style_to_css(style);
            if let Some(link) = &style.link {
                let href = escape_attr(link);
                if css.is_empty() {
                    let _ = FmtWrite::write_fmt(
                        &mut html,
                        format_args!("<a href=\"{href}\">{text}</a>"),
                    );
                } else {
                    let _ = FmtWrite::write_fmt(
                        &mut html,
                        format_args!("<a href=\"{href}\" style=\"{css}\">{text}</a>"),
                    );
                }
            } else if css.is_empty() {
                html.push_str(&text);
            } else {
                let _ = FmtWrite::write_fmt(
                    &mut html,
                    format_args!("<span style=\"{css}\">{text}</span>"),
                );
            }
        } else {
            html.push_str(&text);
        }
    }
    html.push_str("</pre>");
    html
}

fn segments_shape(segments: &[Segment<'_>]) -> (usize, usize) {
    let lines = crate::segment::split_lines(segments.iter().cloned().map(Segment::into_owned));
    let mut max_width = 0usize;
    for line in &lines {
        let width: usize = line.iter().map(Segment::cell_length).sum();
        if width > max_width {
            max_width = width;
        }
    }
    (max_width, lines.len())
}

fn style_to_css(style: &Style) -> String {
    if style.is_null() {
        return String::new();
    }

    let mut fg = style.color.as_ref().map(|c| c.get_truecolor().hex());
    let mut bg = style.bgcolor.as_ref().map(|c| c.get_truecolor().hex());

    if style.attributes.contains(Attributes::REVERSE) {
        std::mem::swap(&mut fg, &mut bg);
    }

    let mut css = String::new();
    if let Some(color) = fg {
        let _ = FmtWrite::write_fmt(&mut css, format_args!("color:{color};"));
    }
    if let Some(color) = bg {
        let _ = FmtWrite::write_fmt(&mut css, format_args!("background-color:{color};"));
    }
    if style.attributes.contains(Attributes::BOLD) {
        css.push_str("font-weight:bold;");
    }
    if style.attributes.contains(Attributes::ITALIC) {
        css.push_str("font-style:italic;");
    }

    let mut decorations = Vec::new();
    if style.attributes.contains(Attributes::UNDERLINE)
        || style.attributes.contains(Attributes::UNDERLINE2)
    {
        decorations.push("underline");
    }
    if style.attributes.contains(Attributes::STRIKE) {
        decorations.push("line-through");
    }
    if style.attributes.contains(Attributes::OVERLINE) {
        decorations.push("overline");
    }
    if !decorations.is_empty() {
        let _ = FmtWrite::write_fmt(
            &mut css,
            format_args!("text-decoration:{};", decorations.join(" ")),
        );
    }

    if style.attributes.contains(Attributes::DIM) {
        css.push_str("opacity:0.7;");
    }

    css
}

fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn escape_attr(text: &str) -> String {
    escape_html(text)
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
            if console.color_system.is_none() {
                console.detected_color_system = if ft {
                    terminal::detect_color_system_forced(true)
                } else {
                    None
                };
            }
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
            console.file = Mutex::new(f);
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

        let captured: String = segments.iter().map(|s| s.text.as_ref()).collect();
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
    fn test_export_text_defaults() {
        let console = Console::builder().markup(true).build();
        let output = console.export_text("[bold]Hello[/]");
        assert_eq!(output, "Hello\n");
    }

    #[test]
    fn test_export_text_respects_markup_setting() {
        let console = Console::builder().markup(false).build();
        let output = console.export_text("[bold]Hello[/]");
        assert_eq!(output, "[bold]Hello[/]\n");
    }

    #[test]
    fn test_export_text_replaces_emoji_codes_by_default() {
        let console = Console::builder().markup(false).build();
        let output = console.export_text("hi :smile:");
        assert_eq!(output, "hi 😄\n");
    }

    #[test]
    fn test_export_text_does_not_replace_emoji_codes_when_disabled() {
        let console = Console::builder().markup(false).emoji(false).build();
        let output = console.export_text("hi :smile:");
        assert_eq!(output, "hi :smile:\n");
    }

    #[test]
    fn test_export_text_with_options_no_newline() {
        let console = Console::new();
        let mut options = PrintOptions::new().with_markup(false);
        options.no_newline = true;
        let output = console.export_text_with_options("Hello", &options);
        assert_eq!(output, "Hello");
    }

    #[test]
    fn test_export_renderable_text() {
        use crate::renderables::Rule;

        let console = Console::builder().width(20).build();
        let rule = Rule::with_title("Title");
        let output = console.export_renderable_text(&rule);
        assert!(output.contains("Title"));
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_export_html_svg_capture() {
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
            .markup(false)
            .file(Box::new(buffer))
            .build();

        console.begin_capture();
        console.print_plain("Hello");

        let html = console.export_html(false);
        assert!(html.contains("<pre"));
        assert!(html.contains("Hello"));

        let svg = console.export_svg(true);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Hello"));

        let cleared = console.export_html(false);
        assert!(!cleared.contains("Hello"));
    }

    #[test]
    fn test_escape_html_entities() {
        let escaped = escape_html("<>&\"'");
        assert_eq!(escaped, "&lt;&gt;&amp;&quot;&#x27;");
    }

    #[test]
    fn test_style_to_css_basic_attributes() {
        use crate::color::Color;

        let style = Style::new()
            .color(Color::from_rgb(255, 0, 0))
            .bgcolor(Color::from_rgb(0, 0, 255))
            .bold()
            .italic()
            .underline()
            .strike();
        let css = style_to_css(&style);

        assert!(css.contains("color:#ff0000;"));
        assert!(css.contains("background-color:#0000ff;"));
        assert!(css.contains("font-weight:bold;"));
        assert!(css.contains("font-style:italic;"));
        assert!(css.contains("text-decoration:"));
        assert!(css.contains("underline"));
        assert!(css.contains("line-through"));
    }

    #[test]
    fn test_style_to_css_reverse_swaps_colors() {
        use crate::color::Color;

        let style = Style::new()
            .color(Color::from_rgb(10, 20, 30))
            .bgcolor(Color::from_rgb(200, 210, 220))
            .reverse();
        let css = style_to_css(&style);

        assert!(css.contains("color:#c8d2dc;"));
        assert!(css.contains("background-color:#0a141e;"));
    }

    #[test]
    fn test_export_html_body_links_and_spans() {
        let link_style = Style::new().link("https://example.com").bold();
        let segments = vec![
            Segment::new("Link", Some(link_style)),
            Segment::new(" ", None),
            Segment::new("Plain", None),
        ];

        let html = export_segments_to_html_body(&segments);
        assert!(html.starts_with("<pre"));
        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("font-weight:bold;"));
        assert!(html.contains("Plain"));
    }

    #[test]
    fn test_export_html_escapes_text() {
        let segments = vec![Segment::plain("<tag> & \"quote\"")];
        let html = export_segments_to_html_body(&segments);
        assert!(html.contains("&lt;tag&gt;"));
        assert!(html.contains("&amp;"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn test_export_html_skips_control_segments() {
        use crate::segment::{ControlCode, ControlType};

        let segments = vec![
            Segment::control(vec![ControlCode::new(ControlType::Bell)]),
            Segment::new("Hi", None),
        ];
        let html = export_segments_to_html_body(&segments);
        assert!(html.contains("Hi"));
        assert!(!html.contains("Bell"));
    }

    #[test]
    fn test_export_svg_dimensions() {
        let segments = vec![Segment::plain("AB"), Segment::line(), Segment::plain("C")];
        let svg = export_segments_to_svg(&segments);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("width=\"16\""));
        assert!(svg.contains("height=\"32\""));
        assert!(svg.contains("foreignObject"));
    }

    #[test]
    fn test_export_svg_includes_text() {
        let segments = vec![Segment::plain("Hello")];
        let svg = export_segments_to_svg(&segments);
        assert!(svg.contains("Hello"));
    }

    #[test]
    fn test_export_html_document_structure() {
        let segments = vec![Segment::plain("Hello")];
        let html = export_segments_to_html(&segments);
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<meta charset=\"utf-8\">"));
        assert!(html.contains("<body>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn test_export_html_includes_renderable_content() {
        use crate::renderables::{Column, Panel, Table, Tree, TreeNode};

        let mut console = Console::builder().width(30).build();
        console.begin_capture();

        let mut table = Table::new().with_column(Column::new("Col"));
        table.add_row_cells(["Cell"]);
        console.print_renderable(&table);

        let panel = Panel::from_text("Panel").width(10);
        console.print_renderable(&panel);

        let root = TreeNode::new("Root").child(TreeNode::new("Leaf"));
        let tree = Tree::new(root);
        console.print_renderable(&tree);

        let html = console.export_html(true);
        assert!(html.contains("Col"));
        assert!(html.contains("Cell"));
        assert!(html.contains("Panel"));
        assert!(html.contains("Root"));
        assert!(html.contains("Leaf"));
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

    // ========== ConsoleBuilder Tests ==========

    #[test]
    fn test_console_builder_color_system() {
        let console = Console::builder()
            .color_system(ColorSystem::TrueColor)
            .build();
        assert_eq!(console.color_system(), Some(ColorSystem::TrueColor));
    }

    #[test]
    fn test_console_builder_no_color() {
        let console = Console::builder().no_color().build();
        assert_eq!(console.color_system, None);
    }

    #[test]
    fn test_console_builder_force_terminal() {
        let console = Console::builder().force_terminal(true).build();
        assert!(console.is_terminal());
    }

    #[test]
    fn test_console_builder_tab_size() {
        let console = Console::builder().tab_size(4).build();
        assert_eq!(console.tab_size(), 4);
    }

    #[test]
    fn test_console_builder_emoji() {
        let console = Console::builder().emoji(false).build();
        assert!(!console.emoji);
    }

    #[test]
    fn test_console_builder_highlight() {
        let console = Console::builder().highlight(false).build();
        assert!(!console.highlight);
    }

    #[test]
    fn test_console_builder_safe_box() {
        let console = Console::builder().safe_box(true).build();
        assert!(console.safe_box);
    }

    #[test]
    fn test_console_builder_all_options() {
        let console = Console::builder()
            .color_system(ColorSystem::EightBit)
            .force_terminal(true)
            .tab_size(2)
            .markup(false)
            .emoji(false)
            .highlight(false)
            .width(120)
            .height(40)
            .safe_box(true)
            .build();

        assert_eq!(console.color_system(), Some(ColorSystem::EightBit));
        assert!(console.is_terminal());
        assert_eq!(console.tab_size(), 2);
        assert!(!console.markup);
        assert!(!console.emoji);
        assert!(!console.highlight);
        assert_eq!(console.width(), 120);
        assert_eq!(console.height(), 40);
        assert!(console.safe_box);
    }

    // ========== Console Size Tests ==========

    #[test]
    fn test_console_size_returns_dimensions() {
        let console = Console::builder().width(100).height(50).build();
        let size = console.size();
        assert_eq!(size.width, 100);
        assert_eq!(size.height, 50);
    }

    #[test]
    fn test_console_default_dimensions() {
        let console = Console::new();
        // Default should be reasonable terminal size
        assert!(console.width() >= 40);
        assert!(console.height() >= 10);
    }

    // ========== PrintOptions Tests ==========

    #[test]
    fn test_print_options_default() {
        let options = PrintOptions::new();
        assert_eq!(options.markup, None);
        assert!(options.style.is_none());
        assert_eq!(options.sep, " ");
        assert_eq!(options.end, "\n");
        assert_eq!(options.no_wrap, None);
        assert!(!options.no_newline);
    }

    #[test]
    fn test_print_options_with_sep() {
        let options = PrintOptions::new().with_sep(", ");
        assert_eq!(options.sep, ", ");
    }

    #[test]
    fn test_print_options_with_end() {
        let options = PrintOptions::new().with_end("\r\n");
        assert_eq!(options.end, "\r\n");
    }

    #[test]
    fn test_print_options_with_overflow() {
        let options = PrintOptions::new().with_overflow(OverflowMethod::Crop);
        assert_eq!(options.overflow, Some(OverflowMethod::Crop));
    }

    #[test]
    fn test_print_options_with_crop() {
        let options = PrintOptions::new().with_crop(true);
        assert!(options.crop);
    }

    #[test]
    fn test_print_options_with_soft_wrap() {
        let options = PrintOptions::new().with_soft_wrap(true);
        assert!(options.soft_wrap);
    }

    #[test]
    fn test_print_options_chained() {
        let style = Style::new().bold().italic();
        let options = PrintOptions::new()
            .with_markup(false)
            .with_style(style.clone())
            .with_sep(" | ")
            .with_end("")
            .with_justify(JustifyMethod::Right)
            .with_overflow(OverflowMethod::Ellipsis)
            .with_no_wrap(true)
            .with_no_newline(true)
            .with_highlight(true)
            .with_width(40)
            .with_crop(true)
            .with_soft_wrap(true);

        assert_eq!(options.markup, Some(false));
        assert!(options.style.is_some());
        assert_eq!(options.sep, " | ");
        assert_eq!(options.end, "");
        assert_eq!(options.justify, Some(JustifyMethod::Right));
        assert_eq!(options.overflow, Some(OverflowMethod::Ellipsis));
        assert_eq!(options.no_wrap, Some(true));
        assert!(options.no_newline);
        assert!(options.highlight);
        assert_eq!(options.width, Some(40));
        assert!(options.crop);
        assert!(options.soft_wrap);
    }

    // ========== ConsoleOptions Tests ==========

    #[test]
    fn test_console_options_update_width() {
        let console = Console::builder().width(100).build();
        let options = console.options();
        // update_width clamps to the new width (min of current and new)
        let updated = options.update_width(80);
        assert_eq!(updated.max_width, 80);
    }

    #[test]
    fn test_console_options_update_height() {
        let console = Console::builder().height(24).build();
        let options = console.options();
        // update_height sets the height in the options
        let updated = options.update_height(50);
        assert_eq!(updated.height, Some(50));
    }

    // ========== Color System Tests ==========

    #[test]
    fn test_console_is_color_enabled_with_system() {
        let console = Console::builder()
            .color_system(ColorSystem::Standard)
            .build();
        assert!(console.is_color_enabled());
    }

    #[test]
    fn test_console_is_color_enabled_no_color() {
        let console = Console::builder().no_color().build();
        assert!(!console.is_color_enabled());
    }

    // ========== Capture Mode Tests ==========

    #[test]
    fn test_capture_empty() {
        let mut console = Console::new();
        console.begin_capture();
        let segments = console.end_capture();
        assert!(segments.is_empty());
    }

    #[test]
    fn test_capture_with_styled_text() {
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
            .width(80)
            .markup(true)
            .color_system(ColorSystem::TrueColor)
            .file(Box::new(buffer))
            .build();

        console.begin_capture();
        console.print("[bold]Test[/]");
        let segments = console.end_capture();

        // Should have captured at least one segment
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_capture_multiple_prints() {
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
            .width(80)
            .markup(false)
            .file(Box::new(buffer))
            .build();

        console.begin_capture();
        console.print_plain("First");
        console.print_plain("Second");
        let segments = console.end_capture();

        let text: String = segments.iter().map(|s| s.text.as_ref()).collect();
        assert!(text.contains("First"));
        assert!(text.contains("Second"));
    }

    // ========== Print Method Tests ==========

    #[test]
    fn test_print_text_direct() {
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
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build();

        let text = Text::new("Direct text");
        console.print_text(&text);

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        assert!(result.contains("Direct text"));
    }

    #[test]
    fn test_print_styled() {
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
            .color_system(ColorSystem::TrueColor)
            .file(Box::new(buffer.clone()))
            .build();

        console.print_styled("Styled", Style::new().bold());

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        assert!(result.contains("Styled"));
        // Should contain ANSI codes for bold
        assert!(result.contains("\x1b["));
    }

    #[test]
    fn test_print_to_writer() {
        let console = Console::builder().width(80).markup(false).build();
        let mut output = Vec::new();
        let options = PrintOptions::new();

        console
            .print_to(&mut output, "Writer test", &options)
            .expect("failed to print");

        let text = String::from_utf8(output).expect("invalid utf8");
        assert!(text.contains("Writer test"));
    }

    #[test]
    fn test_print_segments() {
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

        let segments = vec![Segment::plain("Hello "), Segment::plain("World")];
        console.print_segments(&segments);

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        assert!(result.contains("Hello "));
        assert!(result.contains("World"));
    }

    // ========== Rule Method Tests ==========

    #[test]
    fn test_rule_without_title() {
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
            .width(20)
            .file(Box::new(buffer.clone()))
            .build();

        console.rule(None);

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        // Rule should contain horizontal line characters
        assert!(result.contains('─') || result.contains('-'));
    }

    #[test]
    fn test_rule_with_title() {
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
            .width(40)
            .file(Box::new(buffer.clone()))
            .build();

        console.rule(Some("Section"));

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        assert!(result.contains("Section"));
    }

    // ========== Log Method Tests ==========

    #[test]
    fn test_log_debug() {
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

        console.log("Debug message", LogLevel::Debug);

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        assert!(result.contains("Debug message"));
    }

    #[test]
    fn test_log_info() {
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

        console.log("Info message", LogLevel::Info);

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        assert!(result.contains("Info message"));
    }

    #[test]
    fn test_log_warning() {
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

        console.log("Warning message", LogLevel::Warning);

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        assert!(result.contains("Warning message"));
    }

    #[test]
    fn test_log_error() {
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

        console.log("Error message", LogLevel::Error);

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        assert!(result.contains("Error message"));
    }

    // ========== Markup Integration Tests ==========

    #[test]
    fn test_markup_enabled() {
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
            .markup(true)
            .color_system(ColorSystem::TrueColor)
            .file(Box::new(buffer.clone()))
            .build();

        console.print("[bold]Bold text[/]");

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        // Should contain ANSI codes, not literal [bold]
        assert!(!result.contains("[bold]"));
        assert!(result.contains("\x1b["));
    }

    #[test]
    fn test_markup_disabled() {
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
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build();

        console.print("[bold]Literal markup[/]");

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        // Should contain literal markup tags
        assert!(result.contains("[bold]"));
    }

    // ========== Width Constraint Tests ==========

    #[test]
    fn test_print_with_width_constraint() {
        let console = Console::builder().width(80).markup(false).build();
        let mut output = Vec::new();
        let mut options = PrintOptions::new();
        options.width = Some(10);

        console
            .print_to(
                &mut output,
                "This is a long text that should wrap",
                &options,
            )
            .expect("failed to print");

        let text = String::from_utf8(output).expect("invalid utf8");
        // Text should be wrapped at width 10
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_justify_left() {
        let console = Console::builder().width(20).markup(false).build();
        let mut output = Vec::new();
        let mut options = PrintOptions::new().with_justify(JustifyMethod::Left);
        options.no_newline = true;

        console
            .print_to(&mut output, "Left", &options)
            .expect("failed to print");

        let text = String::from_utf8(output).expect("invalid utf8");
        assert!(text.starts_with("Left"));
    }

    #[test]
    fn test_justify_right() {
        let console = Console::builder().width(20).markup(false).build();
        let mut output = Vec::new();
        let mut options = PrintOptions::new().with_justify(JustifyMethod::Right);
        options.no_newline = true;

        console
            .print_to(&mut output, "Right", &options)
            .expect("failed to print");

        let text = String::from_utf8(output).expect("invalid utf8");
        assert!(text.ends_with("Right"));
        assert!(text.len() == 20);
    }

    // ========== ConsoleDimensions Tests ==========

    #[test]
    fn test_console_dimensions_default() {
        let dims = ConsoleDimensions::default();
        assert_eq!(dims.width, 80);
        assert_eq!(dims.height, 24);
    }

    #[test]
    fn test_console_dimensions_custom() {
        let dims = ConsoleDimensions {
            width: 120,
            height: 40,
        };
        assert_eq!(dims.width, 120);
        assert_eq!(dims.height, 40);
    }

    // ========== PrintOptions Default Trait ==========

    #[test]
    fn test_print_options_implements_default() {
        // Default::default() uses derived defaults (empty strings)
        // PrintOptions::new() sets explicit defaults (sep=" ", end="\n")
        let default_options = PrintOptions::default();
        assert_eq!(default_options.sep, "");
        assert_eq!(default_options.end, "");

        // new() provides the typical defaults
        let new_options = PrintOptions::new();
        assert_eq!(new_options.sep, " ");
        assert_eq!(new_options.end, "\n");
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_print_empty_string() {
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

        console.print_plain("");

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        // Should only have newline
        assert_eq!(result.trim(), "");
    }

    #[test]
    fn test_print_unicode() {
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

        console.print_plain("Hello 世界 🌍");

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        assert!(result.contains("世界"));
        assert!(result.contains("🌍"));
    }

    #[test]
    fn test_print_with_newlines() {
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

        console.print_plain("Line 1\nLine 2\nLine 3");

        let output = buffer.0.lock().unwrap();
        let result = String::from_utf8_lossy(&output);
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_overflow_crop() {
        let console = Console::builder().width(80).markup(false).build();
        let mut output = Vec::new();
        let mut options = PrintOptions::new()
            .with_no_wrap(true)
            .with_overflow(OverflowMethod::Crop);
        options.width = Some(5);
        options.no_newline = true;

        console
            .print_to(&mut output, "Hello World", &options)
            .expect("failed to print");

        let text = String::from_utf8(output).expect("invalid utf8");
        assert_eq!(text, "Hello");
    }
}
