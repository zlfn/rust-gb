use core::{ffi::c_char, fmt::Write};

use super::gbdk_c::stdio::putchar;

/// Prints to the GameBoy screen.
/// If you've ever used `print!` macro in `std`, you'll familiar with this.
///
/// Equivalent to the [`println!`] macro except that newline is not printed at
/// the end of the message.
///
/// The `print!` macro will make new `GBStream` on each call. This will probably
/// be optimized at the compilation time. So you don't have to worry about it.
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
        let mut s = crate::gb::io::GBStream::new();
        s.write_fmt(core::format_args!($($arg)*));
    }};
}

/// Prints to the GameBoy screen, with a newline.
/// If you've ever used `println!` macro in `std`, you'll familiar with this.
///
/// Equivalent to the [`print!`] macro except that newline is printed at the
/// end of the message.
///
/// The `println!` macro will make new `GBStream` on each call. This will probably
/// be optimized at the compilation time. So you don't have to worry about it.
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
        let mut s = crate::gb::io::GBStream::new();
        s.write_fmt(core::format_args_nl!($($arg)*));
    }};
}

/// Byte print stream of GameBoy.
///
/// Currently, GBStream prints bytes one by one using GBDK's `putchar` function.
/// In the long run, it is likely to change to RustGB own implementation.
///
/// # Examples
/// ```
/// use core::fmt::Write;
///
/// let mut w = GBStream::new();
/// write!(w, "Hello, World!");
/// ```
pub struct GBStream {}

impl GBStream {
    /// Creates new GBStream.
    ///
    /// Currently, this function is no-op
    pub fn new() -> Self {
        GBStream {}
    }
}

impl Write for GBStream {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            unsafe { putchar(c as c_char) }
        }
        Ok(())
    }
}
