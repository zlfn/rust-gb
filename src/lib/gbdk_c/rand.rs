pub const RAND_MAX: u8 = 255;
pub const RANDW_MAX: u16 = 65535;

extern "C" {
    #[link_name = "initrand __sdcccall(0)"]
    pub fn initrand(seed: u16);
    #[link_name = "rand __sdcccall(0)"]
    pub fn rand() -> u8;
    #[link_name = "randw __sdcccall(0)"]
    pub fn randw() -> u16;

    #[link_name = "initarand __sdcccall(0)"]
    pub fn initarand(seed: u16);
    #[link_name = "arand __sdcccall(0)"]
    pub fn arand() -> u8;
}
