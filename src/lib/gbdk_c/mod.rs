//! Direct access API to GBDK extern functions.
//!
//! It's not recommended to using it, but you may be forced to use this due
//! to the incomplete Rust-GB functionality.
//!
//! If so, we recommend that you read GBDK's documents sufficiently. And keep
//! in mind that there is a possibility of conflict with basic features of Rust-GB,
//! everything in this module is "unsafe"

/// Binding of GBDK's `gb/*.h`
pub mod gb;

/// Binding of GBDK's `rand.h`
pub mod rand;

/// Binding of GBDK's `stdio.h`
pub mod stdio;

/// Binding of GBDK's `console.h`
pub mod console;

/// Binding of GBDK's `font.h`
pub mod font;
