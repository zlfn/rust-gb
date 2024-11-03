#![no_std]
#![no_main]
#![feature(format_args_nl)]

use gb::{io::Joypad, println};

fn detect_once(condition: bool, message: &str, pressed: &mut bool) {
    if condition && !(*pressed) {
        println!("{}", message);
        *pressed = true;
    }
    else if !condition {
        *pressed = false;
    }
}

#[no_mangle]
pub extern fn main() {
    println!("Press Any Button");
    let mut pressed: [bool; 8] = [false; 8];
    loop {
        let pad = Joypad::read();
        detect_once(pad.a(), "A pressed!", &mut pressed[0]);
        detect_once(pad.b(), "B pressed!", &mut pressed[1]);
        detect_once(pad.select(), "Select pressed!", &mut pressed[2]);
        detect_once(pad.start(), "Start pressed!", &mut pressed[3]);
        detect_once(pad.up(), "Up pressed!", &mut pressed[4]);
        detect_once(pad.down(), "Down pressed!", &mut pressed[5]);
        detect_once(pad.left(), "Left pressed!", &mut pressed[6]);
        detect_once(pad.right(), "Right pressed!", &mut pressed[7]);
    }
}

#[panic_handler]
#[allow(unused_variables)]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
