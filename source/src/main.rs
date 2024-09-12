#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm_experimental_arch)]

use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;
use core::{cmp, ptr};

extern crate alloc;

const GRAPHICS_WIDTH: u8 = 160;
const GRAPHICS_HEIGHT: u8 = 144;

const WHITE: u8 = 0;
const LTGREY: u8 = 1;
const DKGREY: u8 = 2;
const BLACK: u8 = 3;

const SIGNED: u8 = 1;
const UNSIGNED: u8 = 0;

const M_NOFILL: u8 = 0;
const M_FILL: u8 = 1;

const SOLID: u8 = 0x00;
const OR: u8 = 0x01;
const XOR: u8 = 0x02;
const AND: u8 = 0x03;

extern "C" {
    fn delay(delay: u16);
    #[link_name="circle __sdcccall(0)"]
    fn circle(x: u8, y: u8, radius: u8, style: u8);
    #[link_name="color __sdcccall(0)"]
    fn color(forecolor: u8, backcolor: u8, mode: u8);
    #[link_name="line __sdcccall(0)"]
    fn line(x1: u8, y1: u8, x2: u8, y2: u8);
}

mod libc {
    use core::ffi::c_void;

    #[allow(non_camel_case_types)]
    pub type size_t = usize;

    extern "C" {
        pub fn free(p: *mut c_void);
        //pub fn memalign(align: usize, size: usize) -> *mut c_void;
        pub fn malloc(size: size_t) -> *mut c_void;
    }
}

pub struct LibcAlloc;

#[global_allocator]
static ALLOCATOR: LibcAlloc = LibcAlloc;

#[cfg(not(target_family = "windows"))]
unsafe impl GlobalAlloc for LibcAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        //let align = layout.align().max(core::mem::size_of::<usize>());
        let size = layout.size();

        libc::malloc(size) as *mut u8
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = self.alloc(layout);
        if !ptr.is_null() {
            ptr::write_bytes(ptr, 0, layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        libc::free(ptr as *mut c_void);
    }

    unsafe fn realloc(&self, old_ptr: *mut u8, old_layout: Layout, new_size: usize) -> *mut u8 {
        let new_layout = Layout::from_size_align_unchecked(new_size, old_layout.align());
        let new_ptr = self.alloc(new_layout);
        if !new_ptr.is_null() {
            let size = cmp::min(old_layout.size(), new_size);
            ptr::copy_nonoverlapping(old_ptr, new_ptr, size);
            self.dealloc(old_ptr, old_layout);
        }
        new_ptr
    }
}

#[no_mangle]
pub extern fn main() {
    let mut x = 20;
    let mut y = 20;

    unsafe {
        color(BLACK, WHITE, SOLID);
        while x < GRAPHICS_WIDTH - 15 {
            y = 20;
            while y < GRAPHICS_HEIGHT - 15 {
                circle(x, y, 15, M_NOFILL);
                y += 10;
            }
            x += 10;
        }
    }
}

#[no_mangle]
pub fn add(left: u32, right: u32) -> u32 {
    left * right
}

#[allow(unconditional_recursion)]
#[cfg(not(test))]
#[panic_handler]
fn panic_handler_phony(info: &core::panic::PanicInfo) -> ! {
    panic_handler_phony(info)
}
