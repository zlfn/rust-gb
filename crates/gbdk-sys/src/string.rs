// string.h bindings — string/memory functions

use core::ffi::{c_char, c_int, c_void};

unsafe extern "sdcccall-0" {
    pub fn reverse(s: *mut c_char) -> *mut c_char;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

// strlen/memcmp/memset/memcpy/memmove are runtime symbols that compiler_builtins
// already provides. The linker resolves them to those extern "C" (sdcccall(1))
// definitions, so we must declare them with the same ABI and canonical signature.
// The sdcccall(0) variants in gbdk's libsm83.a use `__sdcc`-suffixed names and are
// never linked.
unsafe extern "C" {
    pub fn strcat(s1: *mut c_char, s2: *const c_char) -> *mut c_char;
    pub fn strncat(s1: *mut c_char, s2: *const c_char, n: c_int) -> *mut c_char;
    pub fn strncmp(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int;
    pub fn strncpy(s1: *mut c_char, s2: *const c_char, n: c_int) -> *mut c_char;

    pub fn strlen(s: *const c_char) -> usize;
    pub fn memcmp(buf1: *const c_void, buf2: *const c_void, count: usize) -> c_int;

    #[deprecated(note = "use core::ptr::write_bytes")]
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    #[deprecated(note = "use core::ptr::copy_nonoverlapping")]
    pub fn memcpy(dest: *mut c_void, src: *const c_void, len: usize) -> *mut c_void;
    #[deprecated(note = "use core::ptr::copy")]
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}
