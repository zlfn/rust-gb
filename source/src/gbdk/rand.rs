extern "C" {
    #[link_name="rand __sdcccall(0)"]
    pub fn rand() -> u8;
    #[link_name="arand __sdcccall(0)"]
    pub fn arand() -> u8;
    #[link_name="initarand __sdcccall(0)"]
    pub fn initarand(seed: u16);
}
