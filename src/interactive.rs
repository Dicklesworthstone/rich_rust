//! Interactive helpers inspired by Python Rich.
//!
//! This module contains Rust-idiomatic equivalents of a few Rich conveniences
//! that combine rendering with terminal interactivity.
//!
//! Note: `rich_rust`'s core remains output-focused; these helpers are best-effort
//! and fall back gracefully when the console is not interactive.

use std::io;
use std::io::Write as _;
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
    message: Arc<Mutex<String>>,
    live: Option<Live>,
}

impl Status {
    /// Start a status spinner with a message.
    pub fn new(console: &Arc<Console>, message: impl Into<String>) -> io::Result<Self> {
        let message = Arc::new(Mutex::new(message.into()));

        if !console.is_interactive() {
            if let Ok(message) = message.lock() {
                console.print_plain(&message);
            }
            return Ok(Self {
                message,
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

        let live =
            Live::with_options(Arc::clone(console), live_options).get_renderable(move || {
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
            message,
            live: Some(live),
        })
    }

    /// Update the displayed message (best-effort).
    pub fn update(&self, message: impl Into<String>) {
        if let Ok(mut slot) = self.message.lock() {
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

type PromptValidator = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// Prompt configuration.
#[derive(Clone)]
pub struct Prompt {
    label: String,
    default: Option<String>,
    allow_empty: bool,
    show_default: bool,
    markup: bool,
    validator: Option<PromptValidator>,
}

impl std::fmt::Debug for Prompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Prompt")
            .field("label", &self.label)
            .field("default", &self.default)
            .field("allow_empty", &self.allow_empty)
            .field("show_default", &self.show_default)
            .field("markup", &self.markup)
            .field("validator", &self.validator.as_ref().map(|_| "<validator>"))
            .finish()
    }
}

impl Prompt {
    /// Create a new prompt.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
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
    pub fn ask(&self, console: &Console) -> Result<String, PromptError> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        self.ask_from(console, &mut reader)
    }

    /// Ask for input from a provided reader (useful for tests).
    pub fn ask_from<R: io::BufRead>(
        &self,
        console: &Console,
        reader: &mut R,
    ) -> Result<String, PromptError> {
        if !console.is_terminal() {
            return self.default.clone().ok_or(PromptError::NotInteractive);
        }

        let mut line = String::new();
        loop {
            self.print_prompt(console);

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
                self.print_error(console, "Input required.");
                continue;
            }

            if let Some(validator) = &self.validator
                && let Err(message) = validator(&value)
            {
                self.print_error(console, &message);
                continue;
            }

            value = value.trim_end().to_string();
            return Ok(value);
        }
    }

    fn print_prompt(&self, console: &Console) {
        let mut prompt = self.label.clone();
        if self.show_default
            && let Some(default) = &self.default
        {
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
            &PrintOptions::new().with_markup(false).with_style(style),
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

impl Default for Pager {
    fn default() -> Self {
        Self::new()
    }
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
        if !console.is_terminal() {
            print_exact(console, content);
            return Ok(());
        }

        let (command, args) = self.resolve_command();
        match spawn_pager(&command, &args, content) {
            Ok(()) => Ok(()),
            Err(_err) => {
                print_exact(console, content);
                Ok(())
            }
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

fn print_exact(console: &Console, content: &str) {
    console.print_with_options(
        content,
        &PrintOptions::new().with_markup(false).with_no_newline(true),
    );
}

fn trim_newline(line: &str) -> &str {
    line.trim_end_matches(&['\n', '\r'][..])
}

/// A choice for the Select prompt.
#[derive(Debug, Clone)]
pub struct Choice {
    /// The value returned when this choice is selected.
    pub value: String,
    /// Optional display label (if different from value).
    pub label: Option<String>,
}

impl Choice {
    /// Create a choice where value and label are the same.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
        }
    }

    /// Create a choice with a separate display label.
    #[must_use]
    pub fn with_label(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: Some(label.into()),
        }
    }

    /// Get the display text for this choice.
    #[must_use]
    pub fn display(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.value)
    }
}

impl<S: Into<String>> From<S> for Choice {
    fn from(value: S) -> Self {
        Self::new(value)
    }
}

/// Select prompt for choosing from a list of options.
///
/// Displays numbered choices and allows selection by number or by typing
/// the choice value directly.
///
/// # Examples
///
/// ```rust,ignore
/// use rich_rust::interactive::Select;
///
/// let color = Select::new("Pick a color")
///     .choices(["red", "green", "blue"])
///     .default("blue")
///     .ask(&console)?;
/// ```
#[derive(Debug, Clone)]
pub struct Select {
    label: String,
    choices: Vec<Choice>,
    default: Option<String>,
    show_default: bool,
    markup: bool,
}

impl Select {
    /// Create a new select prompt.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            choices: Vec::new(),
            default: None,
            show_default: true,
            markup: true,
        }
    }

    /// Add choices to select from.
    #[must_use]
    pub fn choices<I, C>(mut self, choices: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Choice>,
    {
        self.choices.extend(choices.into_iter().map(Into::into));
        self
    }

    /// Add a single choice.
    #[must_use]
    pub fn choice(mut self, choice: impl Into<Choice>) -> Self {
        self.choices.push(choice.into());
        self
    }

    /// Set the default choice (used when user enters empty input or in non-interactive mode).
    #[must_use]
    pub fn default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Show/hide the default value in the prompt.
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

    /// Ask for selection using stdin.
    pub fn ask(&self, console: &Console) -> Result<String, PromptError> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        self.ask_from(console, &mut reader)
    }

    /// Ask for selection from a provided reader (useful for tests).
    pub fn ask_from<R: io::BufRead>(
        &self,
        console: &Console,
        reader: &mut R,
    ) -> Result<String, PromptError> {
        if self.choices.is_empty() {
            return Err(PromptError::Validation("No choices provided".to_string()));
        }

        if !console.is_terminal() {
            return self.default.clone().ok_or(PromptError::NotInteractive);
        }

        let mut line = String::new();
        loop {
            self.print_choices(console);
            self.print_prompt(console);

            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                return Err(PromptError::Eof);
            }

            let input = trim_newline(&line).trim();

            // Empty input uses default
            if input.is_empty() {
                if let Some(default) = &self.default {
                    if self.find_choice(default).is_some() {
                        return Ok(default.clone());
                    }
                }
                self.print_error(console, "Please select an option.");
                continue;
            }

            // Try as number first
            if let Ok(num) = input.parse::<usize>() {
                if num >= 1 && num <= self.choices.len() {
                    return Ok(self.choices[num - 1].value.clone());
                }
            }

            // Try as exact match (case insensitive)
            if let Some(choice) = self.find_choice(input) {
                return Ok(choice.value.clone());
            }

            self.print_error(console, &format!("Invalid choice: {input}"));
        }
    }

