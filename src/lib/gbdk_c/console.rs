use core::ffi::c_char;
extern "C" {
    #[link_name = "gotoxy __sdcccall(0)"]
    pub fn gotoxy(x: u8, y: u8);
    #[link_name = "posx __sdcccall(0)"]
    pub fn posx() -> u8;
    #[link_name = "posy __sdcccall(0)"]
    pub fn posy() -> u8;
    #[link_name = "setchar __sdcccall(0)"]
    pub fn setchar(c: c_char);

    pub fn cls();
}
