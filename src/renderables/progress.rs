//! Progress bar renderable.
//!
//! This module provides progress bar components for displaying task progress
//! in the terminal with various styles and features.

use crate::cells;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;
use std::time::{Duration, Instant};

/// Bar style variants for the progress bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarStyle {
    /// Standard ASCII bar using # and -
    Ascii,
    /// Unicode block characters (█▓░)
    #[default]
    Block,
    /// Thin line style (─╸)
    Line,
    /// Dots style (●○)
    Dots,
    /// Shaded gradient style (█▇▆▅▄▃▂▁░)
    Gradient,
}

impl BarStyle {
    /// Get the completed character for this style.
    #[must_use]
    pub const fn completed_char(&self) -> &str {
        match self {
            Self::Ascii => "#",
            Self::Block => "\u{2588}",    // █
            Self::Line => "\u{2501}",     // ━
            Self::Dots => "\u{25CF}",     // ●
            Self::Gradient => "\u{2588}", // █
        }
    }

    /// Get the remaining character for this style.
    #[must_use]
    pub const fn remaining_char(&self) -> &str {
        match self {
            Self::Ascii => "-",
            Self::Block => "\u{2591}",    // ░
            Self::Line => "\u{2500}",     // ─
            Self::Dots => "\u{25CB}",     // ○
            Self::Gradient => "\u{2591}", // ░
        }
    }

    /// Get the pulse character for this style (edge of completion).
    #[must_use]
    pub const fn pulse_char(&self) -> &str {
        match self {
            Self::Ascii => ">",
            Self::Block => "\u{2593}",    // ▓
            Self::Line => "\u{257A}",     // ╺
            Self::Dots => "\u{25CF}",     // ●
            Self::Gradient => "\u{2593}", // ▓
        }
    }
}

/// Spinner animation frames.
#[derive(Debug, Clone)]
pub struct Spinner {
    /// Animation frames.
    frames: Vec<&'static str>,
    /// Current frame index.
    frame_index: usize,
    /// Style for the spinner.
    style: Style,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::dots()
    }
}

