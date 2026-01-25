//! Logging integration similar to Python Rich's `RichHandler`.
//!
//! Optional tracing integration is available via `RichTracingLayer` when the
//! `tracing` feature is enabled.

use std::sync::{Arc, Mutex};

use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use time::{OffsetDateTime, format_description::OwnedFormatItem};

use crate::console::Console;
use crate::markup;
use crate::style::Style;
use crate::text::Text;

const DEFAULT_KEYWORDS: [&str; 8] = [
    "GET", "POST", "HEAD", "PUT", "DELETE", "OPTIONS", "TRACE", "PATCH",
];

/// Rich-style logger for the `log` crate.
pub struct RichLogger {
    console: Arc<Console>,
    level: LevelFilter,
    show_time: bool,
    omit_repeated_times: bool,
    show_level: bool,
    show_path: bool,
    enable_link_path: bool,
    markup: bool,
    keywords: Vec<String>,
    time_format: OwnedFormatItem,
    last_time: Mutex<Option<String>>,
    keyword_style: Style,
}

impl RichLogger {
    /// Create a new `RichLogger` with default settings.
    #[must_use]
    pub fn new(console: Arc<Console>) -> Self {
        let time_format =
            time::format_description::parse_owned::<2>("[%F %T]").unwrap_or_else(|_| {
                time::format_description::parse_owned::<2>("[hour]:[minute]:[second]")
                    .expect("fallback time format should parse")
            });
        Self {
            console,
            level: LevelFilter::Info,
            show_time: true,
            omit_repeated_times: true,
            show_level: true,
            show_path: true,
            enable_link_path: true,
            markup: false,
            keywords: DEFAULT_KEYWORDS.iter().map(ToString::to_string).collect(),
            time_format,
            last_time: Mutex::new(None),
            keyword_style: Style::parse("bold yellow").unwrap_or_default(),
        }
    }

    /// Set the minimum log level.
    #[must_use]
    pub fn level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// Enable or disable timestamps.
    #[must_use]
    pub fn show_time(mut self, show: bool) -> Self {
        self.show_time = show;
        self
    }

    /// Omit repeated timestamps.
    #[must_use]
    pub fn omit_repeated_times(mut self, omit: bool) -> Self {
        self.omit_repeated_times = omit;
        self
    }

    /// Enable or disable log levels.
    #[must_use]
    pub fn show_level(mut self, show: bool) -> Self {
        self.show_level = show;
        self
    }

    /// Enable or disable path column.
    #[must_use]
    pub fn show_path(mut self, show: bool) -> Self {
        self.show_path = show;
        self
    }

    /// Enable terminal hyperlinks for paths.
    #[must_use]
    pub fn enable_link_path(mut self, enable: bool) -> Self {
        self.enable_link_path = enable;
        self
    }

    /// Enable Rich markup parsing for messages.
    #[must_use]
    pub fn markup(mut self, markup: bool) -> Self {
        self.markup = markup;
        self
    }

    /// Override keyword list.
    #[must_use]
    pub fn keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Override time format.
    #[must_use]
    pub fn time_format(mut self, format: &str) -> Self {
        if let Ok(parsed) = time::format_description::parse_owned::<2>(format) {
            self.time_format = parsed;
        }
        self
    }

    /// Install as the global logger.
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_max_level(self.level);
        log::set_boxed_logger(Box::new(self))
    }

    fn format_time(&self) -> String {
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        now.format(&self.time_format)
            .unwrap_or_else(|_| now.to_string())
    }

    fn level_style(level: Level) -> Style {
        match level {
            Level::Trace => Style::parse("dim").unwrap_or_default(),
            Level::Debug => Style::parse("blue dim").unwrap_or_default(),
            Level::Info => Style::parse("green").unwrap_or_default(),
            Level::Warn => Style::parse("yellow").unwrap_or_default(),
            Level::Error => Style::parse("bold red").unwrap_or_default(),
        }
    }

    fn format_record(&self, record: &Record<'_>) -> Text {
        let mut line = Text::new("");

        if self.show_time {
            let time_str = self.format_time();
            let display = if self.omit_repeated_times {
                let mut last = self
                    .last_time
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if last.as_ref() == Some(&time_str) {
                    " ".repeat(time_str.len())
                } else {
                    *last = Some(time_str.clone());
                    time_str.clone()
                }
            } else {
                time_str
            };
            line.append(&display);
            line.append(" ");
        }

        if self.show_level {
            let level_name = record.level().to_string();
            let padded = format!("{level_name:<8}");
            line.append_styled(&padded, Self::level_style(record.level()));
            line.append(" ");
        }

        let mut message = if self.markup {
            markup::render_or_plain(&record.args().to_string())
        } else {
            Text::new(record.args().to_string())
        };

        if !self.keywords.is_empty() {
            let keywords: Vec<&str> = self.keywords.iter().map(String::as_str).collect();
            message.highlight_words(&keywords, &self.keyword_style, false);
        }

        line.append_text(&message);

        if self.show_path
            && let Some(path) = record.file()
        {
            let mut path_text = Text::new(" ");
            let style = if self.enable_link_path {
                Style::new().link(format!("file://{path}"))
            } else {
                Style::default()
            };
            path_text.append_styled(path, style.clone());
            if let Some(line_no) = record.line() {
                path_text.append(":");
                let line_style = if self.enable_link_path {
                    Style::new().link(format!("file://{path}#{line_no}"))
                } else {
                    Style::default()
                };
                path_text.append_styled(&line_no.to_string(), line_style);
            }
            line.append_text(&path_text);
        }

        line
    }
}

