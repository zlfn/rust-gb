// setjmp.h bindings — non-local jumps
//
// SM83 jmp_buf: 3 bytes (1 SP + 2 return address)

pub type JmpBuf = [u8; 3];

unsafe extern "sdcccall-0" {
    #[link_name = "__setjmp"]
    pub fn setjmp(buf: *mut JmpBuf) -> i16;
}

unsafe extern "C" {
    pub fn longjmp(buf: *mut JmpBuf, val: i16) -> !;
}
