//! Game Boy hardware identification flags.
//!
//! The boot ROM leaves identification values in registers `A` and `B` when it
//! hands control to the cartridge at `0x100`, and they are valid only at that first
//! instruction.
//!
//! Startup routines store such registers to help subsequent code identify the
//! Game Boy machine type.
//!
//! | Register | Value | Console |
//! |----------|-------|---------|
//! | `A` | `0x01` | DMG, and Super Game Boy |
//! | `A` | `0xFF` | MGB (Pocket, Light) |
//! | `A` | `0x11` | CGB, GBA |
//! | `B` | bit 0 set | GBA running in CGB mode |

unsafe extern "C" {
    // SAFETY: rrt0 defines these, and updates them only before `main`.

    /// Value in register `A` set by the boot ROM.
    #[link_name = "_boot_a"]
    pub safe static BOOT_A: u8;
    /// Value in register `B` set by the boot ROM.
    #[link_name = "_boot_b"]
    pub safe static BOOT_B: u8;
}