impl Log for RichLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let text = self.format_record(record);
        self.console.print_text(&text);
    }

    fn flush(&self) {}
}

#[cfg(feature = "tracing")]
mod tracing_integration {
    use super::*;
    use std::fmt::Debug;

    use tracing::field::{Field, Visit};
    use tracing::{Event, Level as TracingLevel, Subscriber};
    use tracing_subscriber::{Layer, layer::Context};

    /// Tracing layer that formats events using RichLogger styling.
    pub struct RichTracingLayer {
        logger: RichLogger,
    }

    impl RichTracingLayer {
        /// Create a tracing layer backed by a RichLogger.
        #[must_use]
        pub fn new(console: Arc<Console>) -> Self {
            Self {
                logger: RichLogger::new(console),
            }
        }

        /// Use an existing logger configuration.
        #[must_use]
        pub fn with_logger(logger: RichLogger) -> Self {
            Self { logger }
        }

        /// Install as the global tracing subscriber.
        pub fn init(self) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
            use tracing_subscriber::prelude::*;

            let subscriber = tracing_subscriber::registry().with(self);
            tracing::subscriber::set_global_default(subscriber)
        }
    }

    #[derive(Default)]
    struct EventVisitor {
        message: Option<String>,
        fields: Vec<(String, String)>,
    }

    impl Visit for EventVisitor {
        fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
            let rendered = format!("{value:?}");
            let rendered = strip_quotes(&rendered).to_string();
            if field.name() == "message" {
                self.message = Some(rendered);
            } else {
                self.fields.push((field.name().to_string(), rendered));
            }
        }
    }

    impl<S> Layer<S> for RichTracingLayer
    where
        S: Subscriber,
    {
        fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
            let metadata = event.metadata();
            let mut visitor = EventVisitor::default();
            event.record(&mut visitor);

            let mut message = visitor.message.unwrap_or_default();
            if !visitor.fields.is_empty() {
                let extra = visitor
                    .fields
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                if message.is_empty() {
                    message = extra;
                } else {
                    message.push(' ');
                    message.push_str(&extra);
                }
            }

            let record = log::Record::builder()
                .args(format_args!("{message}"))
                .level(map_tracing_level(metadata.level()))
                .target(metadata.target())
                .file(metadata.file())
                .line(metadata.line())
                .module_path(metadata.module_path())
                .build();

            let text = self.logger.format_record(&record);
            self.logger.console.print_text(&text);
        }
    }

    fn map_tracing_level(level: &TracingLevel) -> Level {
        match *level {
            TracingLevel::TRACE => Level::Trace,
            TracingLevel::DEBUG => Level::Debug,
            TracingLevel::INFO => Level::Info,
            TracingLevel::WARN => Level::Warn,
            TracingLevel::ERROR => Level::Error,
        }
    }

    fn strip_quotes(value: &str) -> &str {
        if value.len() >= 2 && value.starts_with('\"') && value.ends_with('\"') {
            &value[1..value.len() - 1]
        } else {
            value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_strip_quotes() {
            assert_eq!(strip_quotes("\"hello\""), "hello");
            assert_eq!(strip_quotes("plain"), "plain");
        }
    }
}

#[cfg(feature = "tracing")]
pub use tracing_integration::RichTracingLayer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_record_includes_message_and_path() {
        let console = Arc::new(Console::builder().markup(false).build());
        let logger = RichLogger::new(console)
            .show_time(false)
            .show_level(false)
            .show_path(true)
            .enable_link_path(false);

        let record = log::Record::builder()
            .args(format_args!("Hello"))
            .level(Level::Info)
            .file(Some("main.rs"))
            .line(Some(42))
            .build();

        let text = logger.format_record(&record);
        let plain = text.plain();
        assert!(plain.contains("Hello"));
        assert!(plain.contains("main.rs:42"));
    }
}