    fn find_choice(&self, input: &str) -> Option<&Choice> {
        let input_lower = input.to_lowercase();
        self.choices
            .iter()
            .find(|c| c.value.to_lowercase() == input_lower || c.display().to_lowercase() == input_lower)
    }

    fn print_choices(&self, console: &Console) {
        for (i, choice) in self.choices.iter().enumerate() {
            let num = i + 1;
            let display = choice.display();
            let is_default = self.default.as_deref() == Some(&choice.value);

            let line = if is_default && self.show_default {
                format!("  [bold cyan]{num}.[/] {display} [dim](default)[/]")
            } else {
                format!("  [cyan]{num}.[/] {display}")
            };

            console.print_with_options(
                &line,
                &PrintOptions::new().with_markup(self.markup),
            );
        }
    }

    fn print_prompt(&self, console: &Console) {
        let mut prompt = self.label.clone();
        if self.show_default {
            if let Some(default) = &self.default {
                let default_display = self
                    .find_choice(default)
                    .map(Choice::display)
                    .unwrap_or(default.as_str());
                let escaped = if self.markup {
                    markup::escape(default_display)
                } else {
                    default_display.to_string()
                };
                prompt.push_str(" [");
                prompt.push_str(&escaped);
                prompt.push(']');
            }
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
            &PrintOptions::new().with_markup(false).with_style(style),
        );
    }
}

/// Confirm prompt (yes/no question).
///
/// # Examples
///
/// ```rust,ignore
/// use rich_rust::interactive::Confirm;
///
/// let proceed = Confirm::new("Continue?")
///     .default(true)
///     .ask(&console)?;
/// ```
#[derive(Debug, Clone)]
pub struct Confirm {
    label: String,
    default: Option<bool>,
    markup: bool,
}

