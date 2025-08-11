//! Theme system for consistent styling
//!
//! Provides centralized color palettes and style definitions.

use super::{Color, Style};
use std::collections::HashMap;

/// A color palette for theming
#[derive(Debug, Clone)]
pub struct ColorPalette {
    /// Primary color for main actions and highlights
    pub primary: Color,
    /// Secondary color for accents
    pub secondary: Color,
    /// Tertiary color for additional accents
    pub tertiary: Color,
    /// Success state color
    pub success: Color,
    /// Warning state color
    pub warning: Color,
    /// Error state color
    pub error: Color,
    /// Information state color
    pub info: Color,
    /// Main background color
    pub background: Color,
    /// Surface/card background color
    pub surface: Color,
    /// Primary text color
    pub text: Color,
    /// Secondary text color for less important content
    pub text_secondary: Color,
    /// Border color for separators and boundaries
    pub border: Color,
}

impl ColorPalette {
    /// Create the Nord theme palette
    pub fn nord() -> Self {
        Self {
            primary: Color::hex("#88C0D0"),        // Nord 8 - Frost cyan
            secondary: Color::hex("#81A1C1"),      // Nord 9 - Frost blue
            tertiary: Color::hex("#5E81AC"),       // Nord 10 - Frost dark blue
            success: Color::hex("#A3BE8C"),        // Nord 14 - Aurora green
            warning: Color::hex("#EBCB8B"),        // Nord 13 - Aurora yellow
            error: Color::hex("#BF616A"),          // Nord 11 - Aurora red
            info: Color::hex("#B48EAD"),           // Nord 15 - Aurora purple
            background: Color::hex("#2E3440"),     // Nord 0 - Polar night
            surface: Color::hex("#3B4252"),        // Nord 1 - Polar night
            text: Color::hex("#ECEFF4"),           // Nord 6 - Snow storm
            text_secondary: Color::hex("#D8DEE9"), // Nord 4 - Snow storm
            border: Color::hex("#4C566A"),         // Nord 3 - Polar night
        }
    }

    /// Create the Dracula theme palette
    pub fn dracula() -> Self {
        Self {
            primary: Color::hex("#BD93F9"),        // Purple
            secondary: Color::hex("#FF79C6"),      // Pink
            tertiary: Color::hex("#8BE9FD"),       // Cyan
            success: Color::hex("#50FA7B"),        // Green
            warning: Color::hex("#F1FA8C"),        // Yellow
            error: Color::hex("#FF5555"),          // Red
            info: Color::hex("#8BE9FD"),           // Cyan
            background: Color::hex("#282A36"),     // Background
            surface: Color::hex("#44475A"),        // Current line
            text: Color::hex("#F8F8F2"),           // Foreground
            text_secondary: Color::hex("#6272A4"), // Comment
            border: Color::hex("#44475A"),         // Current line
        }
    }

    /// Create the Solarized Dark theme palette
    pub fn solarized_dark() -> Self {
        Self {
            primary: Color::hex("#268BD2"),        // Blue
            secondary: Color::hex("#2AA198"),      // Cyan
            tertiary: Color::hex("#859900"),       // Green
            success: Color::hex("#859900"),        // Green
            warning: Color::hex("#B58900"),        // Yellow
            error: Color::hex("#DC322F"),          // Red
            info: Color::hex("#6C71C4"),           // Violet
            background: Color::hex("#002B36"),     // Base03
            surface: Color::hex("#073642"),        // Base02
            text: Color::hex("#839496"),           // Base0
            text_secondary: Color::hex("#586E75"), // Base01
            border: Color::hex("#073642"),         // Base02
        }
    }

    /// Create the Solarized Light theme palette
    pub fn solarized_light() -> Self {
        Self {
            primary: Color::hex("#268BD2"),        // Blue
            secondary: Color::hex("#2AA198"),      // Cyan
            tertiary: Color::hex("#859900"),       // Green
            success: Color::hex("#859900"),        // Green
            warning: Color::hex("#B58900"),        // Yellow
            error: Color::hex("#DC322F"),          // Red
            info: Color::hex("#6C71C4"),           // Violet
            background: Color::hex("#FDF6E3"),     // Base3
            surface: Color::hex("#EEE8D5"),        // Base2
            text: Color::hex("#657B83"),           // Base00
            text_secondary: Color::hex("#93A1A1"), // Base1
            border: Color::hex("#EEE8D5"),         // Base2
        }
    }

