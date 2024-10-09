use core::ffi::CStr;

use super::gbdk_c::drawing;

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
            _ => panic!("Style from u8 outbounded\0")
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
            _ => panic!("Color from u8 outbounded\0")
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

// TODO: Chante &str to &CStr
// Currently, rust-bundler-cp does not support CStr literal
pub fn gb_print(text: &str) {
    unsafe { drawing::gprint(text.as_ptr() as *const i8) }
}
