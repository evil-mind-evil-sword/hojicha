//! Adaptive color system for terminal themes
//!
//! Colors that automatically adapt based on terminal background (light/dark mode).

use ratatui::style::Color as RatatuiColor;

/// Terminal background mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundMode {
    Light,
    Dark,
}

/// Color profile for the current terminal
#[derive(Debug, Clone)]
pub struct ColorProfile {
    background_mode: BackgroundMode,
    supports_true_color: bool,
}

impl ColorProfile {
    /// Create a new color profile
    pub fn new(background_mode: BackgroundMode, supports_true_color: bool) -> Self {
        Self {
            background_mode,
            supports_true_color,
        }
    }

    /// Get the background mode
    pub fn background_mode(&self) -> BackgroundMode {
        self.background_mode
    }

    /// Check if terminal supports true color
    pub fn supports_true_color(&self) -> bool {
        self.supports_true_color
    }

    /// Detect color profile from environment (best effort)
    pub fn detect() -> Self {
        // Check for common environment variables
        let colorterm = std::env::var("COLORTERM").unwrap_or_default();
        let supports_true_color = colorterm.contains("truecolor") || colorterm.contains("24bit");
        
        // Try to detect dark/light mode (this is a heuristic)
        // In practice, this might need more sophisticated detection
        let background_mode = if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            if term_program.contains("Apple_Terminal") {
                // macOS Terminal.app defaults to light
                BackgroundMode::Light
            } else {
                // Most modern terminals default to dark
                BackgroundMode::Dark
            }
        } else {
            BackgroundMode::Dark
        };
        
        Self {
            background_mode,
            supports_true_color,
        }
    }
}

impl Default for ColorProfile {
    fn default() -> Self {
        Self {
            background_mode: BackgroundMode::Dark,
            supports_true_color: true,
        }
    }
}

/// A color that can adapt based on terminal background
#[derive(Debug, Clone)]
pub struct AdaptiveColor {
    light: RatatuiColor,
    dark: RatatuiColor,
}

impl AdaptiveColor {
    /// Create a new adaptive color
    pub fn new(light: RatatuiColor, dark: RatatuiColor) -> Self {
        Self { light, dark }
    }

    /// Resolve to a concrete color based on the profile
    pub fn resolve(&self, profile: &ColorProfile) -> RatatuiColor {
        match profile.background_mode() {
            BackgroundMode::Light => self.light,
            BackgroundMode::Dark => self.dark,
        }
    }
}

/// High-level color type that can be adaptive or fixed
#[derive(Debug, Clone)]
pub enum Color {
    /// Fixed color that doesn't change
    Fixed(RatatuiColor),
    /// Adaptive color that changes based on terminal background
    Adaptive(AdaptiveColor),
    /// Named semantic color (resolved through theme)
    Semantic(String),
}

impl Color {
    /// Create a fixed color from RGB values
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Fixed(RatatuiColor::Rgb(r, g, b))
    }

    /// Create a fixed color from a hex string
    pub fn hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Self::Fixed(RatatuiColor::White);
        }
        
        if let Ok(rgb) = u32::from_str_radix(hex, 16) {
            let r = ((rgb >> 16) & 0xff) as u8;
            let g = ((rgb >> 8) & 0xff) as u8;
            let b = (rgb & 0xff) as u8;
            Self::rgb(r, g, b)
        } else {
            Self::Fixed(RatatuiColor::White)
        }
    }

    /// Create an adaptive color
    pub fn adaptive() -> AdaptiveColorBuilder {
        AdaptiveColorBuilder::new()
    }

    /// Create a semantic color reference
    pub fn semantic(name: impl Into<String>) -> Self {
        Self::Semantic(name.into())
    }

    /// Convert to Ratatui color
    pub fn to_ratatui(&self, profile: &ColorProfile) -> RatatuiColor {
        match self {
            Self::Fixed(color) => *color,
            Self::Adaptive(adaptive) => adaptive.resolve(profile),
            Self::Semantic(_) => {
                // This would be resolved through a theme
                // For now, default to gray
                RatatuiColor::Gray
            }
        }
    }

    // Convenience constructors for common colors
    pub fn black() -> Self {
        Self::Fixed(RatatuiColor::Black)
    }

    pub fn red() -> Self {
        Self::Fixed(RatatuiColor::Red)
    }

    pub fn green() -> Self {
        Self::Fixed(RatatuiColor::Green)
    }

    pub fn yellow() -> Self {
        Self::Fixed(RatatuiColor::Yellow)
    }

    pub fn blue() -> Self {
        Self::Fixed(RatatuiColor::Blue)
    }

    pub fn magenta() -> Self {
        Self::Fixed(RatatuiColor::Magenta)
    }

    pub fn cyan() -> Self {
        Self::Fixed(RatatuiColor::Cyan)
    }

    pub fn white() -> Self {
        Self::Fixed(RatatuiColor::White)
    }

    pub fn gray() -> Self {
        Self::Fixed(RatatuiColor::Gray)
    }
}

/// Builder for adaptive colors
pub struct AdaptiveColorBuilder {
    light: Option<RatatuiColor>,
    dark: Option<RatatuiColor>,
}

impl AdaptiveColorBuilder {
    /// Create a new adaptive color builder
    pub fn new() -> Self {
        Self {
            light: None,
            dark: None,
        }
    }

    /// Set the color for light backgrounds
    pub fn light(mut self, color: impl Into<RatatuiColor>) -> Self {
        self.light = Some(color.into());
        self
    }

    /// Set the color for light backgrounds from hex
    pub fn light_hex(mut self, hex: &str) -> Self {
        if let Color::Fixed(color) = Color::hex(hex) {
            self.light = Some(color);
        }
        self
    }

    /// Set the color for dark backgrounds
    pub fn dark(mut self, color: impl Into<RatatuiColor>) -> Self {
        self.dark = Some(color.into());
        self
    }

    /// Set the color for dark backgrounds from hex
    pub fn dark_hex(mut self, hex: &str) -> Self {
        if let Color::Fixed(color) = Color::hex(hex) {
            self.dark = Some(color);
        }
        self
    }

    /// Build the adaptive color
    pub fn build(self) -> Color {
        Color::Adaptive(AdaptiveColor {
            light: self.light.unwrap_or(RatatuiColor::Black),
            dark: self.dark.unwrap_or(RatatuiColor::White),
        })
    }
}

impl Default for AdaptiveColorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Implement From traits for convenience
impl From<RatatuiColor> for Color {
    fn from(color: RatatuiColor) -> Self {
        Self::Fixed(color)
    }
}

impl From<AdaptiveColor> for Color {
    fn from(adaptive: AdaptiveColor) -> Self {
        Self::Adaptive(adaptive)
    }
}