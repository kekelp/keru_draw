pub use crate::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Color {
    /// Red component of the color
    pub r: f32,
    /// Green component of the color
    pub g: f32,
    /// Blue component of the color
    pub b: f32,
    /// Alpha component of the color
    pub a: f32,
}

impl Color {
    pub const RED:   Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE:  Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const GREY:  Color = Color { r: 0.2, g: 0.2, b: 0.2, a: 1.0 };
    
    pub const KERU_BLUE:   Color = Color { r: 0.31,g:  0.31, b: 1.0, a: 1.0 };
    pub const KERU_RED:    Color = Color { r: 1.0, g: 0.31, b: 0.31, a: 1.0 };
    pub const KERU_PINK:   Color = Color { r: 0.65, g: 0.31, b: 0.65, a: 1.0 };
    pub const KERU_GREEN:  Color = Color { r: 0.1, g: 1.0, b: 0.1, a: 1.0 };
    pub const KERU_BLACK: Color = Color { r: 0.07, g: 0.07, b: 0.09, a: 1.0 };
    
    pub const DEBUG_RED:   Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 0.3 };
    pub const DEBUG_BLUE:  Color = Color { r: 0.12, g: 0.0, b: 1.0, a: 0.48 };
    
    pub const TRANSPARENT: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.0 };
    pub const GREENSCREEN: Color = Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };


    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from u8 RGBA values.
    pub const fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Apply alpha to a color.
    pub const fn with_alpha(color: Color, alpha: f32) -> Color {
        Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: alpha,
        }
    }

    pub const KERU_GRAD: ColorFill = ColorFill::Gradient(Gradient {
        color_start: Self::KERU_BLUE,
        color_end: Self::KERU_RED,
        gradient_type: GradientType::Linear,
        angle: -0.785398, // -45 degrees
    });

    pub const KERU_GRAD_FW: ColorFill = ColorFill::Gradient(Gradient {
        color_start: Self::KERU_BLUE,
        color_end: Self::KERU_RED,
        gradient_type: GradientType::Linear,
        angle: 0.785398, // 45 degrees
    });
}

/// Fill style for shapes - solid color or gradient
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorFill {
    Color(Color),
    Gradient(Gradient),
}

impl Hash for ColorFill {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ColorFill::Color(color) => {
                0u8.hash(state);
                color.r.to_bits().hash(state);
                color.g.to_bits().hash(state);
                color.b.to_bits().hash(state);
                color.a.to_bits().hash(state);
            },
            ColorFill::Gradient(gradient) => {
                1u8.hash(state);
                gradient.hash(state);
            },
        }
    }
}