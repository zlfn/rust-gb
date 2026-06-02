// gbdk/far_ptr.h bindings — banked/far pointer support

pub type FarPtr = u32;

pub const fn to_far_ptr(ofs: u16, seg: u16) -> FarPtr {
    ((seg as u32) << 16) | (ofs as u32)
}

pub const fn far_seg(ptr: FarPtr) -> u16 {
    (ptr >> 16) as u16
}

pub const fn far_ofs(ptr: FarPtr) -> u16 {
    ptr as u16
}
