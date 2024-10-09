use super::gbdk_c::drawing;

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

pub mod print {
    use core::ffi::CStr;

    use crate::gb::gbdk_c::drawing::{gotogxy, SIGNED, UNSIGNED};

    use super::{super::gbdk_c::drawing, TILE_HEIGHT, TILE_WIDTH};

    pub trait GPrint {
        fn gprint(&self);
    }

    pub trait GPrintNumber: GPrint {
        unsafe fn gprintn(&self, radix: u8);

        fn gprint_oct(&self) {
            unsafe {self.gprintn(8)}
        }

        fn gprint_hex(&self) {
            unsafe {self.gprintn(16)}
        }
    }

    impl<T: GPrintNumber> GPrint for T {
        fn gprint(&self) {
            unsafe {self.gprintn(10)}
        }
    }

    pub fn cursor(x: u8, y: u8) {
        if x >= TILE_WIDTH {
            panic!("Cursor x outbounded\0");
        }

        if y >= TILE_HEIGHT {
            panic!("Cursor y outbounded\0");
        }

        unsafe {gotogxy(x, y)}
    }

    pub fn print<T: GPrint>(s: T) {
        s.gprint();
    }

    pub fn print_oct<T: GPrintNumber>(s: T) {
        s.gprint_oct();
    }

    pub fn print_hex<T: GPrintNumber>(s: T) {
        s.gprint_hex();
    }

    // TODO: Delete this implementation.
    impl GPrint for &str {
        fn gprint(&self) {
            unsafe { drawing::gprint(self.as_ptr() as *const i8) }
        }
    }

    impl GPrint for CStr {
        fn gprint(&self) {
            unsafe { drawing::gprint(self.as_ptr() as *const i8) }
        }
    }

    impl GPrintNumber for u8 {
        unsafe fn gprintn(&self, radix: u8) {
            unsafe { drawing::gprintn(*self as i8, radix as i8, UNSIGNED as i8) }
        }
    }

    impl GPrintNumber for i8 {
        unsafe fn gprintn(&self, radix: u8) {
            unsafe { drawing::gprintn(*self as i8, radix as i8, SIGNED as i8) }
        }
    }

    impl GPrintNumber for u16 {
        unsafe fn gprintn(&self, radix: u8) {
            unsafe { drawing::gprintln(*self as i16, radix as i8, UNSIGNED as i8) }
        }
    }

    impl GPrintNumber for i16 {
        unsafe fn gprintn(&self, radix: u8) {
            unsafe { drawing::gprintln(*self as i16, radix as i8, SIGNED as i8) }
        }
    }
}
