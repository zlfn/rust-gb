#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(strict_provenance)]
#![feature(format_args_nl)]

use core::fmt::Write;

use gb::drawing::{DmgColor, DrawingMode, DrawingStream, DrawingStyle};

mod gb;

fn linetest(x: u8, y: u8, w: u8, draw: &DrawingStream) {
    let w = w as i8;
    DrawingStyle::reversed()
        .foreground(DmgColor::DarkGrey)
        .apply(draw);

    for i in -w..w+1 {
        draw.draw_line((x, y), (x+i as u8, y-w as u8));
    }
    for i in -w..w+1 {
        draw.draw_line((x, y), (x+w as u8, y+i as u8));
    }
    for i in -w..w+1 {
        draw.draw_line((x, y), (x+i as u8, y+w as u8));
    }
    for i in -w..w+1 {
        draw.draw_line((x, y), (x-w as u8, y+i as u8));
    }
}

#[no_mangle]
pub extern fn main() {
    let mut draw = unsafe {DrawingStream::new()};

    let mut c: u8 = 0;
    for a in 0..16 {
        for b in 0..16 {
            draw.cursor(b, a);
            let mut d = a/4;
            let e = b/4;
            if d == e {
                d = 3 - e;
            }
            draw.set_style(
                gb::drawing::DrawingStyle { 
                    foreground: d.into(), 
                    background: e.into(), 
                    drawing_mode: DrawingMode::Solid 
                });
            draw.write_byte(c);
            c += 1;
        }
    }

    let mut style = gb::drawing::DrawingStyle {
        foreground: DmgColor::LightGrey,
        background: DmgColor::White,
        drawing_mode: DrawingMode::Solid
    };
    style.apply(&draw);
    draw.draw_circle((140, 20), 15, true);
    style.foreground(DmgColor::Black).apply(&draw);
    draw.draw_circle((140, 20), 10, false);
    style
        .foreground(DmgColor::DarkGrey)
        .drawing_mode(DrawingMode::Xor)
        .apply(&draw);
    draw.draw_circle((120, 40), 30, true);
    draw.draw_line((0, 0), (159, 143));
    style
        .foreground(DmgColor::Black)
        .background(DmgColor::LightGrey)
        .drawing_mode(DrawingMode::Solid)
        .apply(&draw);
    draw.draw_box((0, 130), (40, 143), false);
    draw.draw_box((50, 130), (90, 143), true);

    linetest(255, 100, 20, &draw);
}

#[panic_handler]
#[allow(unused_variables)]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    //gb::io::GbStream::clear();
    //println!("PANIC occured");
    //println!("{}", info.message().as_str().unwrap());

    loop {}
}
