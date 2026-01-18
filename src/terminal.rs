//! Terminal detection and manipulation.
//!
//! This module provides functionality to detect terminal capabilities
//! and query terminal dimensions.

use std::io::IsTerminal;

use crate::color::ColorSystem;

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
    get_terminal_size().map(|(w, _)| w).unwrap_or(80)
}

/// Get the terminal height in rows.
///
/// Returns a default of 24 if the height cannot be determined.
#[must_use]
pub fn get_terminal_height() -> usize {
    get_terminal_size().map(|(_, h)| h).unwrap_or(24)
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
    // Check NO_COLOR env var (https://no-color.org/)
    if std::env::var("NO_COLOR").is_ok() {
        return None;
    }

    // Check FORCE_COLOR env var
    if std::env::var("FORCE_COLOR").is_ok() {
        // Force colors, check for level
        if let Ok(level) = std::env::var("FORCE_COLOR") {
            match level.as_str() {
                "3" => return Some(ColorSystem::TrueColor),
                "2" => return Some(ColorSystem::EightBit),
                "1" | "" => return Some(ColorSystem::Standard),
                _ => {}
            }
        }
        return Some(ColorSystem::TrueColor);
    }

    // Check COLORTERM for truecolor
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        let colorterm = colorterm.to_lowercase();
        if colorterm == "truecolor" || colorterm == "24bit" {
            return Some(ColorSystem::TrueColor);
        }
    }

    // Check TERM for color support
    if let Ok(term) = std::env::var("TERM") {
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
        if std::env::var("WT_SESSION").is_ok() {
            // Windows Terminal supports true color
            return Some(ColorSystem::TrueColor);
        }
        // Windows console supports true color via VT sequences
        return Some(ColorSystem::TrueColor);
    }

    // Default to standard colors if we're on a terminal
    #[cfg(not(windows))]
    if is_terminal() {
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

    #[test]
    fn test_detect_color_system() {
        // Just ensure it doesn't panic
        let _ = detect_color_system();
    }

    #[test]
    fn test_is_terminal() {
        // Just ensure it runs (result depends on test environment)
        let _ = is_terminal();
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
}