impl Confirm {
    /// Create a new confirmation prompt.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            default: None,
            markup: true,
        }
    }

    /// Set the default value.
    #[must_use]
    pub const fn default(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    /// Enable/disable markup parsing for the prompt label.
    #[must_use]
    pub const fn markup(mut self, markup: bool) -> Self {
        self.markup = markup;
        self
    }

    /// Ask for confirmation using stdin.
    pub fn ask(&self, console: &Console) -> Result<bool, PromptError> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        self.ask_from(console, &mut reader)
    }

    /// Ask for confirmation from a provided reader (useful for tests).
    pub fn ask_from<R: io::BufRead>(
        &self,
        console: &Console,
        reader: &mut R,
    ) -> Result<bool, PromptError> {
        if !console.is_terminal() {
            return self.default.ok_or(PromptError::NotInteractive);
        }

        let mut line = String::new();
        loop {
            self.print_prompt(console);

            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                return Err(PromptError::Eof);
            }

            let input = trim_newline(&line).trim().to_lowercase();

            if input.is_empty() {
                if let Some(default) = self.default {
                    return Ok(default);
                }
                self.print_error(console, "Please enter y or n.");
                continue;
            }

            match input.as_str() {
                "y" | "yes" | "true" | "1" => return Ok(true),
                "n" | "no" | "false" | "0" => return Ok(false),
                _ => {
                    self.print_error(console, "Please enter y or n.");
                }
            }
        }
    }

    fn print_prompt(&self, console: &Console) {
        let mut prompt = self.label.clone();

        let choices = match self.default {
            Some(true) => "[Y/n]",
            Some(false) => "[y/N]",
            None => "[y/n]",
        };
        prompt.push(' ');
        prompt.push_str(choices);
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
            &PrintOptions::new().with_markup(false).with_style(style),
        );
    }
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

        let _status = Status::new(&console, "Working...").expect("status");

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
        let answer = prompt.ask(&console).expect("prompt");
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
        let answer = prompt.ask_from(&console, &mut reader).expect("prompt");
        assert_eq!(answer, "42");

        let out = buffer.0.lock().unwrap();
        let text = String::from_utf8_lossy(&out);
        // The error message may have ANSI codes around it due to the bold red style,
        // so we just check for the text content rather than a literal sequence with newline.
        assert!(
            text.contains("digits only"),
            "Expected error message 'digits only' in output, got: {text:?}"
        );
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

    #[test]
    fn test_select_by_number() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let select = Select::new("Pick a color").choices(["red", "green", "blue"]);

        let input = b"2\n";
        let mut reader = io::Cursor::new(&input[..]);
        let answer = select.ask_from(&console, &mut reader).expect("select");
        assert_eq!(answer, "green");
    }

    #[test]
    fn test_select_by_value() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let select = Select::new("Pick a color").choices(["red", "green", "blue"]);

        let input = b"blue\n";
        let mut reader = io::Cursor::new(&input[..]);
        let answer = select.ask_from(&console, &mut reader).expect("select");
        assert_eq!(answer, "blue");
    }

    #[test]
    fn test_select_case_insensitive() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let select = Select::new("Pick").choices(["Red", "Green"]);

        let input = b"red\n";
        let mut reader = io::Cursor::new(&input[..]);
        let answer = select.ask_from(&console, &mut reader).expect("select");
        assert_eq!(answer, "Red");
    }

    #[test]
    fn test_select_default() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let select = Select::new("Pick").choices(["a", "b", "c"]).default("b");

        let input = b"\n"; // Empty input
        let mut reader = io::Cursor::new(&input[..]);
        let answer = select.ask_from(&console, &mut reader).expect("select");
        assert_eq!(answer, "b");
    }

    #[test]
    fn test_select_non_interactive_uses_default() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(false)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let select = Select::new("Pick").choices(["a", "b"]).default("b");
        let answer = select.ask(&console).expect("select");
        assert_eq!(answer, "b");
    }

    #[test]
    fn test_select_with_labels() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let select = Select::new("Pick")
            .choice(Choice::with_label("us-east-1", "US East (N. Virginia)"))
            .choice(Choice::with_label("eu-west-1", "EU (Ireland)"));

        let input = b"1\n";
        let mut reader = io::Cursor::new(&input[..]);
        let answer = select.ask_from(&console, &mut reader).expect("select");
        assert_eq!(answer, "us-east-1");
    }

    #[test]
    fn test_confirm_yes() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let confirm = Confirm::new("Continue?");

        let input = b"y\n";
        let mut reader = io::Cursor::new(&input[..]);
        let answer = confirm.ask_from(&console, &mut reader).expect("confirm");
        assert!(answer);
    }

    #[test]
    fn test_confirm_no() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let confirm = Confirm::new("Continue?");

        let input = b"n\n";
        let mut reader = io::Cursor::new(&input[..]);
        let answer = confirm.ask_from(&console, &mut reader).expect("confirm");
        assert!(!answer);
    }

    #[test]
    fn test_confirm_default_yes() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let confirm = Confirm::new("Continue?").default(true);

        let input = b"\n"; // Empty input
        let mut reader = io::Cursor::new(&input[..]);
        let answer = confirm.ask_from(&console, &mut reader).expect("confirm");
        assert!(answer);
    }

    #[test]
    fn test_confirm_non_interactive_uses_default() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(false)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();

        let confirm = Confirm::new("Continue?").default(false);
        let answer = confirm.ask(&console).expect("confirm");
        assert!(!answer);
    }

    #[test]
    fn test_choice_display() {
        let simple = Choice::new("value");
        assert_eq!(simple.display(), "value");

        let labeled = Choice::with_label("value", "Display Label");
        assert_eq!(labeled.display(), "Display Label");
    }
}
