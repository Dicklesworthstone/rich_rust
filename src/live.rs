//! Live display system for dynamic terminal updates.
//!
//! This module implements Rich-style Live updates with cursor control.

use std::io;
use std::sync::{
    Arc, Mutex, RwLock,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::console::{Console, ConsoleOptions, RenderHook};
use crate::renderables::Renderable;
use crate::segment::{ControlCode, ControlType, Segment, split_lines};
use crate::style::Style;
use crate::text::{JustifyMethod, OverflowMethod, Text};

/// Vertical overflow handling for Live renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalOverflowMethod {
    Crop,
    #[default]
    Ellipsis,
    Visible,
}

/// Configuration for Live.
#[derive(Debug, Clone)]
pub struct LiveOptions {
    pub screen: bool,
    pub auto_refresh: bool,
    pub refresh_per_second: f64,
    pub transient: bool,
    pub redirect_stdout: bool,
    pub redirect_stderr: bool,
    pub vertical_overflow: VerticalOverflowMethod,
}

impl Default for LiveOptions {
    fn default() -> Self {
        Self {
            screen: false,
            auto_refresh: true,
            refresh_per_second: 4.0,
            transient: false,
            redirect_stdout: true,
            redirect_stderr: true,
            vertical_overflow: VerticalOverflowMethod::Ellipsis,
        }
    }
}

/// Live display handle.
#[derive(Clone)]
pub struct Live {
    inner: Arc<LiveInner>,
}

/// Write-only proxy that routes output through the Console during Live.
#[derive(Clone)]
pub struct LiveWriter {
    console: Arc<Console>,
}

impl LiveWriter {
    #[must_use]
    pub fn new(console: Arc<Console>) -> Self {
        Self { console }
    }
}

impl io::Write for LiveWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        self.console.print_plain(&text);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

type RenderableFactory = Arc<dyn Fn() -> Box<dyn Renderable + Send + Sync> + Send + Sync>;

pub(crate) struct LiveInner {
    console: Arc<Console>,
    options: Mutex<LiveOptions>,
    renderable: RwLock<Option<Box<dyn Renderable + Send + Sync>>>,
    get_renderable: Mutex<Option<RenderableFactory>>,
    started: AtomicBool,
    nested: AtomicBool,
    alt_screen_active: AtomicBool,
    refresh_stop: Arc<AtomicBool>,
    refresh_thread: Mutex<Option<JoinHandle<()>>>,
    live_render: Mutex<LiveRender>,
}

impl Live {
    /// Create a Live instance.
    #[must_use]
    pub fn new(console: Arc<Console>) -> Self {
        Self::with_options(console, LiveOptions::default())
    }

    /// Create a Live instance with explicit options.
    #[must_use]
    pub fn with_options(console: Arc<Console>, options: LiveOptions) -> Self {
        assert!(
            options.refresh_per_second > 0.0,
            "refresh_per_second must be > 0"
        );
        let mut options = options;
        if options.screen {
            options.transient = true;
        }
        Self {
            inner: Arc::new(LiveInner {
                console,
                options: Mutex::new(options),
                renderable: RwLock::new(None),
                get_renderable: Mutex::new(None),
                started: AtomicBool::new(false),
                nested: AtomicBool::new(false),
                alt_screen_active: AtomicBool::new(false),
                refresh_stop: Arc::new(AtomicBool::new(false)),
                refresh_thread: Mutex::new(None),
                live_render: Mutex::new(LiveRender::default()),
            }),
        }
    }

    /// Set a static renderable to display.
    #[must_use]
    pub fn renderable<R>(self, renderable: R) -> Self
    where
        R: Renderable + Send + Sync + 'static,
    {
        if let Ok(mut slot) = self.inner.renderable.write() {
            *slot = Some(Box::new(renderable));
        }
        self
    }

    /// Set a callback to provide dynamic renderables.
    #[must_use]
    pub fn get_renderable<F>(self, callback: F) -> Self
    where
        F: Fn() -> Box<dyn Renderable + Send + Sync> + Send + Sync + 'static,
    {
        if let Ok(mut slot) = self.inner.get_renderable.lock() {
            *slot = Some(Arc::new(callback));
        }
        self
    }

