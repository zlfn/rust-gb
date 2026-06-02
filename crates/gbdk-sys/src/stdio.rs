use core::ffi::c_char;

unsafe extern "C" {
    pub fn printf(format: *const c_char, ...);
    pub fn sprintf(str: *mut c_char, format: *const c_char, ...);
    pub fn puts(char: *const c_char);
}

unsafe extern "sdcccall-0" {
    pub fn putchar(c: c_char);
    pub fn gets(s: *mut c_char) -> *mut c_char;
    pub fn getchar();
}
