// gb/hardware.h bindings — hardware register constants and IO registers

/// Volatile pointer to a memory-mapped hardware register.
#[derive(Clone, Copy)]
pub struct Reg(pub *mut u8);

impl Reg {
    #[inline(always)]
    pub unsafe fn read(self) -> u8 {
        unsafe { core::ptr::read_volatile(self.0) }
    }
    #[inline(always)]
    pub unsafe fn write(self, val: u8) {
        unsafe { core::ptr::write_volatile(self.0, val) }
    }
}

// ── Hardware IO Registers ───────────────────────────────────────────────────

// Joypad
pub const P1_REG: Reg = Reg(0xFF00 as *mut u8);

// Serial
pub const SB_REG: Reg = Reg(0xFF01 as *mut u8);
pub const SC_REG: Reg = Reg(0xFF02 as *mut u8);

// Timer
pub const DIV_REG: Reg = Reg(0xFF04 as *mut u8);
pub const TIMA_REG: Reg = Reg(0xFF05 as *mut u8);
pub const TMA_REG: Reg = Reg(0xFF06 as *mut u8);
pub const TAC_REG: Reg = Reg(0xFF07 as *mut u8);

// Interrupt flags
pub const IF_REG: Reg = Reg(0xFF0F as *mut u8);

// Sound Channel 1
pub const NR10_REG: Reg = Reg(0xFF10 as *mut u8);
pub const NR11_REG: Reg = Reg(0xFF11 as *mut u8);
pub const NR12_REG: Reg = Reg(0xFF12 as *mut u8);
pub const NR13_REG: Reg = Reg(0xFF13 as *mut u8);
pub const NR14_REG: Reg = Reg(0xFF14 as *mut u8);

// Sound Channel 2
pub const NR21_REG: Reg = Reg(0xFF16 as *mut u8);
pub const NR22_REG: Reg = Reg(0xFF17 as *mut u8);
pub const NR23_REG: Reg = Reg(0xFF18 as *mut u8);
pub const NR24_REG: Reg = Reg(0xFF19 as *mut u8);

// Sound Channel 3
pub const NR30_REG: Reg = Reg(0xFF1A as *mut u8);
pub const NR31_REG: Reg = Reg(0xFF1B as *mut u8);
pub const NR32_REG: Reg = Reg(0xFF1C as *mut u8);
pub const NR33_REG: Reg = Reg(0xFF1D as *mut u8);
pub const NR34_REG: Reg = Reg(0xFF1E as *mut u8);

// Sound Channel 4
pub const NR41_REG: Reg = Reg(0xFF20 as *mut u8);
pub const NR42_REG: Reg = Reg(0xFF21 as *mut u8);
pub const NR43_REG: Reg = Reg(0xFF22 as *mut u8);
pub const NR44_REG: Reg = Reg(0xFF23 as *mut u8);

// Sound Master
pub const NR50_REG: Reg = Reg(0xFF24 as *mut u8);
pub const NR51_REG: Reg = Reg(0xFF25 as *mut u8);
pub const NR52_REG: Reg = Reg(0xFF26 as *mut u8);

// LCD
pub const LCDC_REG: Reg = Reg(0xFF40 as *mut u8);
pub const STAT_REG: Reg = Reg(0xFF41 as *mut u8);
pub const SCY_REG: Reg = Reg(0xFF42 as *mut u8);
pub const SCX_REG: Reg = Reg(0xFF43 as *mut u8);
pub const LY_REG: Reg = Reg(0xFF44 as *mut u8);
pub const LYC_REG: Reg = Reg(0xFF45 as *mut u8);
pub const DMA_REG: Reg = Reg(0xFF46 as *mut u8);
pub const BGP_REG: Reg = Reg(0xFF47 as *mut u8);
pub const OBP0_REG: Reg = Reg(0xFF48 as *mut u8);
pub const OBP1_REG: Reg = Reg(0xFF49 as *mut u8);
pub const WY_REG: Reg = Reg(0xFF4A as *mut u8);
pub const WX_REG: Reg = Reg(0xFF4B as *mut u8);

// CGB Speed
pub const KEY1_REG: Reg = Reg(0xFF4D as *mut u8);

// CGB VRAM Bank
pub const VBK_REG: Reg = Reg(0xFF4F as *mut u8);

