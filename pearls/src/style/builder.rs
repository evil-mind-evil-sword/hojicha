//! Fluent style builder API
//!
//! Provides an ergonomic, chainable API for building styles.

use super::color::{Color, ColorProfile};
use ratatui::style::{Modifier, Style as RatatuiStyle};

/// Padding configuration for styled elements
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Padding {
    /// Top padding
    pub top: u16,
    /// Right padding
    pub right: u16,
    /// Bottom padding
    pub bottom: u16,
    /// Left padding
    pub left: u16,
}

impl Padding {
    /// Create padding with the same value on all sides
    pub fn all(value: u16) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create padding with vertical and horizontal values
    pub fn symmetric(vertical: u16, horizontal: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

/// Margin configuration for styled elements
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Margin {
    /// Top margin
    pub top: u16,
    /// Right margin
    pub right: u16,
    /// Bottom margin
    pub bottom: u16,
    /// Left margin
    pub left: u16,
}

impl Margin {
    /// Create margin with the same value on all sides
    pub fn all(value: u16) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create margin with vertical and horizontal values
    pub fn symmetric(vertical: u16, horizontal: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

/// Text alignment options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl TextAlign {
    /// Convert to Ratatui alignment
    pub fn to_ratatui(&self) -> ratatui::layout::Alignment {
        match self {
            Self::Left => ratatui::layout::Alignment::Left,
            Self::Center => ratatui::layout::Alignment::Center,
            Self::Right => ratatui::layout::Alignment::Right,
        }
    }
}

/// Border style configuration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BorderStyle {
    #[default]
    None,
    Normal,
    Rounded,
    Double,
    Thick,
}

impl BorderStyle {
    /// Convert to Ratatui border type
    pub fn to_ratatui(&self) -> ratatui::widgets::BorderType {
        match self {
            Self::None | Self::Normal => ratatui::widgets::BorderType::Plain,
            Self::Rounded => ratatui::widgets::BorderType::Rounded,
            Self::Double => ratatui::widgets::BorderType::Double,
            Self::Thick => ratatui::widgets::BorderType::Thick,
        }
    }
}

/// High-level style builder with fluent API
#[derive(Debug, Clone, Default)]
pub struct Style {
    foreground: Option<Color>,
    background: Option<Color>,
    modifiers: Modifier,
    padding: Padding,
    margin: Margin,
    border: BorderStyle,
    border_color: Option<Color>,
    width: Option<u16>,
    height: Option<u16>,
    max_width: Option<u16>,
    max_height: Option<u16>,
    text_align: TextAlign,
}

impl Style {
    /// Create a new style builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set foreground color
    pub fn fg(mut self, color: impl Into<Color>) -> Self {
        self.foreground = Some(color.into());
        self
    }

    /// Set background color
    pub fn bg(mut self, color: impl Into<Color>) -> Self {
        self.background = Some(color.into());
        self
    }

    /// Add bold modifier
    pub fn bold(mut self) -> Self {
        self.modifiers |= Modifier::BOLD;
        self
    }

    /// Add italic modifier
    pub fn italic(mut self) -> Self {
        self.modifiers |= Modifier::ITALIC;
        self
    }

    /// Add underline modifier
    pub fn underline(mut self) -> Self {
        self.modifiers |= Modifier::UNDERLINED;
        self
    }

    /// Add underlined modifier (alias for underline)
    pub fn underlined(mut self) -> Self {
        self.modifiers |= Modifier::UNDERLINED;
        self
    }

    /// Add dim modifier
    pub fn dim(mut self) -> Self {
        self.modifiers |= Modifier::DIM;
        self
    }

    /// Add strikethrough modifier
    pub fn strikethrough(mut self) -> Self {
        self.modifiers |= Modifier::CROSSED_OUT;
        self
    }

    /// Set padding on all sides
    pub fn padding(mut self, top: u16, right: u16, bottom: u16, left: u16) -> Self {
        self.padding = Padding {
            top,
            right,
            bottom,
            left,
        };
        self
    }

    /// Set padding with a Padding struct
    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Set uniform padding on all sides
    pub fn padding_all(mut self, value: u16) -> Self {
        self.padding = Padding::all(value);
        self
    }

    /// Set vertical and horizontal padding
    pub fn padding_symmetric(mut self, vertical: u16, horizontal: u16) -> Self {
        self.padding = Padding::symmetric(vertical, horizontal);
        self
    }

    /// Set margin on all sides
    pub fn margin(mut self, top: u16, right: u16, bottom: u16, left: u16) -> Self {
        self.margin = Margin {
            top,
            right,
            bottom,
            left,
        };
        self
    }

    /// Set margin with a Margin struct
    pub fn with_margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    /// Set uniform margin on all sides
    pub fn margin_all(mut self, value: u16) -> Self {
        self.margin = Margin::all(value);
        self
    }

    /// Set vertical and horizontal margin
    pub fn margin_symmetric(mut self, vertical: u16, horizontal: u16) -> Self {
        self.margin = Margin::symmetric(vertical, horizontal);
        self
    }

    /// Set border style
    pub fn border(mut self, style: BorderStyle) -> Self {
        self.border = style;
        self
    }

    /// Set border color
    pub fn border_color(mut self, color: impl Into<Color>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    /// Set fixed width
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set fixed height
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    /// Set maximum width
    pub fn max_width(mut self, width: u16) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set maximum height
    pub fn max_height(mut self, height: u16) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Set text alignment
    pub fn align(mut self, alignment: TextAlign) -> Self {
        self.text_align = alignment;
        self
    }

    /// Align text to the left
    pub fn align_left(mut self) -> Self {
        self.text_align = TextAlign::Left;
        self
    }

    /// Align text to the center
    pub fn align_center(mut self) -> Self {
        self.text_align = TextAlign::Center;
        self
    }

    /// Align text to the right
    pub fn align_right(mut self) -> Self {
        self.text_align = TextAlign::Right;
        self
    }

    /// Merge with another style (other style takes precedence)
    pub fn merge(mut self, other: &Style) -> Self {
        if other.foreground.is_some() {
            self.foreground = other.foreground.clone();
        }
        if other.background.is_some() {
            self.background = other.background.clone();
        }
        self.modifiers |= other.modifiers;
        if other.padding != Padding::default() {
            self.padding = other.padding;
        }
        if other.margin != Margin::default() {
            self.margin = other.margin;
        }
        if other.border != BorderStyle::None {
            self.border = other.border;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color.clone();
        }
        if other.width.is_some() {
            self.width = other.width;
        }
        if other.height.is_some() {
            self.height = other.height;
        }
        if other.max_width.is_some() {
            self.max_width = other.max_width;
        }
        if other.max_height.is_some() {
            self.max_height = other.max_height;
        }
        if other.text_align != TextAlign::Left {
            self.text_align = other.text_align;
        }
        self
    }

    /// Convert to Ratatui style (loses layout information)
    pub fn to_ratatui(&self, profile: &ColorProfile) -> RatatuiStyle {
        let mut style = RatatuiStyle::default();

        if let Some(ref fg) = self.foreground {
            style = style.fg(fg.to_ratatui(profile));
        }

        if let Some(ref bg) = self.background {
            style = style.bg(bg.to_ratatui(profile));
        }

        style = style.add_modifier(self.modifiers);

        style
    }

    /// Get padding
    pub fn get_padding(&self) -> &Padding {
        &self.padding
    }

    /// Get margin
    pub fn get_margin(&self) -> &Margin {
        &self.margin
    }

    /// Get border style
    pub fn get_border(&self) -> &BorderStyle {
        &self.border
    }

    /// Get border color
    pub fn get_border_color(&self) -> Option<&Color> {
        self.border_color.as_ref()
    }

    /// Get foreground color
    pub fn get_foreground(&self) -> Option<&Color> {
        self.foreground.as_ref()
    }

    /// Get background color
    pub fn get_background(&self) -> Option<&Color> {
        self.background.as_ref()
    }

    /// Get width constraint
    pub fn get_width(&self) -> Option<u16> {
        self.width
    }

    /// Get height constraint
    pub fn get_height(&self) -> Option<u16> {
        self.height
    }

    /// Get text alignment
    pub fn get_text_align(&self) -> TextAlign {
        self.text_align
    }
}

/// Builder for creating styles with method chaining
pub struct StyleBuilder {
    style: Style,
}

impl StyleBuilder {
    /// Create a new style builder
    pub fn new() -> Self {
        Self {
            style: Style::new(),
        }
    }

    /// Build the final style
    pub fn build(self) -> Style {
        self.style
    }
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Implement all the same methods as Style for StyleBuilder
impl std::ops::Deref for StyleBuilder {
    type Target = Style;

    fn deref(&self) -> &Self::Target {
        &self.style
    }
}

impl std::ops::DerefMut for StyleBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.style
    }
}
