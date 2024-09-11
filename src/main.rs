#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

extern {
    fn display_off();
    fn delay(delay: u16);
}

#[no_mangle]
pub extern fn main() -> u8 {
    unsafe {
        delay(10000);
    }
    return 42;
}

#[no_mangle]
pub fn add(left: u8, right: u8) -> u8 {
    left + right
}

#[allow(unconditional_recursion)]
#[panic_handler]
fn panic_handler_phony(info: &core::panic::PanicInfo) -> ! {
    panic_handler_phony(info)
}
