use core::ffi::c_char;

extern "C" {
    #[link_name="putchar __sdcccall(0)"]
    pub fn putchar(char: c_char);

    pub fn printf(format: *const c_char, ...); 

    pub fn sprintf(char: *mut c_char, format: *const c_char, ...);

    pub fn puts(char: *const c_char);

    #[link_name="gets __sdcccall(0)"]
    pub fn gets(s: *mut c_char) -> *const c_char;

    #[link_name="getchar __sdcccall(0)"]
    pub fn getchar() -> c_char;
}
