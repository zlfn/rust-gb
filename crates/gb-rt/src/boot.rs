//! Hardware identification captured by the startup code at boot.
//!
//! The boot ROM leaves identification values in registers `A` and `B` when it
//! hands control to the cartridge at `0x100`, valid only at that first
//! instruction. `rrt0` captures them into `__boot_a` and `__boot_b`; this module
//! reads them back. Interpreting the values is left to callers:
//!
//! | Register | Value | Console |
//! |----------|-------|---------|
//! | `A` | `0x01` | DMG, and Super Game Boy |
//! | `A` | `0xFF` | MGB (Pocket, Light) |
//! | `A` | `0x11` | CGB, also reported by the Game Boy Advance |
//! | `B` | bit 0 set | Game Boy Advance running in CGB mode |

unsafe extern "C" {
    #[link_name = "_boot_a"]
    static BOOT_A: u8;
    #[link_name = "_boot_b"]
    static BOOT_B: u8;
}

/// Value left in register `A` by the boot ROM.
#[inline]
pub fn a() -> u8 {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(BOOT_A)) }
}

/// Value left in register `B` by the boot ROM.
#[inline]
pub fn b() -> u8 {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(BOOT_B)) }
}
