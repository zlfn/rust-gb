#![no_std]
#![no_main]

#[no_mangle]
pub extern fn main() {
}

#[allow(unconditional_recursion)]
#[cfg(not(test))]
#[panic_handler]
fn panic_handler_phony(info: &core::panic::PanicInfo) -> ! {
    panic_handler_phony(info)
}