// CGB HDMA
pub const HDMA1_REG: Reg = Reg(0xFF51 as *mut u8);
pub const HDMA2_REG: Reg = Reg(0xFF52 as *mut u8);
pub const HDMA3_REG: Reg = Reg(0xFF53 as *mut u8);
pub const HDMA4_REG: Reg = Reg(0xFF54 as *mut u8);
pub const HDMA5_REG: Reg = Reg(0xFF55 as *mut u8);

// CGB IR Port
pub const RP_REG: Reg = Reg(0xFF56 as *mut u8);

// CGB Palette
pub const BCPS_REG: Reg = Reg(0xFF68 as *mut u8);
pub const BCPD_REG: Reg = Reg(0xFF69 as *mut u8);
pub const OCPS_REG: Reg = Reg(0xFF6A as *mut u8);
pub const OCPD_REG: Reg = Reg(0xFF6B as *mut u8);

// CGB WRAM Bank
pub const SVBK_REG: Reg = Reg(0xFF70 as *mut u8);

// CGB PCM
pub const PCM12_REG: Reg = Reg(0xFF76 as *mut u8);
pub const PCM34_REG: Reg = Reg(0xFF77 as *mut u8);

// Interrupt Enable
pub const IE_REG: Reg = Reg(0xFFFF as *mut u8);

// ── Screen dimensions ───────────────────────────────────────────────────────

pub const DEVICE_SCREEN_WIDTH: u8 = 20;
pub const DEVICE_SCREEN_HEIGHT: u8 = 18;
pub const DEVICE_SCREEN_BUFFER_WIDTH: u8 = 32;
pub const DEVICE_SCREEN_BUFFER_HEIGHT: u8 = 32;
pub const DEVICE_SCREEN_X_OFFSET: u8 = 0;
pub const DEVICE_SCREEN_Y_OFFSET: u8 = 0;
pub const DEVICE_SCREEN_PX_WIDTH: u8 = 160;
pub const DEVICE_SCREEN_PX_HEIGHT: u8 = 144;
pub const DEVICE_SCREEN_MAP_ENTRY_SIZE: u8 = 1;
pub const DEVICE_SPRITE_PX_OFFSET_X: u8 = 8;
pub const DEVICE_SPRITE_PX_OFFSET_Y: u8 = 16;
pub const DEVICE_WINDOW_PX_OFFSET_X: u8 = 7;
pub const DEVICE_WINDOW_PX_OFFSET_Y: u8 = 0;

// ── LCDC flags ──────────────────────────────────────────────────────────────

pub const LCDCF_OFF: u8 = 0b00000000;
pub const LCDCF_ON: u8 = 0b10000000;
pub const LCDCF_WIN9800: u8 = 0b00000000;
pub const LCDCF_WIN9C00: u8 = 0b01000000;
pub const LCDCF_WINOFF: u8 = 0b00000000;
pub const LCDCF_WINON: u8 = 0b00100000;
pub const LCDCF_BG8800: u8 = 0b00000000;
pub const LCDCF_BG8000: u8 = 0b00010000;
pub const LCDCF_BG9800: u8 = 0b00000000;
pub const LCDCF_BG9C00: u8 = 0b00001000;
pub const LCDCF_OBJ8: u8 = 0b00000000;
pub const LCDCF_OBJ16: u8 = 0b00000100;
pub const LCDCF_OBJOFF: u8 = 0b00000000;
pub const LCDCF_OBJON: u8 = 0b00000010;
pub const LCDCF_BGOFF: u8 = 0b00000000;
pub const LCDCF_BGON: u8 = 0b00000001;

// ── STAT flags ──────────────────────────────────────────────────────────────

pub const STATF_LYC: u8 = 0b01000000;
pub const STATF_MODE10: u8 = 0b00100000;
pub const STATF_MODE01: u8 = 0b00010000;
pub const STATF_MODE00: u8 = 0b00001000;
pub const STATF_LYCF: u8 = 0b00000100;
pub const STATF_HBL: u8 = 0b00000000;
pub const STATF_VBL: u8 = 0b00000001;
pub const STATF_OAM: u8 = 0b00000010;
pub const STATF_LCD: u8 = 0b00000011;
pub const STATF_BUSY: u8 = 0b00000010;

// ── Interrupt enable flags ──────────────────────────────────────────────────

