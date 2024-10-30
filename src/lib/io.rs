//! Helpers for GameBoy I/O. including buttons, texts, and else.
//!
//! This modules contains a helper for simple input and output. you can print
//! text or read joypad input as bytes.

use core::{ffi::c_char, fmt::{Error, Write}};

use crate::{gbdk_c::gb::gb::delay, mmio::JOYP};

use super::{drawing::{DmgColor, TILE_HEIGHT, TILE_WIDTH}, gbdk_c::{console::{cls, gotoxy}, font::{font_color, font_ibm, font_init, font_load, font_set, font_t}, stdio::putchar}};

/// Prints to the GameBoy screen, with a newline.
/// If you've ever used `println!` macro in `std`, you'll familiar with this.
///
/// The `println!` macro will work with default `GbStream`. So, texts that
/// written with your custom GbStream will removed.
///
/// # Warning
///
/// Since the compiled fmt function is very large, care must be taken not to
/// exceed the ROM capacity of GameBoy.
///
/// In addition, compilation will fail if formatting is attempted for floating points
/// and integers over 32bits. Attempts to use `Debug` trait (`{:?}`) will also fail.
/// 
/// # Examples
///
/// ```
/// println!(); //prints just a newline
/// println!("Hello, Rust-GB!");
/// println!("Answer!: {}", 42);
/// ```
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut s = gb::io::GbStream::stream();
        s.write_fmt(core::format_args_nl!($($arg)*)).unwrap();
    }};
}

/// Prints to the GameBoy screen.
/// If you've ever used `print!` macro in `std`, you'll familiar with this.
///
/// Equivalent to the [`println!`] macro except that newline is not printed at
/// the end of the message.
///
/// The `print!` macro will work with default `GbStream`. So, texts that
/// written with your custom GbStream will removed.
///
/// # Warning
///
/// Since the compiled fmt function is very large, care must be taken not to
/// exceed the ROM capacity of GameBoy.
///
/// In addition, compilation will fail if formatting is attempted for floating points
/// and integers over 32bits. Attempts to use `Debug` trait (`{:?}`) will also fail.
///
/// # Examples
///
/// ```
/// print!("Hello, ");
/// print!("Rust-GB!\n");
/// print!("Answer!: {}", 42);
/// ```
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut s = gb::io::GbStream::stream();
        s.write_fmt(core::format_args!($($arg)*)).unwrap();
    }};
}

/// Byte print stream of GameBoy.
///
/// Currently, GbStream prints bytes one by one using GBDK's `putchar` function.
/// In the long run, it is likely to change to RustGB own implementation.
///
/// Optionally, GbStream can have font and color.
///
/// # Examples
/// ```
/// use core::fmt::Write;
///
/// let mut w = GbStream::new();
/// write!(w, "Hello, World!");
/// ```
pub struct GbStream {
    private: ()
}

impl GbStream {
    /// Get GameBoy console stream.
    pub fn stream() -> Self {
        GbStream { private: () }
    }

    /// Clear GameBoy console.
    pub fn clear() {
        unsafe { cls();}
    }

    /// Set cursor of [`GbStream`].
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
    /// cursor(0, 1); //prints to second line.
    /// print!("Hello, cursor!");
    ///
    /// ```
    pub fn cursor(x: u8, y: u8) {
        if x >= TILE_WIDTH {
            panic!("Cursor x outbounded");
        }

        if y >= TILE_HEIGHT {
            panic!("Cursor y outbounded");
        }

        unsafe {gotoxy(x, y)};
    }

    /// Set a default font and custom color.
    ///
    /// # Caution
    ///
    /// It will clear GameBoy console and reset the cursor.
    ///
    /// # Safety
    ///
    /// It is safe because only predetermined default fonts are loaded.
    pub fn set_color(foreground: DmgColor, background: DmgColor) {
        unsafe { 
            cls();
            font_init();
            font_color(foreground as u8, background as u8);
        }
        let font = unsafe { font_load(font_ibm) };
        unsafe { font_set(font) };
    }

    /// Set a GbStream with custom font.
    ///
    /// # Caution
    ///
    /// It will clear GameBoy console and reset the cursor.
    ///
    /// # Safety
    ///
    /// If an invalid font address is entered, it causes an Undefined Behavior.
    pub unsafe fn set_font_and_color(font: font_t, foreground: DmgColor, background: DmgColor) {
        unsafe { 
            cls();
            font_init();
            font_color(foreground as u8, background as u8) 
        }
        let font = unsafe { font_load(font) };
        unsafe { font_set(font) };
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
        unsafe { putchar(b as c_char) }
        Ok(())
    }
}

impl Write for GbStream {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            unsafe { putchar(c as c_char) }
        }
        Ok(())
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
enum JoypadMode {
    None = 0x30,
    DPad = 0x20,
    Button = 0x10,
    All = 0x00
}

pub struct Joypad { }

impl Joypad {
    fn change_mode(mode: JoypadMode) {
        unsafe {JOYP.write(mode as u8)};
        // 1 instruction delay is required after writing JOYP
        JOYP.read();
    }

    pub fn a() -> bool {
        Self::change_mode(JoypadMode::Button);
        JOYP.read() & (1 << 0) == 0
    }

    pub fn b() -> bool {
        Self::change_mode(JoypadMode::Button);
        JOYP.read() & (1 << 1) == 0
    }

    pub fn select() -> bool {
        Self::change_mode(JoypadMode::Button);
        JOYP.read() & (1 << 2) == 0
    }

    pub fn start() -> bool {
        Self::change_mode(JoypadMode::Button);
        JOYP.read() & (1 << 3) == 0
    }

    pub fn right() -> bool {
        Self::change_mode(JoypadMode::DPad);
        JOYP.read() & (1 << 0) == 0
    }

    pub fn left() -> bool {
        Self::change_mode(JoypadMode::DPad);
        JOYP.read() & (1 << 1) == 0
    }

    pub fn up() -> bool {
        Self::change_mode(JoypadMode::DPad);
        JOYP.read() & (1 << 2) == 0
    }

    pub fn down() -> bool {
        Self::change_mode(JoypadMode::DPad);
        JOYP.read() & (1 << 3) == 0
    }
}


