//! Terminal management logic extracted from Program for testability

use crate::program::MouseMode;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};

/// Configuration for terminal setup
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    pub alt_screen: bool,
    pub mouse_mode: MouseMode,
    pub bracketed_paste: bool,
    pub focus_reporting: bool,
    pub headless: bool,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            alt_screen: true,
            mouse_mode: MouseMode::None,
            bracketed_paste: false,
            focus_reporting: false,
            headless: false,
        }
    }
}

/// Manages terminal setup, teardown, and state
pub struct TerminalManager {
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
    config: TerminalConfig,
    alt_screen_was_active: bool,
    is_released: bool,
}

impl TerminalManager {
    /// Create a new terminal manager
    pub fn new(config: TerminalConfig) -> io::Result<Self> {
        let terminal = if !config.headless {
            Some(Self::setup_terminal(&config)?)
        } else {
            None
        };

        Ok(Self {
            terminal,
            config,
            alt_screen_was_active: false,
            is_released: false,
        })
    }

    /// Set up the terminal with the given configuration
    fn setup_terminal(config: &TerminalConfig) -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
        let mut stdout = io::stdout();

        // Enable raw mode
        enable_raw_mode()?;

        // Enter alternate screen if requested
        if config.alt_screen {
            execute!(stdout, EnterAlternateScreen)?;
        }

        // Set up mouse mode
        match config.mouse_mode {
            MouseMode::CellMotion => {
                execute!(stdout, crossterm::event::EnableMouseCapture)?;
            }
            MouseMode::AllMotion => {
                execute!(
                    stdout,
                    crossterm::event::EnableMouseCapture,
                    crossterm::cursor::Show,
                    crossterm::cursor::Hide,
                )?;
            }
            MouseMode::None => {}
        }

        // Enable bracketed paste if requested
        if config.bracketed_paste {
            execute!(stdout, crossterm::event::EnableBracketedPaste)?;
        }

        // Enable focus reporting if requested
        if config.focus_reporting {
            execute!(stdout, crossterm::event::EnableFocusChange)?;
        }

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        Ok(terminal)
    }

    /// Get a reference to the terminal
    pub fn terminal(&self) -> Option<&Terminal<CrosstermBackend<Stdout>>> {
        self.terminal.as_ref()
    }

    /// Get a mutable reference to the terminal
    pub fn terminal_mut(&mut self) -> Option<&mut Terminal<CrosstermBackend<Stdout>>> {
        self.terminal.as_mut()
    }

    /// Release the terminal (for running external commands)
    pub fn release(&mut self) -> io::Result<()> {
        if self.is_released || self.config.headless {
            return Ok(());
        }

        // Store alt screen state
        self.alt_screen_was_active = self.config.alt_screen;

        // Show cursor before releasing
        if let Some(ref mut terminal) = self.terminal {
            terminal.show_cursor()?;
        }

        // Exit alt screen if active
        if self.config.alt_screen {
            execute!(io::stdout(), LeaveAlternateScreen)?;
        }

        // Disable raw mode
        disable_raw_mode()?;

        self.is_released = true;
        Ok(())
    }

    /// Restore the terminal after releasing it
    pub fn restore(&mut self) -> io::Result<()> {
        if !self.is_released || self.config.headless {
            return Ok(());
        }

        // Re-enable raw mode
        enable_raw_mode()?;

        // Restore alt screen if it was active
        if self.alt_screen_was_active {
            execute!(io::stdout(), EnterAlternateScreen)?;
        }

        // Hide cursor again
        if let Some(ref mut terminal) = self.terminal {
            terminal.hide_cursor()?;
            // Force a redraw
            terminal.clear()?;
        }

        self.is_released = false;
        Ok(())
    }

    /// Check if the terminal is currently released
    pub fn is_released(&self) -> bool {
        self.is_released
    }

    /// Clean up terminal state
    pub fn cleanup(&mut self) -> io::Result<()> {
        if self.config.headless {
            return Ok(());
        }

        // Show cursor
        if let Some(ref mut terminal) = self.terminal {
            let _ = terminal.show_cursor();
        }

        // Disable various terminal features
        let mut stdout = io::stdout();

        if self.config.focus_reporting {
            let _ = execute!(stdout, crossterm::event::DisableFocusChange);
        }

        if self.config.bracketed_paste {
            let _ = execute!(stdout, crossterm::event::DisableBracketedPaste);
        }

        if self.config.mouse_mode != MouseMode::None {
            let _ = execute!(stdout, crossterm::event::DisableMouseCapture);
        }

        if self.config.alt_screen && !self.is_released {
            let _ = execute!(stdout, LeaveAlternateScreen);
        }

        // Always try to disable raw mode
        let _ = disable_raw_mode();

        Ok(())
    }

    /// Draw a frame (wrapper for terminal.draw)
    pub fn draw<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        if let Some(ref mut terminal) = self.terminal {
            terminal.draw(f)?;
        }
        Ok(())
    }

    /// Get the current terminal size
    pub fn size(&self) -> io::Result<ratatui::layout::Rect> {
        if let Some(ref terminal) = self.terminal {
            let size = terminal.size()?;
            Ok(ratatui::layout::Rect::new(0, 0, size.width, size.height))
        } else {
            // Return a default size for headless mode
            Ok(ratatui::layout::Rect::new(0, 0, 80, 24))
        }
    }

    /// Clear the terminal
    pub fn clear(&mut self) -> io::Result<()> {
        if let Some(ref mut terminal) = self.terminal {
            terminal.clear()?;
        }
        Ok(())
    }
}

