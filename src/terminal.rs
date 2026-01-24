//! Terminal detection and manipulation.
//!
//! This module provides functionality to detect terminal capabilities
//! and query terminal dimensions.

use std::io::IsTerminal;

use crate::color::ColorSystem;

struct EnvSettings {
    no_color: Option<String>,
    force_color: Option<String>,
    colorterm: Option<String>,
    term: Option<String>,
    #[cfg(windows)]
    wt_session: Option<String>,
}

fn read_env_settings() -> EnvSettings {
    EnvSettings {
        no_color: std::env::var("NO_COLOR").ok(),
        force_color: std::env::var("FORCE_COLOR").ok(),
        colorterm: std::env::var("COLORTERM").ok(),
        term: std::env::var("TERM").ok(),
        #[cfg(windows)]
        wt_session: std::env::var("WT_SESSION").ok(),
    }
}

/// Get the terminal size (width, height) in cells.
///
/// Returns `None` if the terminal size cannot be determined.
#[must_use]
pub fn get_terminal_size() -> Option<(usize, usize)> {
    crossterm::terminal::size()
        .ok()
        .map(|(w, h)| (w as usize, h as usize))
}

/// Get the terminal width in cells.
///
/// Returns a default of 80 if the width cannot be determined.
#[must_use]
pub fn get_terminal_width() -> usize {
    get_terminal_size().map_or(80, |(w, _)| w)
}

/// Get the terminal height in rows.
///
/// Returns a default of 24 if the height cannot be determined.
#[must_use]
pub fn get_terminal_height() -> usize {
    get_terminal_size().map_or(24, |(_, h)| h)
}

/// Check if stdout is connected to a terminal.
#[must_use]
pub fn is_terminal() -> bool {
    std::io::stdout().is_terminal()
}

/// Check if stderr is connected to a terminal.
#[must_use]
pub fn is_stderr_terminal() -> bool {
    std::io::stderr().is_terminal()
}

/// Detect the color system supported by the terminal.
///
/// Checks environment variables to determine color capabilities:
/// - `NO_COLOR`: Disables colors
/// - `COLORTERM=truecolor` or `24bit`: 24-bit color
/// - `TERM` containing `256color`: 256 colors
/// - `TERM=dumb`: No colors
/// - Otherwise: Standard 16 colors (if terminal)
#[must_use]
pub fn detect_color_system() -> Option<ColorSystem> {
    detect_color_system_with(&read_env_settings(), is_terminal())
}

fn detect_color_system_with(env: &EnvSettings, #[allow(unused_variables)] is_tty: bool) -> Option<ColorSystem> {
    // Check NO_COLOR env var (https://no-color.org/)
    if env.no_color.is_some() {
        return None;
    }

    // Check FORCE_COLOR env var
    if let Some(level) = env.force_color.as_deref() {
        // A value of "0" disables colors entirely.
        if level == "0" {
            return None;
        }

        // Force colors, check for level
        return match level {
            "3" => Some(ColorSystem::TrueColor),
            "2" => Some(ColorSystem::EightBit),
            "1" | "" => Some(ColorSystem::Standard),
            _ => Some(ColorSystem::TrueColor),
        };
    }

    // Check COLORTERM for truecolor
    if let Some(colorterm) = env.colorterm.as_ref() {
        let colorterm = colorterm.to_lowercase();
        if colorterm == "truecolor" || colorterm == "24bit" {
            return Some(ColorSystem::TrueColor);
        }
    }

    // Check TERM for color support
    if let Some(term) = env.term.as_ref() {
        let term = term.to_lowercase();
        if term == "dumb" {
            return None;
        }
        if term.contains("256color") || term.contains("256") {
            return Some(ColorSystem::EightBit);
        }
        if term.contains("color") || term.contains("xterm") || term.contains("vt100") {
            return Some(ColorSystem::Standard);
        }
    }

    // Check for Windows legacy console
    #[cfg(windows)]
    {
        if env.wt_session.is_some() {
            // Windows Terminal supports true color
            return Some(ColorSystem::TrueColor);
        }
        // Windows console supports true color via VT sequences
        return Some(ColorSystem::TrueColor);
    }

    // Default to standard colors if we're on a terminal
    #[cfg(not(windows))]
    if is_tty {
        Some(ColorSystem::Standard)
    } else {
        None
    }
}