    /// Start the Live display.
    pub fn start(&self, refresh: bool) -> io::Result<()> {
        if self.inner.started.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        if !self.inner.console.set_live(&self.inner) {
            self.inner.nested.store(true, Ordering::SeqCst);
            return Ok(());
        }

        let options = self.inner.options();
        if options.screen {
            self.inner.console.set_alt_screen(true)?;
            self.inner.alt_screen_active.store(true, Ordering::SeqCst);
        }

        self.inner.console.show_cursor(false)?;

        // Redirect stdout/stderr is best-effort; currently only applies to Console output.
        self.inner
            .console
            .push_render_hook(Arc::clone(&self.inner) as Arc<dyn RenderHook>);

        if refresh {
            self.refresh()?;
        }

        if options.auto_refresh {
            Arc::clone(&self.inner).start_refresh_thread();
        }

        Ok(())
    }

    /// Stop the Live display.
    pub fn stop(&self) -> io::Result<()> {
        if !self.inner.started.swap(false, Ordering::SeqCst) {
            return Ok(());
        }

        self.inner.stop_refresh_thread();
        self.inner.console.clear_live();

        if self.inner.nested.load(Ordering::SeqCst) {
            return Ok(());
        }

        {
            let mut options = self.inner.options_mut();
            options.vertical_overflow = VerticalOverflowMethod::Visible;
        }

        if !self.inner.alt_screen_active.load(Ordering::SeqCst) && self.inner.console.is_terminal()
        {
            let _ = self.refresh();
            self.inner.console.line();
        }

        self.inner.console.pop_render_hook();
        let _ = self.inner.console.show_cursor(true);

        if self.inner.alt_screen_active.swap(false, Ordering::SeqCst) {
            let _ = self.inner.console.set_alt_screen(false);
        }

        if self.inner.options().transient && !self.inner.alt_screen_active.load(Ordering::SeqCst) {
            let controls = self.inner.live_render_controls_restore();
            let _ = self.inner.console.write_control_codes(controls);
        }

        Ok(())
    }

    /// Update the renderable content.
    pub fn update<R>(&self, renderable: R, refresh: bool)
    where
        R: Renderable + Send + Sync + 'static,
    {
        if let Ok(mut slot) = self.inner.renderable.write() {
            *slot = Some(Box::new(renderable));
        }
        if refresh {
            let _ = self.refresh();
        }
    }

    /// Refresh the live display.
    pub fn refresh(&self) -> io::Result<()> {
        self.inner.refresh_display()
    }

    /// Create a stdout proxy writer that routes output through the Console.
    #[must_use]
    pub fn stdout_proxy(&self) -> LiveWriter {
        LiveWriter::new(Arc::clone(&self.inner.console))
    }

    /// Create a stderr proxy writer that routes output through the Console.
    #[must_use]
    pub fn stderr_proxy(&self) -> LiveWriter {
        LiveWriter::new(Arc::clone(&self.inner.console))
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl LiveInner {
    fn options(&self) -> LiveOptions {
        self.options.lock().map(|o| o.clone()).unwrap_or_default()
    }

    fn options_mut(&self) -> std::sync::MutexGuard<'_, LiveOptions> {
        self.options.lock().expect("Live options mutex poisoned")
    }

    fn current_renderable(
        &self,
        console: &Console,
        options: &ConsoleOptions,
    ) -> Vec<Segment<'static>> {
        let callback = self
            .get_renderable
            .lock()
            .ok()
            .and_then(|slot| slot.clone());
        if let Some(callback) = callback {
            let renderable = callback();
            return renderable
                .render(console, options)
                .into_iter()
                .map(Segment::into_owned)
                .collect();
        }

        if let Ok(slot) = self.renderable.read()
            && let Some(renderable) = slot.as_ref()
        {
            return renderable
                .render(console, options)
                .into_iter()
                .map(Segment::into_owned)
                .collect();
        }

        Vec::new()
    }

