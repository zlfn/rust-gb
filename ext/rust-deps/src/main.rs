#![no_std]
#![no_main]

#[no_mangle]
pub extern fn main() {
}


/// This function is called on panic.
use core::panic::PanicInfo;

#[allow(unconditional_recursion)]
#[cfg(not(test))]
#[panic_handler]
fn panic_handler_phony(info: &core::panic::PanicInfo) -> ! {
    panic_handler_phony(info)
}
