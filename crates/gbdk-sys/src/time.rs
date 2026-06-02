// time.h bindings — time functions

pub const CLOCKS_PER_SEC: u16 = 60;

pub type TimeT = u16;

unsafe extern "C" {
    pub fn clock() -> TimeT;
    pub fn time(t: *mut TimeT) -> TimeT;
}
