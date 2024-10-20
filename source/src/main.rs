#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(strict_provenance)]
#![feature(format_args_nl)]

mod gb;

#[no_mangle]
pub extern fn main() {
    let u: u8 = u8::MIN;
    println!("Hello, World!");
    println!("{}", u);
}

#[allow(unconditional_recursion)]
#[panic_handler]
#[cfg(not(test))]
fn panic_handler_phony(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
