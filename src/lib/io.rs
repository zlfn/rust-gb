//! Helpers for GameBoy I/O. including buttons, texts, and else.
//!
//! This modules contains a helper for simple input and output. you can print
//! text or read joypad input as bytes.

use core::{
    ffi::c_char,
    fmt::{Error, Write},
};

use crate::mmio::JOYP;

#[allow(unused_imports)]
use super::{
    drawing::{DmgColor, TILE_HEIGHT, TILE_WIDTH},
    gbdk_c::{
        console::{cls, gotoxy},
        font::{font_color, font_ibm, font_init, font_load, font_set, font_t},
        stdio::putchar,
    },
};

// Imports for docs
#[allow(unused_imports)]
use crate::gbdk_c;

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
    private: (),
}

impl GbStream {
    /// Get GameBoy console stream.
    pub fn stream() -> Self {
        GbStream { private: () }
    }

    /// Clear GameBoy console.
    pub fn clear() {
        unsafe {
            cls();
        }
    }

    /// Set cursor of [`GbStream`].
    ///
    /// # Panics
    ///
    /// Panics if coordinate parameter out of bounds.
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

        unsafe { gotoxy(x, y) };
    }

    /// Set a default font and custom color.
    ///
    /// # Caution
    ///
    /// It will clear GameBoy console and reset the cursor.
    #[cfg(feature = "prototype")]
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
    #[cfg(feature = "prototype")]
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
enum JoypadMode {
    None = 0x30,
    DPad = 0x20,
    Button = 0x10,
    All = 0x00,
}

/// Joypad key enum.
///
/// The value of the keys returned by the [`Joypad`] struct.
/// Same as the `J_*` consts in GBDK.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum JoypadKey {
    Right = 0x01,
    Left = 0x02,
    Up = 0x04,
    Down = 0x08,
    A = 0x10,
    B = 0x20,
    Select = 0x40,
    Start = 0x80,
}

/// Joypad input struct.
///
/// Joypad (Buttons and D-Pad key) can be checked by writing and reading the [`JOYP`]
/// register. (`0xFF00`) This struct makes it easy and safe.
///
/// The most recommended use is to receive key information on all keys at once using the [`Joypad::read()`]
/// function for once in each frame, and then check the key press using a method.
///
/// # Examples
/// ```
/// let key = Joypad::read();
/// if key.a() {
///     println!("A pressed!");
/// }
/// if key.b() {
///     println!("B pressed!");
/// }
/// ```
#[derive(Clone, Copy)]
pub struct Joypad(u8);

impl Joypad {
    fn change_mode(mode: JoypadMode) {
        unsafe { JOYP.write(mode as u8) };
        // 1 instruction delay is required after writing JOYP
        JOYP.read();
    }

    /// Get all buttons status.
    ///
    /// Internally, write and read twice in the [`JOYP`] register, and returns
    /// [`Joypad`] tuple struct with bitwise OR of [`JoypadKey`] values.
    pub fn read() -> Self {
        Joypad::change_mode(JoypadMode::Button);
        let buttons = (JOYP.read() << 4) | 0xF;
        Joypad::change_mode(JoypadMode::DPad);
        let d_pad = JOYP.read() | 0xF0;

        Joypad(!(buttons & d_pad))
    }

    /// Check if A button is pressed.
    pub fn a(&self) -> bool {
        self.0 & (JoypadKey::A as u8) != 0
    }

    /// Check if B button is pressed.
    pub fn b(&self) -> bool {
        self.0 & (JoypadKey::B as u8) != 0
    }

    /// Check if Select button is pressed.
    pub fn select(&self) -> bool {
        self.0 & (JoypadKey::Select as u8) != 0
    }

    /// Check if Start button is pressed.
    pub fn start(&self) -> bool {
        self.0 & (JoypadKey::Start as u8) != 0
    }

    /// Check if Right of d-pad is pressed.
    pub fn right(&self) -> bool {
        self.0 & (JoypadKey::Right as u8) != 0
    }

    /// Check if Left of d-pad is pressed.
    pub fn left(&self) -> bool {
        self.0 & (JoypadKey::Left as u8) != 0
    }

    /// Check if Up of d-pad is pressed.
    pub fn up(&self) -> bool {
        self.0 & (JoypadKey::Up as u8) != 0
    }

    /// Check if Down of d-pad is pressed.
    pub fn down(&self) -> bool {
        self.0 & (JoypadKey::Down as u8) != 0
    }

    /// Waits until any key pressed.
    pub fn wait_until_press() -> Self {
        let mut current = Joypad::read();
        while u8::from(current) == 0 {
            current = Joypad::read();
        }
        current
    }

    /// Waits until all key released.
    pub fn wait_until_release() {
        let mut current = Joypad::read();
        while u8::from(current) != 0 {
            current = Joypad::read();
        }
    }
}

impl From<Joypad> for u8 {
    fn from(value: Joypad) -> Self {
        value.0
    }
}
