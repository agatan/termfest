//! This module provides rendering attributes like color or bold.

use std::default::Default;

/// `Attribute` is a rendering attribute that contains fg color, bg color and text effect.
///
/// ```
/// use termfest::attr::{Attribute, Color, BOLD};
///
/// Attribute { fg: Color::Red, effect: BOLD, ..Attribute::default() };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attribute {
    pub fg: Color,
    pub bg: Color,
    pub effect: Effect,
}

impl Default for Attribute {
    fn default() -> Attribute {
        Attribute {
            fg: Color::default(),
            bg: Color::default(),
            effect: Effect::empty(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    /// Specify Colors with 0 ~ 255 index
    EightBit(u8),
    /// Specify colors with RGB (true color)
    Rgb(u8, u8, u8),
}

impl Default for Color {
    fn default() -> Color {
        Color::Default
    }
}

bitflags! {
    pub struct Effect: u8 {
        const BOLD = 0b00000001;
        const DIM = 0b00000010;
        const UNDERLINE = 0b00000100;
        const BLINK = 0b00001000;
        const REVERSE = 0b00010000;
    }
}
