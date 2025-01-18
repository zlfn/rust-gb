#[allow(non_camel_case_types)]
type int_handler = extern "C" fn();

pub const SYSTEM_60HZ: u8 = 0x00;
pub const SYSTEM_50HZ: u8 = 0x01;

pub const J_UP: u8 = 0x04;
pub const J_DOWN: u8 = 0x08;
pub const J_LEFT: u8 = 0x02;
pub const J_RIGHT: u8 = 0x01;
pub const J_A: u8 = 0x10;
pub const J_B: u8 = 0x20;
pub const J_SELECT: u8 = 0x40;
pub const J_START: u8 = 0x80;

pub const M_DRAWING: u8 = 0x01;
pub const M_TEXT_OUT: u8 = 0x02;
pub const M_TEXT_INOUT: u8 = 0x03;
pub const M_NO_SCROLL: u8 = 0x04;
pub const M_NO_INTERP: u8 = 0x08;

pub const EMPTY_IFLAG: u8 = 0x00;
pub const VBL_IFLAG: u8 = 0x01;
pub const LCD_IFLAG: u8 = 0x02;
pub const TIM_IFLAG: u8 = 0x04;
pub const SIO_IFLAG: u8 = 0x08;
pub const JOY_IFLAG: u8 = 0x10;

extern "C" {
    pub fn remove_VBL(h: int_handler);
    pub fn remove_LCD(h: int_handler);
    pub fn remove_TIM(h: int_handler);
    pub fn remove_SIO(h: int_handler);
    pub fn remove_JOY(h: int_handler);

    #[link_name = "add_VBL __critical"]
    pub fn add_VBL(h: int_handler);
    #[link_name = "add_LCD __critical"]
    pub fn add_LCD(h: int_handler);
    #[link_name = "add_TIM __critical"]
    pub fn add_TIM(h: int_handler);
    #[link_name = "add_low_priority_TIM __critical"]
    pub fn add_low_priority_TIM(h: int_handler);
    #[link_name = "add_SIO __critical"]
    pub fn add_SIO(h: int_handler);
    #[link_name = "add_JOY __critical"]
    pub fn add_JOY(h: int_handler);

    pub fn nowait_int_handler();
    pub fn wait_int_handler();

    pub fn mode(mode: u8);

    #[link_name = "delay"]
    pub fn delay(delay: u16);

    #[link_name = "joypad __preserves_regs(b, c, h, l)"]
    pub fn joypad() -> u8;
    #[link_name = "waitpad __preserves_regs(b, c, h, l)"]
    pub fn waitpad(mask: u8) -> u8;
    #[link_name = "waitpadup __preserves_regs(a, b, c, d, e, h, l)"]
    pub fn waitpadup();

    // joypad_init
    // joypad_ex

    pub fn enable_interrupts();
    pub fn disable_interrupts();

    #[link_name = "set_interrupts __preserves_regs(b, c, d, e, h, l)"]
    pub fn set_interrupts(flags: u8);

    pub fn void();

    #[link_name = "vsync __preserves_regs(b, c, d, e, h, l)"]
    pub fn vsync();

    #[allow(clippy::deprecated_semver)]
    #[deprecated(since = "gbdk-2020", note = "please use `vsync` instead")]
    #[link_name = "wait_vbl_done __preserves_regs(b, c, d, e, h, l)"]
    pub fn wait_vbl_done();
}