    fn render_stack_segments(
        &self,
        console: &Console,
        options: &ConsoleOptions,
    ) -> Vec<Segment<'static>> {
        let lives = console.live_stack_snapshot();
        if lives.is_empty() {
            return self.current_renderable(console, options);
        }

        let mut output = Vec::new();
        for (idx, live) in lives.iter().enumerate() {
            let segments = live.current_renderable(console, options);
            if idx > 0 && !segments.is_empty() {
                output.push(Segment::line());
            }
            output.extend(segments);
        }
        output
    }

    fn live_render_controls_restore(&self) -> Vec<ControlCode> {
        self.live_render
            .lock()
            .map(|render| render.restore_cursor_controls())
            .unwrap_or_default()
    }

    fn render_live_segments(
        &self,
        render: &mut LiveRender,
        console: &Console,
        options: &ConsoleOptions,
        vertical_overflow: VerticalOverflowMethod,
    ) -> Vec<Segment<'static>> {
        let raw_segments = self.render_stack_segments(console, options);
        let mut lines = split_lines(raw_segments.into_iter());

        let max_height = options.size.height;
        let mut needs_ellipsis = false;
        if max_height > 0 && lines.len() > max_height {
            match vertical_overflow {
                VerticalOverflowMethod::Crop => {
                    lines.truncate(max_height);
                }
                VerticalOverflowMethod::Ellipsis => {
                    if max_height == 1 {
                        lines.truncate(1);
                    } else {
                        lines.truncate(max_height - 1);
                        needs_ellipsis = true;
                    }
                }
                VerticalOverflowMethod::Visible => {}
            }
        }

        if needs_ellipsis {
            let width = options.max_width;
            let mut ellipsis = Text::styled("...", Style::new().dim());
            ellipsis.overflow = OverflowMethod::Crop;
            ellipsis.justify = JustifyMethod::Center;
            ellipsis.pad(width, JustifyMethod::Center);
            let ellipsis_segments = ellipsis
                .render("")
                .into_iter()
                .map(Segment::into_owned)
                .collect();
            lines.push(ellipsis_segments);
        }

        let mut max_width = 0usize;
        for line in &lines {
            let line_width: usize = line.iter().map(Segment::cell_length).sum();
            max_width = max_width.max(line_width);
        }
        render.shape = Some((max_width, lines.len()));

        let mut flattened = Vec::new();
        let last_index = lines.len().saturating_sub(1);
        for (idx, mut line) in lines.into_iter().enumerate() {
            flattened.append(&mut line);
            if idx < last_index {
                flattened.push(Segment::line());
            }
        }
        flattened
    }

    fn start_refresh_thread(self: &Arc<Self>) {
        if self.refresh_stop.load(Ordering::Relaxed) {
            self.refresh_stop.store(false, Ordering::Relaxed);
        }

        let inner = Arc::clone(self);
        let interval = {
            let options = self.options();
            Duration::from_secs_f64(1.0 / options.refresh_per_second)
        };
        let stop = Arc::clone(&self.refresh_stop);

        let handle = thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                thread::sleep(interval);
                if !stop.load(Ordering::Relaxed) {
                    let _ = inner.refresh_display();
                }
            }
        });

        if let Ok(mut slot) = self.refresh_thread.lock() {
            *slot = Some(handle);
        }
    }

    fn stop_refresh_thread(&self) {
        self.refresh_stop.store(true, Ordering::Relaxed);
        if let Ok(mut slot) = self.refresh_thread.lock()
            && let Some(handle) = slot.take()
        {
            let _ = handle.join();
        }
    }

    fn refresh_display(&self) -> io::Result<()> {
        if self.nested.load(Ordering::SeqCst) {
            if let Some(parent) = self.console.live_stack_snapshot().first() {
                return parent.refresh_display();
            }
            return Ok(());
        }

        if (self.console.is_terminal() && !self.console.is_dumb_terminal())
            || !self.options().transient
        {
            self.console.print_segments(&[]);
        }
        Ok(())
    }
}

