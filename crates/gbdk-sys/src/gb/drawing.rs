// gb/drawing.h bindings — All Points Addressable (APA) mode drawing library

use core::ffi::c_char;

pub const GRAPHICS_WIDTH: u8 = 160;
pub const GRAPHICS_HEIGHT: u8 = 144;

pub const SOLID: u8 = 0x00;
pub const OR: u8 = 0x01;
pub const XOR: u8 = 0x02;
pub const AND: u8 = 0x03;

pub const WHITE: u8 = 0;
pub const LTGREY: u8 = 1;
pub const DKGREY: u8 = 2;
pub const BLACK: u8 = 3;

pub const M_NOFILL: u8 = 0;
pub const M_FILL: u8 = 1;

pub const SIGNED: u8 = 1;
pub const UNSIGNED: u8 = 0;

unsafe extern "C" { 
    pub fn gprint(str: *mut c_char); 
    pub fn gprintln(number: i16, radix: i8, signed_value: i8);
    pub fn gprintn(number: i8, radix: i8, signed_value: i8);
    pub fn gprintf(fmt: *mut c_char, ...) -> i8;
    pub fn draw_image(data: *mut u8);
}

unsafe extern "sdcccall-0" {
    pub fn plot(x: u8, y: u8, colour: u8, mode: u8);
    pub fn plot_point(x: u8, y: u8);
    pub fn switch_data(x: u8, y: u8, src: *mut u8, dst: *mut u8);
    pub fn line(x1: u8, y1: u8, x2: u8, y2: u8);
    pub fn r#box(x1: u8, y1: u8, x2: u8, y2: u8, style: u8);
    pub fn circle(x: u8, y: u8, radius: u8, style: u8);
    pub fn getpix(x: u8, y: u8) -> u8;
    pub fn wrtchr(chr: c_char);
    pub fn gotogxy(x: u8, y: u8);
    pub fn color(forecolor: u8, backcolor: u8, mode: u8);
}
