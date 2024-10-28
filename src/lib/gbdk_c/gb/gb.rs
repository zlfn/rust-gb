#[allow(non_camel_case_types)]
type int_handler = extern fn();

pub const SYSTEM_60HZ: u8 = 0x00;
pub const SYSTEM_50HZ: u8 = 0x01;

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

    #[link_name="add_VBL __critical"]
    pub fn add_VBL(h: int_handler);
    #[link_name="add_LCD __critical"]
    pub fn add_LCD(h: int_handler);
    #[link_name="add_TIM __critical"]
    pub fn add_TIM(h: int_handler);
    #[link_name="add_low_priority_TIM __critical"]
    pub fn add_low_priority_TIM(h: int_handler);
    #[link_name="add_SIO __critical"]
    pub fn add_SIO(h: int_handler);
    #[link_name="add_JOY __critical"]
    pub fn add_JOY(h: int_handler);

    pub fn nowait_int_handler();
    pub fn wait_int_handler();

    #[link_name="set_interrupts __preserves_regs(b, c, d, e, h, l)"]
    pub fn set_interrupts(flags: u8);

    #[link_name="vsync __preserves_regs(b, c, d, e, h, l)"]
    pub fn vsync();

    pub fn mode(mode: u8);

    #[link_name="delay"]
    pub fn delay(delay: u16);
    #[link_name="waitpad __preserves_regs(b, c, h, l)"]
    pub fn waitpad(mask: u8) -> u8;
    #[link_name="waitpadup __preserves_regs(a, b, c, d, e, h, l)"]
    pub fn waitpadup();

}