impl Spinner {
    /// Create a dots spinner (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏).
    #[must_use]
    pub fn dots() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            frame_index: 0,
            style: Style::new(),
        }
    }

    /// Create a line spinner (⎺⎻⎼⎽⎼⎻).
    #[must_use]
    pub fn line() -> Self {
        Self {
            frames: vec!["⎺", "⎻", "⎼", "⎽", "⎼", "⎻"],
            frame_index: 0,
            style: Style::new(),
        }
    }

    /// Create a simple spinner (|/-\).
    #[must_use]
    pub fn simple() -> Self {
        Self {
            frames: vec!["|", "/", "-", "\\"],
            frame_index: 0,
            style: Style::new(),
        }
    }

    /// Create a bouncing ball spinner (⠁⠂⠄⠂).
    #[must_use]
    pub fn bounce() -> Self {
        Self {
            frames: vec!["⠁", "⠂", "⠄", "⠂"],
            frame_index: 0,
            style: Style::new(),
        }
    }

    /// Create a growing dots spinner (⣾⣽⣻⢿⡿⣟⣯⣷).
    #[must_use]
    pub fn growing() -> Self {
        Self {
            frames: vec!["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            frame_index: 0,
            style: Style::new(),
        }
    }

    /// Create a moon phase spinner (🌑🌒🌓🌔🌕🌖🌗🌘).
    #[must_use]
    pub fn moon() -> Self {
        Self {
            frames: vec!["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
            frame_index: 0,
            style: Style::new(),
        }
    }

    /// Create a clock spinner (🕐🕑🕒🕓🕔🕕🕖🕗🕘🕙🕚🕛).
    #[must_use]
    pub fn clock() -> Self {
        Self {
            frames: vec![
                "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛",
            ],
            frame_index: 0,
            style: Style::new(),
        }
    }

    /// Create a spinner from custom frames.
    #[must_use]
    pub fn custom(frames: Vec<&'static str>) -> Self {
        Self {
            frames,
            frame_index: 0,
            style: Style::new(),
        }
    }

    /// Set the spinner style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Advance to the next frame and return the current frame.
    pub fn next_frame(&mut self) -> &str {
        if self.frames.is_empty() {
            return " ";
        }
        let frame = self.frames[self.frame_index];
        self.frame_index = (self.frame_index + 1) % self.frames.len();
        frame
    }

    /// Get the current frame without advancing.
    #[must_use]
    pub fn current_frame(&self) -> &str {
        if self.frames.is_empty() {
            return " ";
        }
        self.frames[self.frame_index]
    }

    /// Render the current spinner frame as a segment.
    #[must_use]
    pub fn render(&self) -> Segment {
        Segment::new(self.current_frame(), Some(self.style.clone()))
    }
}

/// A progress bar with percentage, ETA, and customizable appearance.
#[derive(Debug, Clone)]
pub struct ProgressBar {
    /// Current progress (0.0 - 1.0).
    completed: f64,
    /// Total expected count (for ETA calculation).
    total: Option<u64>,
    /// Current count (for ETA calculation).
    current: u64,
    /// Bar width in cells.
    width: usize,
    /// Bar style.
    bar_style: BarStyle,
    /// Style for completed portion.
    completed_style: Style,
    /// Style for remaining portion.
    remaining_style: Style,
    /// Style for the pulse character.
    pulse_style: Style,
    /// Show percentage.
    show_percentage: bool,
    /// Show ETA.
    show_eta: bool,
    /// Show elapsed time.
    show_elapsed: bool,
    /// Show speed (items/sec).
    show_speed: bool,
    /// Task description.
    description: Option<Text>,
    /// Start time for ETA calculation.
    start_time: Option<Instant>,
    /// Whether to show brackets around the bar.
    show_brackets: bool,
    /// Finished message (replaces bar when complete).
    finished_message: Option<String>,
    /// Whether the task is complete.
    is_finished: bool,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self {
            completed: 0.0,
            total: None,
            current: 0,
            width: 40,
            bar_style: BarStyle::default(),
            completed_style: Style::new().color_str("green").unwrap_or_default(),
            remaining_style: Style::new().color_str("bright_black").unwrap_or_default(),
            pulse_style: Style::new().color_str("cyan").unwrap_or_default(),
            show_percentage: true,
            show_eta: false,
            show_elapsed: false,
            show_speed: false,
            description: None,
            start_time: None,
            show_brackets: true,
            finished_message: None,
            is_finished: false,
        }
    }
}

impl ProgressBar {
    /// Create a new progress bar.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a progress bar with a known total.
    #[must_use]
    pub fn with_total(total: u64) -> Self {
        Self {
            total: Some(total),
            show_eta: true,
            start_time: Some(Instant::now()),
            ..Self::default()
        }
    }

    /// Set the bar width.
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Set the bar style.
    #[must_use]
    pub fn bar_style(mut self, style: BarStyle) -> Self {
        self.bar_style = style;
        self
    }

    /// Set the completed portion style.
    #[must_use]
    pub fn completed_style(mut self, style: Style) -> Self {
        self.completed_style = style;
        self
    }

    /// Set the remaining portion style.
    #[must_use]
    pub fn remaining_style(mut self, style: Style) -> Self {
        self.remaining_style = style;
        self
    }

    /// Set the pulse character style.
    #[must_use]
    pub fn pulse_style(mut self, style: Style) -> Self {
        self.pulse_style = style;
        self
    }

