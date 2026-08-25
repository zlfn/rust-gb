//! Bank Pack: ROM banking test suite on the new `gb-bank` API.
//! A: run tests, D-PAD: swap tiles.
//!
//! The resident `main` is `#[bank::zero]`, so `.drive()` / `.there()` thread its
//! ambient token automatically. Every banked module is its own bank group.

#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)]

use gb_bank::*;

mod arithmetic;
mod color;
mod dispatch;
mod geometry;

// ── Banked tile data, each table in its own bank group ──
mod tiles_a {
    use gb_bank::*;
    bank::module!();
    #[bank]
    pub static DATA: [u8; 10240] = *include_bytes!("../res/tiles_a.bin");
}
mod tiles_b {
    use gb_bank::*;
    bank::module!();
    #[bank]
    pub static DATA: [u8; 10240] = *include_bytes!("../res/tiles_b.bin");
}
mod tiles_c {
    use gb_bank::*;
    bank::module!();
    #[bank]
    pub static DATA: [u8; 10240] = *include_bytes!("../res/tiles_c.bin");
}
mod tiles_d {
    use gb_bank::*;
    bank::module!();
    #[bank]
    pub static DATA: [u8; 10240] = *include_bytes!("../res/tiles_d.bin");
}

use gbdk_sys::gb::gb::*;
use gbdk_sys::gbdk::console::*;
use gbdk_sys::stdio::*;

// ── Console helpers (resident, no banked calls) ──

unsafe fn hex(val: u8) {
    let h = |n: u8| -> i8 {
        if n < 10 { b'0' as i8 + n as i8 } else { b'A' as i8 + (n - 10) as i8 }
    };
    putchar(h(val >> 4));
    putchar(h(val & 0x0F));
}

/// Two-digit decimal (val <= 99), zero-padded for grid alignment.
unsafe fn dec2(val: u8) {
    putchar((b'0' + val / 10) as i8);
    putchar((b'0' + val % 10) as i8);
}

unsafe fn msg(s: &[u8]) {
    for &c in s {
        putchar(c as i8);
    }
}

/// Print a compact pass/fail cell in a 3-column grid.
///
/// Just the test number, no label, to fit ~16 tests on the 18-row screen.
unsafe fn check(n: &mut u8, got: u8, expected: u8) -> bool {
    let i = *n;
    *n += 1;
    gotoxy((i % 3) * 7, 2 + i / 3);
    dec2(i + 1);
    let ok = got == expected;
    msg(if ok { b" OK" } else { b" NG" });
    ok
}

// ── Resident helper (bank 0): drives a banked call, propagating the caller's
//    token via `#[bank::zero]`. Called as `resident_sum(..).drive()`. ──

#[bank::zero]
pub fn resident_sum(a: u8, b: u8) -> u8 {
    // A banked call inside a resident helper: switches to arithmetic's bank and
    // back to whatever bank the caller was in.
    arithmetic::add(a, b).drive()
}

// A `fn(u8) -> u8` resident helper, so it can be `far!`-ed and dropped into a
// dispatch table next to banked functions (resident -> Far<_, GroupZero>).
#[bank::zero]
pub fn resident_inc(x: u8) -> u8 {
    arithmetic::add(x, 1).drive()
}

/// Generic resident helper: drives a generic banked call. Monomorphized per `T`
/// and per caller group.
#[bank::zero]
pub fn resident_clamp<T: PartialOrd + Copy + BankSafe>(v: T, lo: T, hi: T) -> T {
    arithmetic::clamp(v, lo, hi).drive()
}

// A resident type whose methods drive banked calls: exercises `#[bank::zero]`
// on an `impl` (inherent + trait) and on a `trait`.
struct Counter {
    n: u8,
}

#[bank::zero]
impl Counter {
    fn bumped(&self, by: u8) -> u8 {
        arithmetic::add(self.n, by).drive()
    }
}

#[bank::zero]
trait Doubler {
    fn doubled(&self) -> u8;
}

#[bank::zero]
impl Doubler for Counter {
    fn doubled(&self) -> u8 {
        arithmetic::add(self.n, self.n).drive()
    }
}

// ── Main (resident entry, bank 0) ──

