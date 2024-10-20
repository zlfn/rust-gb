/// All Points Addressable (APA) mode drawing library.
/// 
/// This provies wrapper of `drawing.h` routines from GBDK.
///
/// # Caution
/// The GameBoy graphics hardware is not well suited to frame-buffer style
/// graphics such as the kind provided in `drawing`. Due to that, most drawing
/// functions will slow.
///
/// When possible it's much faster and more efficient to work will the tiles
/// and tiles maps that the GameBoy hardware is built around.
///
/// # Safety
/// Due to the complex side effect of APA mode, `drawing` functions can cause
/// unexpected issues. All the expected issues are wrapped in Rust-GB, but it is
/// your own risk to use this module.
pub mod drawing;

/// Helpers for GameBoy I/O. including buttons, texts, and else.
///
/// This modules contains a helper for simple input and output. you can print
/// text or read joypad input as bytes.
pub mod io;

#[cfg(feature = "prototype")]
pub mod memory;

/// Direct access API to GBDK extern functions
///
/// It's not recommended to using it, but you may be forced to use this due
/// to the incomplete Rust-GB functionality.
///
/// If so, we recommend that you read GBDK's documents sufficiently. And keep
/// in mind that there is a possibility of conflict with basic features of Rust-GB,
/// everything in this module is "unsafe"
pub mod gbdk_c;
