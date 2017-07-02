use std::default::Default;

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