impl RenderHook for LiveInner {
    fn process(&self, console: &Console, segments: &[Segment<'static>]) -> Vec<Segment<'static>> {
        let options = console.options();
        let overflow = self.options().vertical_overflow;

        let Ok(mut render) = self.live_render.lock() else {
            return segments.to_vec();
        };

        let mut output = Vec::new();
        if console.is_interactive() {
            if self.alt_screen_active.load(Ordering::SeqCst) {
                output.push(Segment::control(vec![ControlCode::new(ControlType::Home)]));
            } else {
                let controls = render.position_cursor_controls();
                if !controls.is_empty() {
                    output.push(Segment::control(controls));
                }
            }
            output.extend_from_slice(segments);
            let live_segments = self.render_live_segments(&mut render, console, &options, overflow);
            output.extend(live_segments);
            output
        } else if !self.started.load(Ordering::SeqCst) && !self.options().transient {
            output.extend_from_slice(segments);
            let live_segments = self.render_live_segments(&mut render, console, &options, overflow);
            output.extend(live_segments);
            output
        } else {
            segments.to_vec()
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct LiveRender {
    shape: Option<(usize, usize)>,
}

impl LiveRender {
    fn position_cursor_controls(&self) -> Vec<ControlCode> {
        let Some((_, height)) = self.shape else {
            return Vec::new();
        };
        if height == 0 {
            return Vec::new();
        }

        let mut controls = Vec::new();
        controls.push(ControlCode::new(ControlType::CarriageReturn));
        controls.push(ControlCode::with_params(ControlType::EraseInLine, vec![2]));

        if height > 1 {
            for _ in 0..(height - 1) {
                controls.push(ControlCode::with_params(ControlType::CursorUp, vec![1]));
                controls.push(ControlCode::with_params(ControlType::EraseInLine, vec![2]));
            }
        }

        controls
    }

    fn restore_cursor_controls(&self) -> Vec<ControlCode> {
        let Some((_, height)) = self.shape else {
            return Vec::new();
        };
        if height == 0 {
            return Vec::new();
        }

        let mut controls = Vec::new();
        controls.push(ControlCode::new(ControlType::CarriageReturn));
        for _ in 0..height {
            controls.push(ControlCode::with_params(ControlType::CursorUp, vec![1]));
            controls.push(ControlCode::with_params(ControlType::EraseInLine, vec![2]));
        }
        controls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
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

    #[test]
    fn test_live_refresh_outputs_renderable() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();
        let options = LiveOptions {
            auto_refresh: false,
            screen: false,
            transient: false,
            ..LiveOptions::default()
        };
        let live = Live::with_options(console, options).renderable(Text::new("Hello"));
        live.start(true).expect("start");
        let _ = live.refresh();
        live.stop().expect("stop");

        let output = buffer.0.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("Hello"), "output missing: {text}");
    }

    #[test]
    fn test_live_vertical_overflow_ellipsis() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .width(10)
            .height(2)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();
        let options = LiveOptions {
            auto_refresh: false,
            screen: false,
            transient: false,
            vertical_overflow: VerticalOverflowMethod::Ellipsis,
            ..LiveOptions::default()
        };
        let live = Live::with_options(console, options).renderable(Text::new("a\nb\nc"));
        live.start(true).expect("start");
        let _ = live.refresh();
        live.stop().expect("stop");

        let output = buffer.0.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("..."), "expected ellipsis, got: {text}");
    }

    #[test]
    fn test_live_writer_proxy() {
        let buffer = SharedBuffer(Arc::new(Mutex::new(Vec::new())));
        let console = Console::builder()
            .force_terminal(true)
            .markup(false)
            .file(Box::new(buffer.clone()))
            .build()
            .shared();
        let live = Live::new(console.clone());
        let mut writer = live.stdout_proxy();
        let _ = writer.write_all(b"proxy output");

        let output = buffer.0.lock().unwrap();
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("proxy output"));
    }
}
