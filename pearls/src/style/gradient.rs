//! Gradient support for backgrounds and text
//!
//! Provides linear and radial gradients for enhanced visual effects.

use super::{Color, ColorProfile};
use ratatui::style::Color as RatatuiColor;

/// Type of gradient
#[derive(Clone, Debug, PartialEq)]
pub enum GradientType {
    /// Linear gradient from one point to another
    Linear(LinearDirection),
    /// Radial gradient from center outward
    Radial,
    /// Diagonal gradient
    Diagonal(DiagonalDirection),
}

/// Direction for linear gradients
#[derive(Clone, Debug, PartialEq)]
pub enum LinearDirection {
    /// From left to right
    Horizontal,
    /// From top to bottom
    Vertical,
}

/// Direction for diagonal gradients
#[derive(Clone, Debug, PartialEq)]
pub enum DiagonalDirection {
    /// From top-left to bottom-right
    TopLeftToBottomRight,
    /// From top-right to bottom-left
    TopRightToBottomLeft,
}

/// A color gradient
#[derive(Clone, Debug)]
pub struct Gradient {
    /// Start color
    start_color: Color,
    /// End color
    end_color: Color,
    /// Optional middle color for 3-point gradients
    middle_color: Option<Color>,
    /// Type of gradient
    gradient_type: GradientType,
    /// Number of steps in the gradient
    steps: usize,
}

impl Gradient {
    /// Create a new gradient
    pub fn new(start: Color, end: Color) -> Self {
        Self {
            start_color: start,
            end_color: end,
            middle_color: None,
            gradient_type: GradientType::Linear(LinearDirection::Horizontal),
            steps: 10,
        }
    }

    /// Create a linear gradient
    pub fn linear(start: Color, end: Color, direction: LinearDirection) -> Self {
        Self {
            start_color: start,
            end_color: end,
            middle_color: None,
            gradient_type: GradientType::Linear(direction),
            steps: 10,
        }
    }

    /// Create a radial gradient
    pub fn radial(center: Color, edge: Color) -> Self {
        Self {
            start_color: center,
            end_color: edge,
            middle_color: None,
            gradient_type: GradientType::Radial,
            steps: 10,
        }
    }

    /// Add a middle color for 3-point gradient
    pub fn with_middle(mut self, color: Color) -> Self {
        self.middle_color = Some(color);
        self
    }

    /// Set the number of steps
    pub fn with_steps(mut self, steps: usize) -> Self {
        self.steps = steps.max(2);
        self
    }

    /// Set the gradient type
    pub fn with_type(mut self, gradient_type: GradientType) -> Self {
        self.gradient_type = gradient_type;
        self
    }

    /// Generate gradient colors
    pub fn generate_colors(&self, profile: &ColorProfile) -> Vec<RatatuiColor> {
        if let Some(ref middle) = self.middle_color {
            // 3-point gradient
            let half_steps = self.steps / 2;
            let mut colors = Vec::with_capacity(self.steps);

            // First half: start to middle
            for i in 0..half_steps {
                let t = i as f32 / half_steps as f32;
                colors.push(self.interpolate_colors(&self.start_color, middle, t, profile));
            }

            // Second half: middle to end
            for i in 0..=(self.steps - half_steps) {
                let t = i as f32 / (self.steps - half_steps) as f32;
                colors.push(self.interpolate_colors(middle, &self.end_color, t, profile));
            }

            colors
        } else {
            // 2-point gradient
            (0..self.steps)
                .map(|i| {
                    let t = i as f32 / (self.steps - 1) as f32;
                    self.interpolate_colors(&self.start_color, &self.end_color, t, profile)
                })
                .collect()
        }
    }

    /// Interpolate between two colors
    fn interpolate_colors(
        &self,
        start: &Color,
        end: &Color,
        t: f32,
        profile: &ColorProfile,
    ) -> RatatuiColor {
        // Convert colors to RGB for interpolation
        let start_rgb = self.color_to_rgb(start, profile);
        let end_rgb = self.color_to_rgb(end, profile);

        let r = (start_rgb.0 as f32 * (1.0 - t) + end_rgb.0 as f32 * t) as u8;
        let g = (start_rgb.1 as f32 * (1.0 - t) + end_rgb.1 as f32 * t) as u8;
        let b = (start_rgb.2 as f32 * (1.0 - t) + end_rgb.2 as f32 * t) as u8;

        RatatuiColor::Rgb(r, g, b)
    }

    /// Convert a Color to RGB
    fn color_to_rgb(&self, color: &Color, profile: &ColorProfile) -> (u8, u8, u8) {
        // Convert to Ratatui color first, then extract RGB
        match color.to_ratatui(profile) {
            RatatuiColor::Rgb(r, g, b) => (r, g, b),
            RatatuiColor::Black => (0, 0, 0),
            RatatuiColor::Red => (255, 0, 0),
            RatatuiColor::Green => (0, 255, 0),
            RatatuiColor::Yellow => (255, 255, 0),
            RatatuiColor::Blue => (0, 0, 255),
            RatatuiColor::Magenta => (255, 0, 255),
            RatatuiColor::Cyan => (0, 255, 255),
            RatatuiColor::Gray => (128, 128, 128),
            RatatuiColor::DarkGray => (64, 64, 64),
            RatatuiColor::LightRed => (255, 128, 128),
            RatatuiColor::LightGreen => (128, 255, 128),
            RatatuiColor::LightYellow => (255, 255, 128),
            RatatuiColor::LightBlue => (128, 128, 255),
            RatatuiColor::LightMagenta => (255, 128, 255),
            RatatuiColor::LightCyan => (128, 255, 255),
            RatatuiColor::White => (255, 255, 255),
            _ => (128, 128, 128), // Default gray for indexed colors
        }
    }

