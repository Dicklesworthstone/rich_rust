//! Interactive helpers inspired by Python Rich.
//!
//! This module contains Rust-idiomatic equivalents of a few Rich conveniences
//! that combine rendering with terminal interactivity.
//!
//! Note: rich_rust's core remains output-focused; these helpers are best-effort
//! and fall back gracefully when the console is not interactive.

use std::io;
use std::io::{BufRead as _, Write as _};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::console::Console;
use crate::console::PrintOptions;
use crate::live::{Live, LiveOptions};
use crate::markup;
use crate::style::Style;
use crate::text::Text;

/// A spinner + message context helper, inspired by Python Rich's `Console.status(...)`.
///
/// When the console is interactive (`Console::is_interactive()`), this starts a `Live`
/// display that refreshes a single-line spinner. When the console is not interactive,
/// it prints the message once and does not animate.
///
/// Dropping this value stops the live display (best-effort).
pub struct Status {
    _message: Arc<Mutex<String>>,
    live: Option<Live>,
}

impl Status {
    /// Start a status spinner with a message.
    pub fn new(console: Arc<Console>, message: impl Into<String>) -> io::Result<Self> {
        let message = Arc::new(Mutex::new(message.into()));

        if !console.is_interactive() {
            if let Ok(message) = message.lock() {
                console.print_plain(&message);
            }
            return Ok(Self {
                _message: message,
                live: None,
            });
        }

        let start = Instant::now();
        let frames: [&str; 4] = ["|", "/", "-", "\\"];
        let frame_interval = Duration::from_millis(100);
        let message_for_render = Arc::clone(&message);

        let live_options = LiveOptions {
            refresh_per_second: 10.0,
            transient: true,
            ..LiveOptions::default()
        };

        let live = Live::with_options(Arc::clone(&console), live_options).get_renderable(move || {
            let elapsed = start.elapsed();
            let tick = elapsed.as_millis() / frame_interval.as_millis().max(1);
            let idx = (tick as usize) % frames.len();
            let frame = frames[idx];
            let msg = message_for_render
                .lock()
                .map(|m| m.clone())
                .unwrap_or_default();
            Box::new(Text::new(format!("{frame} {msg}")))
        });

        live.start(true)?;

        Ok(Self {
            _message: message,
            live: Some(live),
        })
    }

    /// Update the displayed message (best-effort).
    pub fn update(&self, message: impl Into<String>) {
        if let Ok(mut slot) = self._message.lock() {
            *slot = message.into();
        }

        if let Some(live) = &self.live {
            let _ = live.refresh();
        }
    }
}

impl Drop for Status {
    fn drop(&mut self) {
        if let Some(live) = &self.live {
            let _ = live.stop();
        }
    }
}

/// Errors returned by prompt operations.
#[derive(Debug)]
pub enum PromptError {
    /// Prompt requires an interactive console but `Console::is_interactive()` is false.
    NotInteractive,
    /// Prompt is non-interactive and no default value was provided.
    MissingDefault,
    /// Input stream reached EOF without yielding a value.
    Eof,
    /// Input did not pass validation.
    Validation(String),
    /// I/O error while reading input.
    Io(io::Error),
}

impl std::fmt::Display for PromptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInteractive => write!(f, "prompt requires an interactive console"),
            Self::MissingDefault => write!(f, "prompt is non-interactive and has no default"),
            Self::Eof => write!(f, "prompt input reached EOF"),
            Self::Validation(message) => write!(f, "{message}"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for PromptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for PromptError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// Prompt configuration.
#[derive(Debug, Clone)]
pub struct Prompt {
    prompt: String,
    default: Option<String>,
    allow_empty: bool,
    show_default: bool,
    markup: bool,
    validator: Option<Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>>,
}

impl Prompt {
    /// Create a new prompt.
    #[must_use]
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            default: None,
            allow_empty: false,
            show_default: true,
            markup: true,
            validator: None,
        }
    }

    /// Provide a default value (used when the user enters empty input, or when not interactive).
    #[must_use]
    pub fn default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Allow empty input when no default is set.
    #[must_use]
    pub const fn allow_empty(mut self, allow_empty: bool) -> Self {
        self.allow_empty = allow_empty;
        self
    }

    /// Show the default value in the prompt (when present).
    #[must_use]
    pub const fn show_default(mut self, show_default: bool) -> Self {
        self.show_default = show_default;
        self
    }

    /// Enable/disable markup parsing for the prompt label.
    #[must_use]
    pub const fn markup(mut self, markup: bool) -> Self {
        self.markup = markup;
        self
    }

    /// Add validation for user input. Returning `Err(message)` prints the message and re-prompts.
    #[must_use]
    pub fn validate<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.validator = Some(Arc::new(validator));
        self
    }

    /// Ask for input using stdin.
    pub fn ask(&self, console: Arc<Console>) -> Result<String, PromptError> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        self.ask_from(console, &mut reader)
    }

    /// Ask for input from a provided reader (useful for tests).
    pub fn ask_from<R: io::BufRead>(
        &self,
        console: Arc<Console>,
        reader: &mut R,
    ) -> Result<String, PromptError> {
        if !console.is_interactive() {
            return self
                .default
                .clone()
                .ok_or(PromptError::NotInteractive)
                .or_else(|_| Err(PromptError::MissingDefault));
        }

        let mut line = String::new();
        loop {
            self.print_prompt(&console);

            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                return Err(PromptError::Eof);
            }
            let input = trim_newline(&line);
            let mut value = if input.is_empty() {
                self.default.clone().unwrap_or_default()
            } else {
                input.to_string()
            };

            if value.is_empty() && !self.allow_empty && self.default.is_none() {
                self.print_error(&console, "Input required.");
                continue;
            }

            if let Some(validator) = &self.validator {
                if let Err(message) = validator(&value) {
                    self.print_error(&console, &message);
                    continue;
                }
            }

            value = value.trim_end().to_string();
            return Ok(value);
        }
    }

    fn print_prompt(&self, console: &Console) {
        let mut prompt = self.prompt.clone();
        if self.show_default && let Some(default) = &self.default {
            let default = if self.markup {
                markup::escape(default)
            } else {
                default.clone()
            };
            prompt.push_str(" [");
            prompt.push_str(&default);
            prompt.push(']');
        }
        prompt.push_str(": ");

        console.print_with_options(
            &prompt,
            &PrintOptions::new()
                .with_markup(self.markup)
                .with_no_newline(true),
        );
    }

    fn print_error(&self, console: &Console, message: &str) {
        let style = Style::parse("bold red").unwrap_or_default();
        console.print_with_options(
            message,
            &PrintOptions::new()
                .with_markup(false)
                .with_style(style)
                .with_end("\n"),
        );
    }
}

