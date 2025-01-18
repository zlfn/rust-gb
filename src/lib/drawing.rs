//! All Points Addressable (APA) mode drawing library.
//!
//! This provies wrapper of `drawing.h` routines from GBDK.
//!
//! # Caution
//! The GameBoy graphics hardware is not well suited to frame-buffer style
//! graphics such as the kind provided in `drawing`. Due to that, most drawing
//! functions will slow.
//!
//! When possible it's much faster and more efficient to work will the tiles
//! and tiles maps that the GameBoy hardware is built around.
//!
//! We do not recommend using this function in Rust-GB.
//!
//! # Safety
//! Due to the complex side effect of APA mode, `drawing` functions can cause
//! unexpected issues. Most of expected issues are wrapped in Rust-GB, but it is
//! your own risk to use this module.

use core::{
    ffi::c_char,
    fmt::{Error, Write},
};

use super::gbdk_c::gb::{
    drawing::{self, circle, line, plot_point, r#box, M_FILL, M_NOFILL},
    gb::{mode, M_DRAWING},
};

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
    And = drawing::AND,
}

impl From<u8> for DrawingMode {
    fn from(value: u8) -> Self {
        match value {
            drawing::SOLID => Self::Solid,
            drawing::OR => Self::Or,
            drawing::XOR => Self::Xor,
            drawing::AND => Self::And,
            _ => panic!("DrawingMode from u8 outbounded\0"),
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
            _ => panic!("DmgColor from u8 outbounded\0"),
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
/// let mut w = unsafe {DrawingStream::new()};
/// DrawingStyle::default()
///     .foreground(DmgColor::LTGREY);
///     .apply(&w);
///
/// w.set_style(DrawingStyle::reversed());
/// ```
#[derive(PartialEq, Clone, Copy)]
pub struct DrawingStyle {
    pub foreground: DmgColor,
    pub background: DmgColor,
    pub drawing_mode: DrawingMode,
}

impl Default for DrawingStyle {
    /// Creates default `DrawingStyle`.
    ///
    /// Black drawings on a white background.
    ///
    /// Same as when GameBoy starts.
    fn default() -> Self {
        DrawingStyle {
            foreground: DmgColor::Black,
            background: DmgColor::White,
            drawing_mode: DrawingMode::Solid,
        }
    }
}

impl DrawingStyle {
    /// Creates reversed `DrawingStyle`.
    ///
    /// Black drawings on a white background.
    pub fn reversed() -> Self {
        DrawingStyle {
            foreground: DmgColor::White,
            background: DmgColor::Black,
            drawing_mode: DrawingMode::Solid,
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
    /// DrawingStream needed as parameter to ensure GameBoy is in `APA` mode.
    pub fn apply(&self, stream: &DrawingStream) {
        stream.set_style(*self);
    }
}

/// Byte drawing stream of GameBoy.
///
/// It simillars with [`crate::io::GbStream`]. But there are some
/// significant differences.
///
/// 1. `DrawingStream` uses `APA` mode drawing library of GBDK. this causes
/// many side effects, For example, if you try to use `DrawingStream` inside a
/// VBL interrupt, it will have unexpected result. For more detail, please refer
/// [GBDK Docs](https://gbdk-2020.github.io/gbdk-2020/docs/api/drawing_8h.html#aa8abfd58ea514228abd69d8f6330e91d)
///
/// 2. Unable to change line with `\n`, this means, when you want to make a new
/// line, you should use the [`DrawingStream::cursor`] function.
///
/// 3. `DrawingStream` can also draw shapes in addition to texts.
///
/// # Examples
///
/// ```
/// let mut s = unsafe {DrawingStream::new()}; // prints to second line.
/// write!(s, "Hello, APA!");
/// ```
pub struct DrawingStream {
    private: (),
}

impl DrawingStream {
    /// Creates new `DrawingStream`.
    ///
    /// Enable `APA` mode to draw texts.
    ///
    /// # Safety
    ///
    /// This will break [`crate::io::GbStream`].
    ///
    /// After this function, you cannot use GbStream dependent functions such as
    /// `println!`.
    pub unsafe fn new() -> Self {
        unsafe { mode(M_DRAWING) };
        DrawingStream { private: () }
    }

    /// Apply drawing style.
    ///
    /// Internally, call `color` function of GBDK.
    pub fn set_style(&self, style: DrawingStyle) {
        unsafe {
            drawing::color(
                style.foreground as u8,
                style.background as u8,
                style.drawing_mode as u8,
            )
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
    /// let mut s = unsafe {DrawingStream::new()};
    /// DrawingStream::cursor(0, 1); //prints to second line.
    /// write!(s, "Hello, Cursor!");
    ///
    /// ```
    pub fn cursor(&self, x: u8, y: u8) {
        if x >= TILE_WIDTH {
            panic!("Cursor x outbounded");
        }

        if y >= TILE_HEIGHT {
            panic!("Cursor y outbounded");
        }

        unsafe { drawing::gotogxy(x, y) }
    }

    fn panic_screen_bound(x: u8, y: u8) {
        if x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT {
            panic!("DrawingStream coordinate out of bounds");
        }
    }

    /// Draw a point to screen.
    ///
    /// # Panics
    ///
    /// Panics if coordinates out of bounds.
    pub fn draw_point(&self, (x, y): (u8, u8)) {
        Self::panic_screen_bound(x, y);

        unsafe { plot_point(x, y) }
    }

    /// Draw a line to screen.
    ///
    /// # Panics
    ///
    /// Panics if coordinates out of bounds.
    pub fn draw_line(&self, (x1, y1): (u8, u8), (x2, y2): (u8, u8)) {
        Self::panic_screen_bound(x1, y1);
        Self::panic_screen_bound(x2, y2);

        unsafe { line(x1, y1, x2, y2) }
    }

    /// Draw a box to screen.
    ///
    /// # Panics
    ///
    /// Panics if coordinates out of bounds.
    pub fn draw_box(&self, (x1, y1): (u8, u8), (x2, y2): (u8, u8), fill: bool) {
        Self::panic_screen_bound(x1, y1);
        Self::panic_screen_bound(x2, y2);

        if fill {
            unsafe { r#box(x1, y1, x2, y2, M_FILL) }
        } else {
            unsafe { r#box(x1, y1, x2, y2, M_NOFILL) }
        }
    }

    /// Draw a circle to screen.
    ///
    /// # Panics
    ///
    /// Panics if coordinates out of bounds.
    pub fn draw_circle(&self, (x, y): (u8, u8), radius: u8, fill: bool) {
        Self::panic_screen_bound(x, y);

        if fill {
            unsafe { circle(x, y, radius, M_FILL) }
        } else {
            unsafe { circle(x, y, radius, M_NOFILL) }
        }
    }

    /// Writes a byte into this writer, returning whether the write succeeded.
    ///
    /// write_char assumes that the input is valid Unicode character. However,
    /// GBDK maps one byte to one character or symbol.
    ///
    /// Therefore, `write_byte` is recommended when you want to print one
    /// character to the GameBoy.
    ///
    /// # Errors
    ///
    /// This function will return an instance of `Error` on error.
    pub fn write_byte(&mut self, b: u8) -> Result<(), Error> {
        unsafe { drawing::wrtchr(b as c_char) }
        Ok(())
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