#[bank::main]
fn main() -> ! {
    unsafe {
        gbdk_sys::init();

        show_bkg();
        display_on();
        enable_interrupts();

        gotoxy(0, 0);
        msg(b"=== Bank Pack Test =");
        gotoxy(1, 16);
        msg(b"A:test D-PAD:tiles ");

        // Load tile data into VRAM while its bank is mapped (preserves text).
        tiles_a::DATA.there(|t| set_bkg_data(0, 8, t.as_ptr()));

        let mut prev: u8 = 0;
        let mut acc: u8 = 0x10;

        loop {
            vsync();
            let keys = joypad();
            let pressed = keys & !prev;

            if pressed & J_A != 0 {
                let v = acc;
                acc = acc.wrapping_add(1);

                let mut n: u8 = 0;
                let mut pass: u8 = 0;

                // 1. Free fn
                if check(&mut n, arithmetic::add(v, 0x19).drive(), v.wrapping_add(0x19)) {
                    pass += 1;
                }

                // 2. Generic fn
                let exp = if v < 0x10 { 0x10 } else if v > 0x20 { 0x20 } else { v };
                if check(&mut n, arithmetic::clamp(v, 0x10, 0x20).drive(), exp) {
                    pass += 1;
                }

                // 2b. Resident helper (#[bank::zero]) driving a banked call
                if check(&mut n, resident_sum(v, 3).drive(), v.wrapping_add(3)) {
                    pass += 1;
                }

                // 2c. Generic resident helper
                if check(&mut n, resident_clamp(v, 0x10, 0x20).drive(), exp) {
                    pass += 1;
                }

                // 2d. Resident impl method (#[bank::zero] impl)
                if check(&mut n, Counter { n: v }.bumped(5).drive(), v.wrapping_add(5)) {
                    pass += 1;
                }

                // 2e. Resident trait method (#[bank::zero] trait + impl)
                if check(&mut n, Counter { n: v }.doubled().drive(), v.wrapping_add(v)) {
                    pass += 1;
                }

                // 3. Inherent method + struct return
                let p = geometry::make_point(v, 0x05).drive();
                let q = p.offset(0x10, 0x20).drive();
                if check(&mut n, q.x, v.wrapping_add(0x10)) {
                    pass += 1;
                }

                // 4. Trait method
                use geometry::Metric;
                if check(&mut n, p.distance().drive(), if v > 0x05 { v } else { 0x05 }) {
                    pass += 1;
                }

                // 5. Trait + associated type
                use geometry::Transform;
                if check(&mut n, p.apply(3).drive().x, v.wrapping_add(3)) {
                    pass += 1;
                }

                // 5b. Trait method returning a struct
                if check(&mut n, p.scale(2).drive().x, v.wrapping_mul(2)) {
                    pass += 1;
                }

                // 5c. Generic trait method
                use geometry::Combine;
                if check(&mut n, p.combine(7).drive(), v.wrapping_add(0x05).wrapping_add(7)) {
                    pass += 1;
                }

                // 6. Generic trait + where clause
                use geometry::Summary;
                if check(&mut n, geometry::Pair { a: v, b: 3u8 }.summarize().drive(), v.wrapping_add(3)) {
                    pass += 1;
                }

                // 7. Nested module
                if check(&mut n, geometry::trig::sin8(0x10).drive(), 90) {
                    pass += 1;
                }

                // 8. Color (struct return + method)
                if check(&mut n, color::make_rgb(0x60, 0x30, 0x00).drive().brightness().drive(), 0x30) {
                    pass += 1;
                }

                // 9. Heterogeneous dispatch: banked AND resident, erased into one
                //    table. The resident entry dispatches through its runtime
                //    save/restore trampoline (Far<_, GroupZero>).
                let table: [DynFar<fn(u8) -> u8>; 4] = [
                    far!(dispatch::add).erase(),
                    far!(dispatch::xor).erase(),
                    far!(dispatch::mul).erase(),
                    far!(resident_inc).erase(),
                ];
                if check(&mut n, table[0].invoke(v), v.wrapping_add(0x11)) {
                    pass += 1;
                }
                if check(&mut n, table[3].invoke(v), v.wrapping_add(1)) {
                    pass += 1;
                }

                // 10. Banked data
                if check(&mut n, tiles_a::DATA.there(|t| t[0]), 0xFF) {
                    pass += 1;
                }

                // 11. scope + near sugar: the anchor, outer token, and each `.near()`
                //     token are all implicit. `b` pins the closure to arithmetic's
                //     group; the near calls run without switching, sharing the scope.
                let batched = scope(|b| {
                    let x = arithmetic::add(v, 1).near();
                    arithmetic::add(x, 2).near()
                });
                if check(&mut n, batched, v.wrapping_add(3)) {
                    pass += 1;
                }

                // 12b. PLACEMENT PROBE: banked fn with a nested outlined fn.
                if check(&mut n, geometry::probe_nest(v).drive(), v.wrapping_mul(3).wrapping_add(1)) {
                    pass += 1;
                }

                // 12. DISASM PROBE: banked->banked cross-bank call
                //     (geometry::cross_add internally drives arithmetic::add).
                if check(&mut n, geometry::cross_add(v).drive(), v.wrapping_add(0x07)) {
                    pass += 1;
                }

                // Summary (below the grid)
                gotoxy(0, 9);
                dec2(pass);
                msg(b"/");
                dec2(n);
                if pass == n {
                    msg(b" PASS v=");
                } else {
                    msg(b" FAIL v=");
                }
                hex(v);
                msg(b"  ");
            }

            // D-PAD: swap tile data (glyphs change, text stays).
            if pressed & J_LEFT != 0 {
                tiles_a::DATA.there(|t| set_bkg_data(0, 8, t.as_ptr()));
                gotoxy(0, 16);
                msg(b"tiles_a            ");
            }
            if pressed & J_UP != 0 {
                tiles_b::DATA.there(|t| set_bkg_data(0, 8, t.as_ptr()));
                gotoxy(0, 16);
                msg(b"tiles_b            ");
            }
            if pressed & J_RIGHT != 0 {
                tiles_c::DATA.there(|t| set_bkg_data(0, 8, t.as_ptr()));
                gotoxy(0, 16);
                msg(b"tiles_c            ");
            }
            if pressed & J_DOWN != 0 {
                tiles_d::DATA.there(|t| set_bkg_data(0, 8, t.as_ptr()));
                gotoxy(0, 16);
                msg(b"tiles_d            ");
            }

            prev = keys;
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