/// Best-effort pager support.
///
/// When interactive, this attempts to pipe content through `$PAGER` (or a platform default).
/// When not interactive or if spawning the pager fails, it falls back to printing directly
/// to the console.
#[derive(Debug, Clone)]
pub struct Pager {
    command: Option<String>,
    allow_color: bool,
}

impl Pager {
    /// Create a new pager with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            command: None,
            allow_color: true,
        }
    }

    /// Override the pager command.
    #[must_use]
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Allow ANSI color sequences (where supported by the pager).
    #[must_use]
    pub const fn allow_color(mut self, allow_color: bool) -> Self {
        self.allow_color = allow_color;
        self
    }

    /// Display content through the pager (best-effort).
    pub fn show(&self, console: &Console, content: &str) -> io::Result<()> {
        if !console.is_interactive() {
            return print_exact(console, content);
        }

        let (command, args) = self.resolve_command();
        match spawn_pager(&command, &args, content) {
            Ok(()) => Ok(()),
            Err(_err) => print_exact(console, content),
        }
    }

    fn resolve_command(&self) -> (String, Vec<String>) {
        let command = self
            .command
            .clone()
            .or_else(|| std::env::var("PAGER").ok())
            .unwrap_or_else(|| {
                #[cfg(windows)]
                {
                    "more".to_string()
                }
                #[cfg(not(windows))]
                {
                    "less".to_string()
                }
            });

        let mut parts = command.split_whitespace();
        let bin = parts.next().unwrap_or("less").to_string();

        let mut args: Vec<String> = parts.map(str::to_string).collect();

        if self.allow_color && bin == "less" && args.iter().all(|arg| arg != "-R") {
            args.push("-R".to_string());
        }

        (bin, args)
    }
}

fn spawn_pager(command: &str, args: &[String], content: &str) -> io::Result<()> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_bytes())?;
        stdin.flush()?;
    }

    let _status = child.wait()?;
    Ok(())
}

fn print_exact(console: &Console, content: &str) -> io::Result<()> {
    console.print_to(
        &mut io::stdout(),
        content,
        &PrintOptions::new()
            .with_markup(false)
            .with_no_newline(true),
    )
}

fn trim_newline(line: &str) -> &str {
    line.trim_end_matches(['\n', '\r'])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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

    #[test]
    fn test_status_non_interactive_prints_message_once() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(false)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let _status = Status::new(console, "Working...").expect("status");

        let out = buffer.0.lock().unwrap();
        let text = String::from_utf8_lossy(&out);
        assert!(text.contains("Working...\n"));
    }

    #[test]
    fn test_prompt_non_interactive_uses_default() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(false)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let prompt = Prompt::new("Name").default("Alice");
        let answer = prompt.ask(console).expect("prompt");
        assert_eq!(answer, "Alice");

        // Non-interactive prompt should not print.
        assert!(buffer.0.lock().unwrap().is_empty());
    }

    #[test]
    fn test_prompt_from_reader_validates_and_reprompts() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let prompt = Prompt::new("Age").validate(|value| {
            if value.chars().all(|c| c.is_ascii_digit()) {
                Ok(())
            } else {
                Err("digits only".to_string())
            }
        });

        let input = b"nope\n42\n";
        let mut reader = io::Cursor::new(&input[..]);
        let answer = prompt.ask_from(console, &mut reader).expect("prompt");
        assert_eq!(answer, "42");

        let out = buffer.0.lock().unwrap();
        let text = String::from_utf8_lossy(&out);
        assert!(text.contains("digits only\n"));
    }

    #[test]
    fn test_pager_non_interactive_falls_back_to_print() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(false)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build();

        Pager::new()
            .show(&console, "hello\nworld\n")
            .expect("pager");

        let out = buffer.0.lock().unwrap();
        let text = String::from_utf8_lossy(&out);
        assert!(text.contains("hello\nworld\n"));
    }
}
