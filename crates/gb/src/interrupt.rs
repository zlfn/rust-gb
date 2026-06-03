#[inline]
pub fn disable() {
    unsafe { core::arch::asm!("di") }
}

#[inline]
pub unsafe fn enable() {
    unsafe { core::arch::asm!("ei") }
}
