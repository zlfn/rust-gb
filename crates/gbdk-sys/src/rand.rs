// rand.h bindings — random number generation

pub const RAND_MAX: u8 = 255;
pub const RANDW_MAX: u16 = 65535;

unsafe extern "C" {
    pub static mut __rand_seed: u16;
}

unsafe extern "sdcccall-0" {
    pub fn initrand(seed: u16);
    pub fn rand() -> u8;
    pub fn randw() -> u16;
    pub fn initarand(seed: u16);
    pub fn arand() -> u8;
}
