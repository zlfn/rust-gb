// stdlib.h bindings — standard library functions

use core::ffi::{c_char, c_int, c_void};

unsafe extern "sdcccall-0" {
    pub fn exit(status: c_int);
    pub fn getkey() -> c_int;
    pub fn labs(num: i32) -> i32;
    pub fn itoa(n: c_int, s: *mut c_char, radix: u8) -> *mut c_char;
    pub fn uitoa(n: u16, s: *mut c_char, radix: u8) -> *mut c_char;
    pub fn ltoa(n: i32, s: *mut c_char, radix: u8) -> *mut c_char;
    pub fn ultoa(n: u32, s: *mut c_char, radix: u8) -> *mut c_char;
}

unsafe extern "C" {
    pub fn abs(i: c_int) -> c_int;
    pub fn atoi(s: *const c_char) -> c_int;
    pub fn atol(s: *const c_char) -> i32;
    pub fn calloc(nmemb: u16, size: u16) -> *mut c_void;
    pub fn malloc(size: u16) -> *mut c_void;
    pub fn realloc(ptr: *mut c_void, size: u16) -> *mut c_void;
    pub fn free(ptr: *mut c_void);
}
