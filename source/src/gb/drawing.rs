use core::{ffi::c_char, fmt::Write};

use super::gbdk_c::gb::drawing;

pub const SCREEN_WIDTH: u8 = drawing::GRAPHICS_WIDTH;
pub const SCREEN_HEIGHT: u8 = drawing::GRAPHICS_WIDTH;
pub const TILE_WIDTH: u8 = SCREEN_WIDTH / 8;
pub const TILE_HEIGHT: u8 = SCREEN_HEIGHT / 8;

/// Drawing mode of `drawing` functions.
///
/// Determines how each drawing function overwrites pixels that already drawn.
///
/// The constant value follows the value of GBDK `drawing.h`
#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum DrawingMode {
    /// 0x00, Overwrites the existing pixels
    Solid = drawing::SOLID,

    /// 0x01, Performs a logical OR
    Or = drawing::OR,

    /// 0x02, Performs a logical XOR
    Xor = drawing::XOR,

    /// 0x03, Performs a logical AND
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

/// Color set of original GameBoy.
/// 
/// The constant value follows the value of GBDK `drawing.h`
#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum DmgColor {
    /// 0x00, WHITE color
    White = drawing::WHITE,

    /// 0x01, LTGREY color
    LightGrey = drawing::LTGREY,

    /// 0x02, DKGREY color
    DarkGrey = drawing::DKGREY,

    /// 0x03, BLACK color
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

/// The style in how `drawing` functions work.
///
/// Corresponds to the `color` function of GBDK.
///
/// Specify a color combination similar to the builder pattern, and apply it 
/// with a method `apply()`.
///
/// # Examples
/// ```
/// DrawingStyle::default()
///     .foreground(DmgColor::LTGREY);
///     .apply();
///
/// DrawingStyle::reversed().apply();
/// ```
#[derive(PartialEq, Clone, Copy)]
pub struct DrawingStyle {
    pub foreground: DmgColor,
    pub background: DmgColor,
    pub drawing_mode: DrawingMode,
}

impl DrawingStyle {
    /// Creates default `DrawingStyle`.
    ///
    /// Black drawings on a white background.
    ///
    /// Same as when GameBoy starts.
    pub fn default() -> Self {
        DrawingStyle {
            foreground: DmgColor::Black, 
            background: DmgColor::White, 
            drawing_mode: DrawingMode::Solid
        }
    }

    /// Creates reversed `DrawingStyle`.
    ///
    /// Black drawings on a white background.
    pub fn reversed() -> Self {
        DrawingStyle {
            foreground: DmgColor::White, 
            background: DmgColor::Black, 
            drawing_mode: DrawingMode::Solid
        }
    }

    /// Set foreground of `DrawingStyle`.
    pub fn foreground(&mut self, color: DmgColor) -> &mut Self {
        self.foreground = color;
        self
    }

    /// Set background of `DrawingStyle`.
    pub fn background(&mut self, color: DmgColor) -> &mut Self {
        self.background = color;
        self
    }

    /// Set drawing mode of `DrawingStyle`.
    pub fn drawing_mode(&mut self, mode: DrawingMode) -> &mut Self {
        self.drawing_mode = mode;
        self
    }

    /// Apply drawing style.
    ///
    /// Internally, call `color` function of GBDK.
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

/// Set cursor of [`DrawingStream`].
///
/// # Panics
///
/// Panics if coordinate parameter out of bounds.
///
/// # Safety
/// 
/// Because of the bound check, it is guaranteed to move the cursor to a
/// valid range.
///
/// # Examples
///
/// ```
/// let mut s = DrawingStream::new();
/// cursor(0, 1); //prints to second line.
/// write!(s, "Hello, Cursor!");
///
/// ```
pub fn cursor(x: u8, y: u8) {
        if x >= TILE_WIDTH {
            panic!("Cursor x outbounded");
        }

        if y >= TILE_HEIGHT {
            panic!("Cursor y outbounded");
        }

        unsafe {drawing::gotogxy(x, y)}
}

/// Byte drawing stream of GameBoy.
///
/// It simillars with [`crate::gb::io::GBStream`]. But there are some
/// significant differences.
///
/// 1. `DrawingStream` uses `APA` mode drawing library of GBDK. this causes
/// many side effects, For example, if you try to use `DrawingStream` inside a
/// VBL interrupt, it will have unexpected result. For more detail, please refer
/// [GBDK Docs](https://gbdk-2020.github.io/gbdk-2020/docs/api/drawing_8h.html#aa8abfd58ea514228abd69d8f6330e91d)
/// 
/// 2. Unable to change line with `\n`, this means, when you want to make a new
/// line, you should use the [`cursor`] fucntion.
///
/// # Examples
///
/// ```
/// let mut s = DrawingStream::with_cursor(0, 1); // prints to second line.
/// write!(s, "Hello, APA!");
/// ```
pub struct DrawingStream {}

impl DrawingStream {
    /// Creates new `DrawingStream`.
    ///
    /// Currently, this function is no-op.
    pub fn new() -> Self {
        DrawingStream {}
    }

    /// Creates new `DrawingStream` and move the cursor.
    ///
    /// Note that `cursor` is global state and does not belong to `DrawingStream`.
    pub fn with_cursor(x: u8, y: u8) -> Self {
        cursor(x , y);
        DrawingStream {}
    }
}


impl Write for DrawingStream {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            unsafe { drawing::wrtchr(c as c_char) }
        }
        Ok(())
    }
}

