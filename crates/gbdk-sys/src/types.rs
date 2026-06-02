//! GBDK type definitions (from asm/types.h).

/// 8.8 fixed-point type.
///
/// Use `.w` for the full 16-bit value.
/// Use `.b.h` and `.b.l` for the high and low 8-bit parts.
///
/// Equivalent to GBDK's `fixed` union type.
#[derive(Clone, Copy)]
#[repr(C)]
pub union Fixed {
    pub b: FixedBytes,
    pub w: u16,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FixedBytes {
    pub l: u8,
    pub h: u8,
}