pub const IEF_HILO: u8 = 0b00010000;
pub const IEF_SERIAL: u8 = 0b00001000;
pub const IEF_TIMER: u8 = 0b00000100;
pub const IEF_STAT: u8 = 0b00000010;
pub const IEF_VBLANK: u8 = 0b00000001;

// ── Timer control ───────────────────────────────────────────────────────────

pub const TACF_START: u8 = 0b00000100;
pub const TACF_STOP: u8 = 0b00000000;
pub const TACF_4KHZ: u8 = 0b00000000;
pub const TACF_16KHZ: u8 = 0b00000011;
pub const TACF_65KHZ: u8 = 0b00000010;
pub const TACF_262KHZ: u8 = 0b00000001;

// ── Joypad register ─────────────────────────────────────────────────────────

pub const P1F_5: u8 = 0b00100000;
pub const P1F_4: u8 = 0b00010000;
pub const P1F_3: u8 = 0b00001000;
pub const P1F_2: u8 = 0b00000100;
pub const P1F_1: u8 = 0b00000010;
pub const P1F_0: u8 = 0b00000001;
pub const P1F_GET_DPAD: u8 = P1F_5;
pub const P1F_GET_BTN: u8 = P1F_4;
pub const P1F_GET_NONE: u8 = P1F_4 | P1F_5;

// ── Serial IO ───────────────────────────────────────────────────────────────

pub const SIOF_XFER_START: u8 = 0b10000000;
pub const SIOF_CLOCK_INT: u8 = 0b00000001;
pub const SIOF_CLOCK_EXT: u8 = 0b00000000;

// ── OAM / sprite attribute flags ────────────────────────────────────────────

pub const OAMF_PRI: u8 = 0b10000000;
pub const OAMF_YFLIP: u8 = 0b01000000;
pub const OAMF_XFLIP: u8 = 0b00100000;
pub const OAMF_PAL0: u8 = 0b00000000;
pub const OAMF_PAL1: u8 = 0b00010000;
pub const OAMF_BANK0: u8 = 0b00000000;
pub const OAMF_BANK1: u8 = 0b00001000;
pub const OAMF_CGB_PAL0: u8 = 0b00000000;
pub const OAMF_CGB_PAL1: u8 = 0b00000001;
pub const OAMF_CGB_PAL2: u8 = 0b00000010;
pub const OAMF_CGB_PAL3: u8 = 0b00000011;
pub const OAMF_CGB_PAL4: u8 = 0b00000100;
pub const OAMF_CGB_PAL5: u8 = 0b00000101;
pub const OAMF_CGB_PAL6: u8 = 0b00000110;
pub const OAMF_CGB_PAL7: u8 = 0b00000111;
pub const OAMF_PALMASK: u8 = 0b00000111;

// ── Background attribute flags (CGB) ────────────────────────────────────────

pub const BKGF_PRI: u8 = 0b10000000;
pub const BKGF_YFLIP: u8 = 0b01000000;
pub const BKGF_XFLIP: u8 = 0b00100000;
pub const BKGF_BANK0: u8 = 0b00000000;
pub const BKGF_BANK1: u8 = 0b00001000;
pub const BKGF_CGB_PAL0: u8 = 0b00000000;
pub const BKGF_CGB_PAL1: u8 = 0b00000001;
pub const BKGF_CGB_PAL2: u8 = 0b00000010;
pub const BKGF_CGB_PAL3: u8 = 0b00000011;
pub const BKGF_CGB_PAL4: u8 = 0b00000100;
pub const BKGF_CGB_PAL5: u8 = 0b00000101;
pub const BKGF_CGB_PAL6: u8 = 0b00000110;
pub const BKGF_CGB_PAL7: u8 = 0b00000111;

// ── Video bank (CGB) ────────────────────────────────────────────────────────

pub const VBK_BANK_0: u8 = 0;
pub const VBK_TILES: u8 = 0;
pub const VBK_BANK_1: u8 = 1;
pub const VBK_ATTRIBUTES: u8 = 1;

// ── HDMA (CGB) ──────────────────────────────────────────────────────────────

pub const HDMA5F_MODE_GP: u8 = 0b00000000;
pub const HDMA5F_MODE_HBL: u8 = 0b10000000;
pub const HDMA5F_BUSY: u8 = 0b10000000;

// ── Speed control (CGB) ─────────────────────────────────────────────────────

pub const KEY1F_DBLSPEED: u8 = 0b10000000;
pub const KEY1F_PREPARE: u8 = 0b00000001;
