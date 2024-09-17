#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm_experimental_arch)]

mod gbdk;

extern "C" {
    fn test_asm_1();
    fn test_asm_2();
}

#[no_mangle]
pub extern fn main() {
    unsafe {
        test_asm_1();
        test_asm_2();
    }
}

#[allow(unconditional_recursion)]
#[cfg(not(test))]
#[panic_handler]
fn panic_handler_phony(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