/// Enable raw terminal mode (for advanced input handling).
pub fn enable_raw_mode() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()
}

/// Disable raw terminal mode.
pub fn disable_raw_mode() -> std::io::Result<()> {
    crossterm::terminal::disable_raw_mode()
}

/// Terminal control sequences.
pub mod control {
    use std::io::Write;

    /// Clear the entire screen.
    pub fn clear_screen<W: Write>(writer: &mut W) -> std::io::Result<()> {
        use crossterm::{
            ExecutableCommand,
            terminal::{Clear, ClearType},
        };
        writer.execute(Clear(ClearType::All))?;
        Ok(())
    }

    /// Clear the current line.
    pub fn clear_line<W: Write>(writer: &mut W) -> std::io::Result<()> {
        use crossterm::{
            ExecutableCommand,
            terminal::{Clear, ClearType},
        };
        writer.execute(Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    /// Move cursor to home position (0, 0).
    pub fn cursor_home<W: Write>(writer: &mut W) -> std::io::Result<()> {
        use crossterm::{ExecutableCommand, cursor::MoveTo};
        writer.execute(MoveTo(0, 0))?;
        Ok(())
    }

    /// Move cursor to a specific position.
    pub fn cursor_move_to<W: Write>(writer: &mut W, x: u16, y: u16) -> std::io::Result<()> {
        use crossterm::{ExecutableCommand, cursor::MoveTo};
        writer.execute(MoveTo(x, y))?;
        Ok(())
    }

    /// Move cursor up by `n` lines.
    pub fn cursor_up<W: Write>(writer: &mut W, n: u16) -> std::io::Result<()> {
        use crossterm::{ExecutableCommand, cursor::MoveUp};
        writer.execute(MoveUp(n))?;
        Ok(())
    }

    /// Move cursor down by `n` lines.
    pub fn cursor_down<W: Write>(writer: &mut W, n: u16) -> std::io::Result<()> {
        use crossterm::{ExecutableCommand, cursor::MoveDown};
        writer.execute(MoveDown(n))?;
        Ok(())
    }

    /// Move cursor forward (right) by `n` columns.
    pub fn cursor_forward<W: Write>(writer: &mut W, n: u16) -> std::io::Result<()> {
        use crossterm::{ExecutableCommand, cursor::MoveRight};
        writer.execute(MoveRight(n))?;
        Ok(())
    }

    /// Move cursor backward (left) by `n` columns.
    pub fn cursor_backward<W: Write>(writer: &mut W, n: u16) -> std::io::Result<()> {
        use crossterm::{ExecutableCommand, cursor::MoveLeft};
        writer.execute(MoveLeft(n))?;
        Ok(())
    }

    /// Hide the cursor.
    pub fn hide_cursor<W: Write>(writer: &mut W) -> std::io::Result<()> {
        use crossterm::{ExecutableCommand, cursor::Hide};
        writer.execute(Hide)?;
        Ok(())
    }

    /// Show the cursor.
    pub fn show_cursor<W: Write>(writer: &mut W) -> std::io::Result<()> {
        use crossterm::{ExecutableCommand, cursor::Show};
        writer.execute(Show)?;
        Ok(())
    }

    /// Enable alternate screen buffer.
    pub fn enable_alt_screen<W: Write>(writer: &mut W) -> std::io::Result<()> {
        use crossterm::{ExecutableCommand, terminal::EnterAlternateScreen};
        writer.execute(EnterAlternateScreen)?;
        Ok(())
    }

    /// Disable alternate screen buffer (return to main screen).
    pub fn disable_alt_screen<W: Write>(writer: &mut W) -> std::io::Result<()> {
        use crossterm::{ExecutableCommand, terminal::LeaveAlternateScreen};
        writer.execute(LeaveAlternateScreen)?;
        Ok(())
    }

    /// Set the terminal window title.
    pub fn set_title<W: Write>(writer: &mut W, title: &str) -> std::io::Result<()> {
        use crossterm::{ExecutableCommand, terminal::SetTitle};
        writer.execute(SetTitle(title))?;
        Ok(())
    }

    /// Ring the terminal bell.
    pub fn bell<W: Write>(writer: &mut W) -> std::io::Result<()> {
        write!(writer, "\x07")?;
        writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create `EnvSettings` for testing
    fn make_env(
        no_color: Option<&str>,
        force_color: Option<&str>,
        colorterm: Option<&str>,
        term: Option<&str>,
    ) -> EnvSettings {
        EnvSettings {
            no_color: no_color.map(String::from),
            force_color: force_color.map(String::from),
            colorterm: colorterm.map(String::from),
            term: term.map(String::from),
            #[cfg(windows)]
            wt_session: None,
        }
    }

    #[test]
    fn test_detect_color_system() {
        // Just ensure it doesn't panic
        let _ = detect_color_system();
    }

    #[test]
    fn test_force_color_zero_disables_colors() {
        let settings = make_env(None, Some("0"), None, Some("xterm-256color"));
        assert_eq!(detect_color_system_with(&settings, true), None);
    }

    #[test]
    fn test_is_terminal() {
        // Just ensure it runs (result depends on test environment)
        let _ = is_terminal();
    }

    #[test]
    fn test_is_stderr_terminal() {
        // Just ensure it runs (result depends on test environment)
        let _ = is_stderr_terminal();
    }

    #[test]
    fn test_get_terminal_size() {
        // May return None in test environment
        let _ = get_terminal_size();
    }

    #[test]
    fn test_get_terminal_width() {
        let width = get_terminal_width();
        assert!(width > 0);
    }

    #[test]
    fn test_get_terminal_height() {
        let height = get_terminal_height();
        assert!(height > 0);
    }

    // =========================================================================
    // NO_COLOR environment variable tests
    // =========================================================================

    #[test]
    fn test_no_color_disables_colors() {
        // NO_COLOR should disable colors regardless of other settings
        let settings = make_env(Some("1"), None, Some("truecolor"), Some("xterm-256color"));
        assert_eq!(detect_color_system_with(&settings, true), None);
    }

    #[test]
    fn test_no_color_empty_string_disables() {
        // Even empty NO_COLOR should disable colors (it's set)
        let settings = make_env(Some(""), None, Some("truecolor"), None);
        assert_eq!(detect_color_system_with(&settings, true), None);
    }

    #[test]
    fn test_no_color_takes_precedence_over_force_color() {
        // NO_COLOR should take precedence over FORCE_COLOR
        let settings = make_env(Some("1"), Some("3"), Some("truecolor"), None);
        assert_eq!(detect_color_system_with(&settings, true), None);
    }

    // =========================================================================
    // FORCE_COLOR environment variable tests
    // =========================================================================

    #[test]
    fn test_force_color_level_1() {
        let settings = make_env(None, Some("1"), None, None);
        assert_eq!(
            detect_color_system_with(&settings, false),
            Some(ColorSystem::Standard)
        );
    }

    #[test]
    fn test_force_color_level_2() {
        let settings = make_env(None, Some("2"), None, None);
        assert_eq!(
            detect_color_system_with(&settings, false),
            Some(ColorSystem::EightBit)
        );
    }

    #[test]
    fn test_force_color_level_3() {
        let settings = make_env(None, Some("3"), None, None);
        assert_eq!(
            detect_color_system_with(&settings, false),
            Some(ColorSystem::TrueColor)
        );
    }

    #[test]
    fn test_force_color_empty_string() {
        // Empty FORCE_COLOR should enable standard colors
        let settings = make_env(None, Some(""), None, None);
        assert_eq!(
            detect_color_system_with(&settings, false),
            Some(ColorSystem::Standard)
        );
    }

    #[test]
    fn test_force_color_unknown_value() {
        // Unknown FORCE_COLOR values default to TrueColor
        let settings = make_env(None, Some("yes"), None, None);
        assert_eq!(
            detect_color_system_with(&settings, false),
            Some(ColorSystem::TrueColor)
        );
    }

    #[test]
    fn test_force_color_overrides_term() {
        // FORCE_COLOR should override TERM detection
        let settings = make_env(None, Some("1"), None, Some("dumb"));
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::Standard)
        );
    }

    // =========================================================================
    // COLORTERM environment variable tests
    // =========================================================================

    #[test]
    fn test_colorterm_truecolor() {
        let settings = make_env(None, None, Some("truecolor"), None);
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::TrueColor)
        );
    }

    #[test]
    fn test_colorterm_24bit() {
        let settings = make_env(None, None, Some("24bit"), None);
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::TrueColor)
        );
    }

    #[test]
    fn test_colorterm_case_insensitive() {
        let settings = make_env(None, None, Some("TRUECOLOR"), None);
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::TrueColor)
        );
    }

    #[test]
    fn test_colorterm_unknown_value() {
        // Unknown COLORTERM should fall through to TERM
        let settings = make_env(None, None, Some("unknown"), Some("xterm-256color"));
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::EightBit)
        );
    }

    // =========================================================================
    // TERM environment variable tests
    // =========================================================================

    #[test]
    fn test_term_dumb() {
        let settings = make_env(None, None, None, Some("dumb"));
        assert_eq!(detect_color_system_with(&settings, true), None);
    }

    #[test]
    fn test_term_dumb_case_insensitive() {
        let settings = make_env(None, None, None, Some("DUMB"));
        assert_eq!(detect_color_system_with(&settings, true), None);
    }

    #[test]
    fn test_term_256color() {
        let settings = make_env(None, None, None, Some("xterm-256color"));
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::EightBit)
        );
    }

    #[test]
    fn test_term_256_variant() {
        let settings = make_env(None, None, None, Some("screen-256"));
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::EightBit)
        );
    }

    #[test]
    fn test_term_xterm() {
        let settings = make_env(None, None, None, Some("xterm"));
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::Standard)
        );
    }

    #[test]
    fn test_term_xterm_color() {
        let settings = make_env(None, None, None, Some("xterm-color"));
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::Standard)
        );
    }

    #[test]
    fn test_term_vt100() {
        let settings = make_env(None, None, None, Some("vt100"));
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::Standard)
        );
    }

    #[test]
    #[allow(unused_variables)]
    fn test_term_linux() {
        // "linux" doesn't contain known keywords, falls through to TTY check
        let settings = make_env(None, None, None, Some("linux"));
        #[cfg(not(windows))]
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::Standard)
        );
    }

    // =========================================================================
    // TTY fallback tests
    // =========================================================================

    #[test]
    #[allow(unused_variables)]
    fn test_no_env_vars_tty_true() {
        let settings = make_env(None, None, None, None);
        #[cfg(not(windows))]
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::Standard)
        );
    }

    #[test]
    #[allow(unused_variables)]
    fn test_no_env_vars_tty_false() {
        let settings = make_env(None, None, None, None);
        #[cfg(not(windows))]
        assert_eq!(detect_color_system_with(&settings, false), None);
    }

    // =========================================================================
    // Windows-specific tests
    // =========================================================================

    #[cfg(windows)]
    #[test]
    fn test_windows_terminal_detected() {
        let settings = EnvSettings {
            no_color: None,
            force_color: None,
            colorterm: None,
            term: None,
            wt_session: Some("1".to_string()),
        };
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::TrueColor)
        );
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_default_truecolor() {
        // Windows without WT_SESSION still defaults to TrueColor
        let settings = EnvSettings {
            no_color: None,
            force_color: None,
            colorterm: None,
            term: None,
            wt_session: None,
        };
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::TrueColor)
        );
    }

    // =========================================================================
    // Edge cases and combinations
    // =========================================================================

    #[test]
    fn test_colorterm_takes_precedence_over_term() {
        // COLORTERM=truecolor should override TERM detection
        let settings = make_env(None, None, Some("truecolor"), Some("xterm"));
        assert_eq!(
            detect_color_system_with(&settings, true),
            Some(ColorSystem::TrueColor)
        );
    }

    #[test]
    fn test_all_env_vars_empty() {
        let settings = make_env(None, None, None, None);
        // Result depends on TTY state and platform
        let _ = detect_color_system_with(&settings, true);
        let _ = detect_color_system_with(&settings, false);
    }
}