impl Drop for TerminalManager {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_config_default() {
        let config = TerminalConfig::default();
        assert!(config.alt_screen);
        assert_eq!(config.mouse_mode, MouseMode::None);
        assert!(!config.bracketed_paste);
        assert!(!config.focus_reporting);
        assert!(!config.headless);
    }

    #[test]
    fn test_terminal_manager_headless() {
        let config = TerminalConfig {
            headless: true,
            ..Default::default()
        };

        let manager = TerminalManager::new(config).unwrap();
        assert!(manager.terminal().is_none());
        assert!(!manager.is_released());
    }

    #[test]
    fn test_terminal_manager_release_restore() {
        let config = TerminalConfig {
            headless: true,
            ..Default::default()
        };

        let mut manager = TerminalManager::new(config).unwrap();

        // In headless mode, release should succeed but not change state
        assert!(manager.release().is_ok());
        assert!(!manager.is_released()); // Headless mode doesn't actually release

        // Restore should also succeed without changing state
        assert!(manager.restore().is_ok());
        assert!(!manager.is_released());
    }

    #[test]
    fn test_terminal_manager_size_headless() {
        let config = TerminalConfig {
            headless: true,
            ..Default::default()
        };

        let manager = TerminalManager::new(config).unwrap();
        let size = manager.size().unwrap();

        // Should return default size in headless mode
        assert_eq!(size.width, 80);
        assert_eq!(size.height, 24);
    }

    #[test]
    fn test_terminal_manager_clear_headless() {
        let config = TerminalConfig {
            headless: true,
            ..Default::default()
        };

        let mut manager = TerminalManager::new(config).unwrap();

        // Clear should not panic in headless mode
        assert!(manager.clear().is_ok());
    }

    #[test]
    fn test_terminal_manager_draw_headless() {
        let config = TerminalConfig {
            headless: true,
            ..Default::default()
        };

        let mut manager = TerminalManager::new(config).unwrap();

        // Draw should not panic in headless mode
        assert!(
            manager
                .draw(|_f| {
                    // Drawing logic would go here
                })
                .is_ok()
        );
    }

    #[test]
    fn test_terminal_manager_cleanup() {
        let config = TerminalConfig {
            headless: true,
            ..Default::default()
        };

        let mut manager = TerminalManager::new(config).unwrap();

        // Cleanup should not panic
        assert!(manager.cleanup().is_ok());
    }

    #[test]
    fn test_terminal_manager_drop() {
        let config = TerminalConfig {
            headless: true,
            ..Default::default()
        };

        {
            let _manager = TerminalManager::new(config).unwrap();
            // Manager should clean up when dropped
        }
        // Should not panic
    }

    #[test]
    fn test_terminal_config_variations() {
        let configs = vec![
            TerminalConfig {
                alt_screen: false,
                mouse_mode: MouseMode::CellMotion,
                bracketed_paste: true,
                focus_reporting: true,
                headless: true,
            },
            TerminalConfig {
                alt_screen: true,
                mouse_mode: MouseMode::AllMotion,
                bracketed_paste: false,
                focus_reporting: false,
                headless: true,
            },
        ];

        for config in configs {
            let manager = TerminalManager::new(config).unwrap();
            assert!(manager.terminal().is_none()); // All are headless
        }
    }
}
