// ctype.h bindings — character classification

use core::ffi::c_char;

unsafe extern "C" {
    pub fn toupper(c: c_char) -> c_char;
    pub fn tolower(c: c_char) -> c_char;
}
