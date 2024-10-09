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

extern "C" {
    #[link_name="gprint __nonbanked"]
    pub fn gprint(str: *const c_char);
    #[link_name="gprintln __nonbanked"]
    pub fn gprintln(number: i16, radix: i8, signed_value: i8);
    #[link_name="gprintn __nonbanked"]
    pub fn gprintn(number: i8, radix: i8, signed_value: i8);
    #[link_name="gprintf __nonbanked"]
    pub fn gprintf(fmt: *const c_char, ...);
    #[link_name="plot __sdcccall(0)"]
    pub fn plot(x: u8, y: u8, colour: u8, mode: u8);
    #[link_name="plot_point __sdcccall(0)"]
    pub fn plot_point(x: u8, y:u8);
    #[link_name="switch_data __sdcccall(0)"]
    pub fn switch_data(x: u8, y: u8, src: *const u8, dst: *const u8);
    #[link_name="draw_image"]
    pub fn draw_image(data: *const u8);
    #[link_name="line __sdcccall(0)"]
    pub fn line(x1: u8, y1: u8, x2: u8, y2: u8);
    #[link_name="box __sdcccall(0)"]
    pub fn r#box(x1: u8, y1: u8, x2: u8, y2: u8, style: u8);
    #[link_name="circle __sdcccall(0)"]
    pub fn circle(x: u8, y: u8, radius: u8, style: u8);
    #[link_name="getpix __sdcccall(0)"]
    pub fn getpix(x: u8, y: u8) -> u8;
    #[link_name="wrtchr __sdcccall(0)"]
    pub fn wrtchr(chr: i8);
    #[link_name="gotogxy __sdcccall(0)"]
    pub fn gotogxy(x: u8, y: u8);
    #[link_name="color __sdcccall(0)"]
    pub fn color(forecolor: u8, backcolor: u8, mode: u8);
}
