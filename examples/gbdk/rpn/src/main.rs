//! GBDK RPN calculator, a port of gbdk-2020/examples/gb/rpn/rpn.c.
//!
//! A reverse-Polish-notation calculator: type space-separated numbers and the
//! operators `+ - * /`, then press Enter to print the top of the stack.

#![no_std]
#![no_main]

use core::ffi::c_char;
use gbdk_sys::stdio::{gets, printf, puts};

const STACKSIZE: usize = 40;
const MAXOP: usize = 40;

/// The NUMBER token: a parsed number is reported as the digit `'0'`.
const NUMBER: u8 = b'0';

fn cstr(s: &[u8]) -> *const c_char {
    s.as_ptr() as *const c_char
}

fn is_digit(c: c_char) -> bool {
    (b'0' as c_char..=b'9' as c_char).contains(&c)
}

/// Calculator state. Kept in a struct (not `static mut`) so the parser position
/// persists across `read_op` calls without tripping the 2024 static-mut lint.
struct Calc {
    sp: usize,
    stack: [i16; STACKSIZE],
    line: [c_char; MAXOP],
    pos: usize,
    n: i16,
}

impl Calc {
    const fn new() -> Self {
        Calc { sp: 0, stack: [0; STACKSIZE], line: [0; MAXOP], pos: 0, n: 0 }
    }

    fn push(&mut self, l: i16) {
        if self.sp < STACKSIZE {
            self.stack[self.sp] = l;
            self.sp += 1;
        } else {
            unsafe { puts(cstr(b"Stack full\0")) };
        }
    }

    fn pop(&mut self) -> i16 {
        if self.sp > 0 {
            self.sp -= 1;
            self.stack[self.sp]
        } else {
            unsafe { puts(cstr(b"Stack empty\0")) };
            0
        }
    }

    fn top(&self) -> i16 {
        if self.sp > 0 {
            self.stack[self.sp - 1]
        } else {
            unsafe { puts(cstr(b"Stack empty\0")) };
            0
        }
    }

    /// Next token: a number (reported as [`NUMBER`], value in `self.n`), an
    /// operator (its own char), or end of line (`'\n'`).
    fn read_op(&mut self) -> u8 {
        if self.pos == 0 {
            unsafe { gets(self.line.as_mut_ptr()) };
        }
        while self.line[self.pos] == b' ' as c_char || self.line[self.pos] == b'\t' as c_char {
            self.pos += 1;
        }
        if self.line[self.pos] == 0 {
            self.pos = 0;
            return b'\n';
        }
        if !is_digit(self.line[self.pos]) {
            let c = self.line[self.pos] as u8;
            self.pos += 1;
            return c;
        }
        self.n = (self.line[self.pos] as u8 - b'0') as i16;
        self.pos += 1;
        while is_digit(self.line[self.pos]) {
            self.n = 10 * self.n + (self.line[self.pos] as u8 - b'0') as i16;
            self.pos += 1;
        }
        NUMBER
    }
}

#[gb_rt::entry]
fn main() -> ! {
    unsafe { gbdk_sys::init() };
    unsafe { puts(cstr(b"RPN Calculator\0")) };

    let mut calc = Calc::new();
    loop {
        match calc.read_op() {
            NUMBER => {
                let n = calc.n;
                calc.push(n);
            }
            b'+' => {
                let v = calc.pop() + calc.pop();
                calc.push(v);
            }
            b'*' => {
                let v = calc.pop() * calc.pop();
                calc.push(v);
            }
            b'-' => {
                let op2 = calc.pop();
                let v = calc.pop() - op2;
                calc.push(v);
            }
            b'/' => {
                let op2 = calc.pop();
                if op2 != 0 {
                    let v = calc.pop() / op2;
                    calc.push(v);
                } else {
                    unsafe { puts(cstr(b"Divide by 0\0")) };
                }
            }
            b'\n' => unsafe {
                // GBDK printf reads the low 16 bits; pass a 32-bit arg as Rust
                // variadics require (c_int is only 16-bit on this target).
                printf(cstr(b"==> %d\n\0"), calc.top() as i32);
            },
            _ => {}
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
