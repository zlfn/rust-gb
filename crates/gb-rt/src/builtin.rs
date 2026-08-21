//! Runtime routines provided by the startup code, exposed as raw symbols.
//!
//! The `RST` helpers use a register-based calling convention, so they are meant
//! to be invoked from inline assembly (for example through the `sym` operand),
//! not called directly. [`isr_noop`] is an ordinary parameterless routine.

unsafe extern "C" {
    /// `RST 0x20` indirect call helper: `jp (hl)`, jumping to the address in `HL`.
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