    /// Set whether to show percentage.
    #[must_use]
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set whether to show ETA.
    #[must_use]
    pub fn show_eta(mut self, show: bool) -> Self {
        self.show_eta = show;
        if show && self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }
        self
    }

    /// Set whether to show elapsed time.
    #[must_use]
    pub fn show_elapsed(mut self, show: bool) -> Self {
        self.show_elapsed = show;
        if show && self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }
        self
    }

    /// Set whether to show speed.
    #[must_use]
    pub fn show_speed(mut self, show: bool) -> Self {
        self.show_speed = show;
        if show && self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }
        self
    }

    /// Set the task description.
    #[must_use]
    pub fn description(mut self, desc: impl Into<Text>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set whether to show brackets around the bar.
    #[must_use]
    pub fn show_brackets(mut self, show: bool) -> Self {
        self.show_brackets = show;
        self
    }

    /// Set the finished message.
    #[must_use]
    pub fn finished_message(mut self, msg: impl Into<String>) -> Self {
        self.finished_message = Some(msg.into());
        self
    }

    /// Update progress directly (0.0 - 1.0).
    pub fn set_progress(&mut self, progress: f64) {
        self.completed = progress.clamp(0.0, 1.0);
        if self.completed >= 1.0 {
            self.is_finished = true;
        }
    }

    /// Update progress with current/total counts.
    pub fn update(&mut self, current: u64) {
        self.current = current;
        if let Some(total) = self.total {
            if total > 0 {
                #[allow(clippy::cast_precision_loss)]
                {
                    self.completed = (current as f64) / (total as f64);
                }
            }
        }
        if self.completed >= 1.0 {
            self.is_finished = true;
        }
    }

    /// Advance progress by a delta.
    pub fn advance(&mut self, delta: u64) {
        self.update(self.current + delta);
    }

    /// Mark the progress bar as finished.
    pub fn finish(&mut self) {
        self.completed = 1.0;
        self.is_finished = true;
    }

    /// Get the current progress (0.0 - 1.0).
    #[must_use]
    pub fn progress(&self) -> f64 {
        self.completed
    }

    /// Check if the progress bar is finished.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.is_finished
    }

    /// Get the elapsed time since start.
    #[must_use]
    pub fn elapsed(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    /// Calculate estimated time remaining.
    #[must_use]
    pub fn eta(&self) -> Option<Duration> {
        if self.completed <= 0.0 || self.completed >= 1.0 {
            return None;
        }

        let elapsed = self.elapsed()?;
        let elapsed_secs = elapsed.as_secs_f64();
        if elapsed_secs < 0.1 {
            return None; // Not enough data
        }

        let remaining_ratio = (1.0 - self.completed) / self.completed;
        let eta_secs = elapsed_secs * remaining_ratio;

        Some(Duration::from_secs_f64(eta_secs))
    }

    /// Calculate items per second.
    #[must_use]
    pub fn speed(&self) -> Option<f64> {
        let elapsed = self.elapsed()?;
        let elapsed_secs = elapsed.as_secs_f64();
        if elapsed_secs < 0.1 {
            return None;
        }

        #[allow(clippy::cast_precision_loss)]
        Some((self.current as f64) / elapsed_secs)
    }

    /// Format a duration as a human-readable string.
    #[must_use]
    fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        if total_secs < 60 {
            format!("{total_secs}s")
        } else if total_secs < 3600 {
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            format!("{mins}:{secs:02}")
        } else {
            let hours = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            let secs = total_secs % 60;
            format!("{hours}:{mins:02}:{secs:02}")
        }
    }

    /// Render the progress bar to segments for a given width.
    #[must_use]
    pub fn render(&self, available_width: usize) -> Vec<Segment> {
        let mut segments = Vec::new();

        // If finished and has a finished message, show that
        if self.is_finished {
            if let Some(ref msg) = self.finished_message {
                let style = Style::new().color_str("green").unwrap_or_default();
                segments.push(Segment::new(&format!("✓ {msg}"), Some(style)));
                segments.push(Segment::line());
                return segments;
            }
        }

        // Description
        let mut used_width = 0;
        if let Some(ref desc) = self.description {
            let desc_text = format!("{} ", desc.plain());
            let desc_width = cells::cell_len(&desc_text);
            segments.push(Segment::new(&desc_text, Some(desc.style().clone())));
            used_width += desc_width;
        }

        // Calculate bar width
        let mut suffix_parts: Vec<String> = Vec::new();

        if self.show_percentage {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let pct = (self.completed * 100.0) as u32;
            suffix_parts.push(format!("{pct:3}%"));
        }

        if self.show_elapsed {
            if let Some(elapsed) = self.elapsed() {
                suffix_parts.push(Self::format_duration(elapsed));
            }
        }

        if self.show_eta && !self.is_finished {
            if let Some(eta) = self.eta() {
                suffix_parts.push(format!("ETA {}", Self::format_duration(eta)));
            }
        }

        if self.show_speed {
            if let Some(speed) = self.speed() {
                if speed >= 1.0 {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let speed_int = speed as u64;
                    suffix_parts.push(format!("{speed_int}/s"));
                } else {
                    suffix_parts.push(format!("{speed:.2}/s"));
                }
            }
        }

        let suffix = if suffix_parts.is_empty() {
            String::new()
        } else {
            format!(" {}", suffix_parts.join(" "))
        };
        let suffix_width = cells::cell_len(&suffix);

        let bracket_width = if self.show_brackets { 2 } else { 0 };
        let bar_width = available_width
            .saturating_sub(used_width)
            .saturating_sub(suffix_width)
            .saturating_sub(bracket_width)
            .min(self.width);

        if bar_width < 3 {
            // Not enough space for a bar, just show percentage
            if self.show_percentage {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let pct = (self.completed * 100.0) as u32;
                segments.push(Segment::new(&format!("{pct}%"), None));
            }
            segments.push(Segment::line());
            return segments;
        }

        // Render the bar
        if self.show_brackets {
            segments.push(Segment::new("[", None));
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let completed_width = ((self.completed * bar_width as f64).round() as usize).min(bar_width);
        let remaining_width = bar_width.saturating_sub(completed_width);

        // Completed portion
        if completed_width > 0 {
            let completed_chars = self.bar_style.completed_char().repeat(completed_width);
            segments.push(Segment::new(
                &completed_chars,
                Some(self.completed_style.clone()),
            ));
        }

        // Pulse character (at the edge)
        // Only show if not at start or end, and we have remaining width
        let show_pulse = completed_width > 0 && remaining_width > 0;
        if show_pulse {
            // Replace last remaining char with pulse
            let remaining_width = remaining_width.saturating_sub(1);
            segments.push(Segment::new(
                self.bar_style.pulse_char(),
                Some(self.pulse_style.clone()),
            ));

            if remaining_width > 0 {
                let remaining_chars = self.bar_style.remaining_char().repeat(remaining_width);
                segments.push(Segment::new(
                    &remaining_chars,
                    Some(self.remaining_style.clone()),
                ));
            }
        } else if remaining_width > 0 {
            let remaining_chars = self.bar_style.remaining_char().repeat(remaining_width);
            segments.push(Segment::new(
                &remaining_chars,
                Some(self.remaining_style.clone()),
            ));
        }

        if self.show_brackets {
            segments.push(Segment::new("]", None));
        }

        // Suffix (percentage, ETA, etc.)
        if !suffix.is_empty() {
            segments.push(Segment::new(&suffix, None));
        }

        segments.push(Segment::line());
        segments
    }

    /// Render the progress bar as a plain string.
    #[must_use]
    pub fn render_plain(&self, width: usize) -> String {
        self.render(width).into_iter().map(|seg| seg.text).collect()
    }
}

