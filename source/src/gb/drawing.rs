use super::gbdk_c::gb::drawing;

pub const SCREEN_WIDTH: u8 = drawing::GRAPHICS_WIDTH;
pub const SCREEN_HEIGHT: u8 = drawing::GRAPHICS_WIDTH;
pub const TILE_WIDTH: u8 = SCREEN_WIDTH / 8;
pub const TILE_HEIGHT: u8 = SCREEN_HEIGHT / 8;

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum DrawingMode {
    Solid = drawing::SOLID,
    Or = drawing::OR,
    Xor = drawing::XOR,
    And = drawing::AND
}

impl From<u8> for DrawingMode {
    fn from(value: u8) -> Self {
        match value {
            drawing::SOLID => Self::Solid,
            drawing::OR => Self::Or,
            drawing::XOR => Self::Xor,
            drawing::AND => Self::And,
            _ => panic!("DrawingMode from u8 outbounded\0")
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum DmgColor {
    White = drawing::WHITE,
    LightGrey = drawing::LTGREY,
    DarkGrey = drawing::DKGREY,
    Black = drawing::BLACK,
}

impl From<u8> for DmgColor {
    fn from(value: u8) -> Self {
        match value {
            drawing::WHITE => Self::White,
            drawing::LTGREY => Self::LightGrey,
            drawing::DKGREY => Self::DarkGrey,
            drawing::BLACK => Self::Black,
            _ => panic!("DmgColor from u8 outbounded\0")
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct DrawingStyle {
    pub foreground: DmgColor,
    pub background: DmgColor,
    pub drawing_mode: DrawingMode,
}

impl DrawingStyle {
    pub fn default() -> Self {
        DrawingStyle {
            foreground: DmgColor::Black, 
            background: DmgColor::White, 
            drawing_mode: DrawingMode::Solid
        }
    }

    pub fn reversed() -> Self {
        DrawingStyle {
            foreground: DmgColor::White, 
            background: DmgColor::Black, 
            drawing_mode: DrawingMode::Solid
        }
    }

    pub fn foreground(&mut self, color: DmgColor) -> &mut Self {
        self.foreground = color;
        self
    }

    pub fn background(&mut self, color: DmgColor) -> &mut Self {
        self.background = color;
        self
    }

    pub fn drawing_mode(&mut self, mode: DrawingMode) -> &mut Self {
        self.drawing_mode = mode;
        self
    }

    pub fn apply(&self) {
        unsafe {
            drawing::color(
                self.foreground as u8,
                self.background as u8,
                self.drawing_mode as u8
            )
        }
    }
}