    /// Generate a gradient for a specific area size
    pub fn generate_for_area(
        &self,
        width: u16,
        height: u16,
        profile: &ColorProfile,
    ) -> Vec<Vec<RatatuiColor>> {
        match self.gradient_type {
            GradientType::Linear(LinearDirection::Horizontal) => {
                let colors = self.generate_colors(profile);
                let mut grid = Vec::new();
                for _ in 0..height {
                    let mut row = Vec::new();
                    for x in 0..width {
                        let index = (x as usize * colors.len()) / width as usize;
                        row.push(colors[index.min(colors.len() - 1)]);
                    }
                    grid.push(row);
                }
                grid
            }
            GradientType::Linear(LinearDirection::Vertical) => {
                let colors = self.generate_colors(profile);
                let mut grid = Vec::new();
                for y in 0..height {
                    let index = (y as usize * colors.len()) / height as usize;
                    let color = colors[index.min(colors.len() - 1)];
                    grid.push(vec![color; width as usize]);
                }
                grid
            }
            GradientType::Diagonal(DiagonalDirection::TopLeftToBottomRight) => {
                let colors = self.generate_colors(profile);
                let mut grid = Vec::new();
                let diagonal_len = (width + height) as usize;

                for y in 0..height {
                    let mut row = Vec::new();
                    for x in 0..width {
                        let progress = (x + y) as usize;
                        let index = (progress * colors.len()) / diagonal_len;
                        row.push(colors[index.min(colors.len() - 1)]);
                    }
                    grid.push(row);
                }
                grid
            }
            GradientType::Diagonal(DiagonalDirection::TopRightToBottomLeft) => {
                let colors = self.generate_colors(profile);
                let mut grid = Vec::new();
                let diagonal_len = (width + height) as usize;

                for y in 0..height {
                    let mut row = Vec::new();
                    for x in 0..width {
                        let progress = ((width - x - 1) + y) as usize;
                        let index = (progress * colors.len()) / diagonal_len;
                        row.push(colors[index.min(colors.len() - 1)]);
                    }
                    grid.push(row);
                }
                grid
            }
            GradientType::Radial => {
                let colors = self.generate_colors(profile);
                let mut grid = Vec::new();
                let center_x = width as f32 / 2.0;
                let center_y = height as f32 / 2.0;
                let max_radius = (center_x * center_x + center_y * center_y).sqrt();

                for y in 0..height {
                    let mut row = Vec::new();
                    for x in 0..width {
                        let dx = x as f32 - center_x;
                        let dy = y as f32 - center_y;
                        let radius = (dx * dx + dy * dy).sqrt();
                        let index = ((radius / max_radius) * colors.len() as f32) as usize;
                        row.push(colors[index.min(colors.len() - 1)]);
                    }
                    grid.push(row);
                }
                grid
            }
        }
    }
}

/// Preset gradients
impl Gradient {
    /// Sunset gradient (orange to purple)
    pub fn sunset() -> Self {
        Self::linear(
            Color::rgb(255, 94, 77),
            Color::rgb(255, 154, 139),
            LinearDirection::Horizontal,
        )
        .with_middle(Color::rgb(255, 206, 166))
    }

    /// Ocean gradient (blue to teal)
    pub fn ocean() -> Self {
        Self::linear(
            Color::rgb(0, 119, 190),
            Color::rgb(0, 180, 216),
            LinearDirection::Vertical,
        )
    }

    /// Forest gradient (dark green to light green)
    pub fn forest() -> Self {
        Self::linear(
            Color::rgb(34, 139, 34),
            Color::rgb(144, 238, 144),
            LinearDirection::Vertical,
        )
    }

    /// Fire gradient (red to yellow)
    pub fn fire() -> Self {
        Self::linear(
            Color::rgb(255, 0, 0),
            Color::rgb(255, 255, 0),
            LinearDirection::Vertical,
        )
        .with_middle(Color::rgb(255, 140, 0))
    }

    /// Night sky gradient (dark blue to black)
    pub fn night_sky() -> Self {
        Self::radial(Color::rgb(25, 25, 112), Color::rgb(0, 0, 0))
    }

    /// Rainbow gradient
    pub fn rainbow() -> Self {
        Self::linear(
            Color::rgb(255, 0, 0),
            Color::rgb(148, 0, 211),
            LinearDirection::Horizontal,
        )
        .with_steps(7)
    }
}

/// Helper to render a gradient as background
pub fn render_gradient_background(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    gradient: &Gradient,
    profile: &ColorProfile,
) {
    use ratatui::widgets::Block;

    let colors = gradient.generate_for_area(area.width, area.height, profile);

    for (y, row) in colors.iter().enumerate() {
        for (x, color) in row.iter().enumerate() {
            let cell_area = ratatui::layout::Rect {
                x: area.x + x as u16,
                y: area.y + y as u16,
                width: 1,
                height: 1,
            };

            let block = Block::default().style(ratatui::style::Style::default().bg(*color));
            frame.render_widget(block, cell_area);
        }
    }
}