/// Create an ASCII-style progress bar.
#[must_use]
pub fn ascii_bar() -> ProgressBar {
    ProgressBar::new().bar_style(BarStyle::Ascii)
}

/// Create a line-style progress bar.
#[must_use]
pub fn line_bar() -> ProgressBar {
    ProgressBar::new().bar_style(BarStyle::Line)
}

/// Create a dots-style progress bar.
#[must_use]
pub fn dots_bar() -> ProgressBar {
    ProgressBar::new().bar_style(BarStyle::Dots)
}

/// Create a gradient-style progress bar.
#[must_use]
pub fn gradient_bar() -> ProgressBar {
    ProgressBar::new().bar_style(BarStyle::Gradient)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_new() {
        let bar = ProgressBar::new();
        assert!((bar.progress() - 0.0).abs() < f64::EPSILON);
        assert!(!bar.is_finished());
    }

    #[test]
    fn test_progress_bar_with_total() {
        let mut bar = ProgressBar::with_total(100);
        bar.update(50);
        assert!((bar.progress() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_bar_set_progress() {
        let mut bar = ProgressBar::new();
        bar.set_progress(0.75);
        assert!((bar.progress() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_bar_advance() {
        let mut bar = ProgressBar::with_total(10);
        bar.advance(3);
        assert!((bar.progress() - 0.3).abs() < f64::EPSILON);
        bar.advance(2);
        assert!((bar.progress() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_bar_finish() {
        let mut bar = ProgressBar::new();
        bar.finish();
        assert!(bar.is_finished());
        assert!((bar.progress() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_bar_render() {
        let mut bar = ProgressBar::new().width(20).show_brackets(true);
        bar.set_progress(0.5);
        let segments = bar.render(80);
        assert!(!segments.is_empty());
        let text: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(text.contains('['));
        assert!(text.contains(']'));
        assert!(text.contains('%'));
    }

    #[test]
    fn test_progress_bar_render_plain() {
        let mut bar = ProgressBar::new().width(10).show_brackets(false);
        bar.set_progress(0.5);
        let plain = bar.render_plain(40);
        assert!(!plain.is_empty());
    }

    #[test]
    fn test_progress_bar_styles() {
        for style in [
            BarStyle::Ascii,
            BarStyle::Block,
            BarStyle::Line,
            BarStyle::Dots,
        ] {
            let mut bar = ProgressBar::new().bar_style(style).width(10);
            bar.set_progress(0.5);
            let segments = bar.render(40);
            assert!(!segments.is_empty());
        }
    }

    #[test]
    fn test_progress_bar_with_description() {
        let mut bar = ProgressBar::new().description("Downloading").width(20);
        bar.set_progress(0.5);
        let plain = bar.render_plain(80);
        assert!(plain.contains("Downloading"));
    }

    #[test]
    fn test_progress_bar_finished_message() {
        let mut bar = ProgressBar::new().finished_message("Done!").width(20);
        bar.finish();
        let plain = bar.render_plain(80);
        assert!(plain.contains("Done!"));
        assert!(plain.contains('✓'));
    }

    #[test]
    fn test_spinner_next_frame() {
        let mut spinner = Spinner::simple();
        assert_eq!(spinner.next_frame(), "|");
        assert_eq!(spinner.next_frame(), "/");
        assert_eq!(spinner.next_frame(), "-");
        assert_eq!(spinner.next_frame(), "\\");
        assert_eq!(spinner.next_frame(), "|"); // Wraps around
    }

    #[test]
    fn test_spinner_current_frame() {
        let spinner = Spinner::simple();
        assert_eq!(spinner.current_frame(), "|");
        assert_eq!(spinner.current_frame(), "|"); // Doesn't advance
    }

    #[test]
    fn test_spinner_render() {
        let spinner = Spinner::dots();
        let segment = spinner.render();
        assert!(!segment.text.is_empty());
    }

    #[test]
    fn test_bar_style_chars() {
        assert_eq!(BarStyle::Ascii.completed_char(), "#");
        assert_eq!(BarStyle::Ascii.remaining_char(), "-");
        assert_eq!(BarStyle::Block.completed_char(), "\u{2588}");
        assert_eq!(BarStyle::Block.remaining_char(), "\u{2591}");
    }

    #[test]
    fn test_ascii_bar() {
        let mut bar = ascii_bar();
        bar.set_progress(0.5);
        let plain = bar.render_plain(40);
        assert!(plain.contains('#'));
        assert!(plain.contains('-'));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(ProgressBar::format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(
            ProgressBar::format_duration(Duration::from_secs(90)),
            "1:30"
        );
        assert_eq!(
            ProgressBar::format_duration(Duration::from_secs(3661)),
            "1:01:01"
        );
    }

    #[test]
    fn test_progress_clamp() {
        let mut bar = ProgressBar::new();
        bar.set_progress(-0.5);
        assert!((bar.progress() - 0.0).abs() < f64::EPSILON);
        bar.set_progress(1.5);
        assert!((bar.progress() - 1.0).abs() < f64::EPSILON);
    }
}
