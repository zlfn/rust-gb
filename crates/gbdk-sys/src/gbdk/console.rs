// gbdk/console.h bindings — console I/O

use core::ffi::c_char;

unsafe extern "sdcccall-0" {
    pub fn gotoxy(x: u8, y: u8);
    pub fn posx() -> u8;
    pub fn posy() -> u8;
    pub fn setchar(c: c_char);
}

unsafe extern "C" {
    pub fn cls();
}
