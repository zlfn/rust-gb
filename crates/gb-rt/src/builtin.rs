//! Runtime routines provided by the startup code.
//!
//! The Game Boy offers the RST vectors, which can be used to jump quickly to
//! specific routines that are frequently used.
//!
//! [`call_hl()`] is assumed by both LLVM-Z80 and GBDK to be present.
//!
//! [`memset_small()`] and [`memcpy_small()`] are currently intended for
//! compatibility with GBDK. You can use them in inline assembly, but it
//! is not recommended. If necessary, LLVM-Z80 should optimize such byte copying.

unsafe extern "C" {
    /// `RST 0x20` indirect call helper: `jp (hl)`.
    #[link_name = "_call_hl"]
    pub fn call_hl();

    /// `RST 0x28` small memset: fill `C` bytes at `HL` with `A`.
    #[link_name = "_MemsetSmall"]
    pub fn memset_small();

    /// `RST 0x30` small memcpy: copy `C` bytes from `DE` to `HL`.
    #[link_name = "_MemcpySmall"]
    pub fn memcpy_small();

    /// Default interrupt handler: returns immediately.
    pub fn isr_noop();
}