    /// Create the Tokyo Night theme palette
    pub fn tokyo_night() -> Self {
        Self {
            primary: Color::hex("#7AA2F7"),        // Blue
            secondary: Color::hex("#BB9AF7"),      // Purple
            tertiary: Color::hex("#7DCFFF"),       // Cyan
            success: Color::hex("#9ECE6A"),        // Green
            warning: Color::hex("#E0AF68"),        // Yellow
            error: Color::hex("#F7768E"),          // Red
            info: Color::hex("#7DCFFF"),           // Cyan
            background: Color::hex("#1A1B26"),     // Background
            surface: Color::hex("#24283B"),        // Background highlight
            text: Color::hex("#C0CAF5"),           // Foreground
            text_secondary: Color::hex("#9AA5CE"), // Foreground dark
            border: Color::hex("#414868"),         // Terminal black
        }
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::nord()
    }
}

/// A complete theme including colors and styles
#[derive(Debug, Clone)]
pub struct Theme {
    pub colors: ColorPalette,
    pub styles: HashMap<String, Style>,
}

impl Theme {
    /// Create a new theme with a color palette
    pub fn new(colors: ColorPalette) -> Self {
        let mut styles = HashMap::new();

        // Define default component styles
        styles.insert(
            "header".to_string(),
            Style::new()
                .fg(colors.text.clone())
                .bold()
                .padding_symmetric(1, 2),
        );

        styles.insert(
            "body".to_string(),
            Style::new()
                .fg(colors.text.clone())
                .bg(colors.background.clone()),
        );

        styles.insert(
            "footer".to_string(),
            Style::new()
                .fg(colors.text_secondary.clone())
                .padding_symmetric(1, 2),
        );

        styles.insert(
            "button".to_string(),
            Style::new()
                .fg(colors.background.clone())
                .bg(colors.primary.clone())
                .padding_symmetric(0, 2)
                .border(super::BorderStyle::Rounded),
        );

        styles.insert(
            "button.active".to_string(),
            Style::new()
                .fg(colors.background.clone())
                .bg(colors.secondary.clone())
                .bold()
                .padding_symmetric(0, 2)
                .border(super::BorderStyle::Rounded),
        );

        styles.insert(
            "input".to_string(),
            Style::new()
                .fg(colors.text.clone())
                .bg(colors.surface.clone())
                .padding_symmetric(0, 1)
                .border(super::BorderStyle::Normal)
                .border_color(colors.border.clone()),
        );

        styles.insert(
            "input.focused".to_string(),
            Style::new()
                .fg(colors.text.clone())
                .bg(colors.surface.clone())
                .padding_symmetric(0, 1)
                .border(super::BorderStyle::Normal)
                .border_color(colors.primary.clone()),
        );

        styles.insert(
            "list.item".to_string(),
            Style::new().fg(colors.text.clone()),
        );

        styles.insert(
            "list.selected".to_string(),
            Style::new()
                .fg(colors.background.clone())
                .bg(colors.primary.clone())
                .bold(),
        );

        styles.insert(
            "error".to_string(),
            Style::new().fg(colors.error.clone()).bold(),
        );

        styles.insert(
            "warning".to_string(),
            Style::new().fg(colors.warning.clone()),
        );

        styles.insert(
            "success".to_string(),
            Style::new().fg(colors.success.clone()),
        );

        styles.insert("info".to_string(), Style::new().fg(colors.info.clone()));

        Self { colors, styles }
    }

    /// Get a style by name
    pub fn get_style(&self, name: &str) -> Option<&Style> {
        self.styles.get(name)
    }

    /// Set a custom style
    pub fn set_style(&mut self, name: impl Into<String>, style: Style) {
        self.styles.insert(name.into(), style);
    }

    /// Create the Nord theme
    pub fn nord() -> Self {
        Self::new(ColorPalette::nord())
    }

    /// Create the Dracula theme
    pub fn dracula() -> Self {
        Self::new(ColorPalette::dracula())
    }

    /// Create the Solarized Dark theme
    pub fn solarized_dark() -> Self {
        Self::new(ColorPalette::solarized_dark())
    }

    /// Create the Solarized Light theme
    pub fn solarized_light() -> Self {
        Self::new(ColorPalette::solarized_light())
    }

    /// Create the Tokyo Night theme
    pub fn tokyo_night() -> Self {
        Self::new(ColorPalette::tokyo_night())
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::nord()
    }
}

/// Trait for components that can be themed
pub trait Themed {
    /// Apply a theme to this component
    fn apply_theme(&mut self, theme: &Theme);

    /// Get a themed style by name
    fn themed_style(&self, theme: &Theme, name: &str) -> Style {
        theme.get_style(name).cloned().unwrap_or_default()
    }
}
