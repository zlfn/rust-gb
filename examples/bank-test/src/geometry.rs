//! Banked geometry: struct returns, inherent methods, and trait impls, all
//! `Warp`-colored. Call sites read `value.method(args).drive()`.

use gb_bank::*;

bank::module!(2);

pub struct Point {
    pub x: u8,
    pub y: u8,
}

#[bank]
pub fn make_point(x: u8, y: u8) -> Point {
    Point { x, y }
}

// DISASM PROBE: a banked fn (geometry bank) calling another bank (arithmetic) via
// `.drive()`. Used to check whether the switch machinery lands resident or inline.
#[bank]
pub fn cross_add(x: u8) -> u8 {
    crate::arithmetic::add(x, 0x07).drive()
}

// PLACEMENT PROBE: a nested, guaranteed-outlined fn inside a banked fn. Its symbol
// leaf is `nested_helper` (not `__bank_*`), but its parent is `__bank_fn_probe_nest`,
// so the packer must pin it into geometry's bank, not leak it resident (bank 0).
#[bank]
pub fn probe_nest(x: u8) -> u8 {
    #[inline(never)]
    fn nested_helper(v: u8) -> u8 {
        v.wrapping_mul(3).wrapping_add(1)
    }
    nested_helper(x)
}

// ── Inherent methods ──

#[bank]
impl Point {
    pub fn manhattan(&self) -> u8 {
        self.x.wrapping_add(self.y)
    }

    pub fn offset(&self, dx: u8, dy: u8) -> Point {
        Point { x: self.x.wrapping_add(dx), y: self.y.wrapping_add(dy) }
    }
}

// ── Simple trait ──

#[bank]
pub trait Metric {
    fn distance(&self) -> u8;
    fn scale(&self, factor: u8) -> Point;
}

#[bank]
impl Metric for Point {
    fn distance(&self) -> u8 {
        if self.x > self.y {
            self.x
        } else {
            self.y
        }
    }

    fn scale(&self, factor: u8) -> Point {
        Point { x: self.x.wrapping_mul(factor), y: self.y.wrapping_mul(factor) }
    }
}

// ── Trait with associated type ──

#[bank]
pub trait Transform {
    type Output;
    fn apply(&self, val: u8) -> Self::Output;
}

#[bank]
impl Transform for Point {
    type Output = Point;
    fn apply(&self, val: u8) -> Self::Output {
        Point { x: self.x.wrapping_add(val), y: self.y.wrapping_sub(val) }
    }
}

// ── Generic trait + concrete impl ──

#[bank]
pub trait Combine<T> {
    fn combine(&self, other: T) -> u8;
}

#[bank]
impl Combine<u8> for Point {
    fn combine(&self, other: u8) -> u8 {
        self.x.wrapping_add(self.y).wrapping_add(other)
    }
}

// ── Generic struct + generic trait impl ──

pub struct Pair<A, B> {
    pub a: A,
    pub b: B,
}

#[bank]
pub trait Summary {
    fn summarize(&self) -> u8;
}

#[bank]
impl<A: Copy + Into<u8>, B: Copy + Into<u8>> Summary for Pair<A, B> {
    fn summarize(&self) -> u8 {
        let a: u8 = self.a.into();
        let b: u8 = self.b.into();
        a.wrapping_add(b)
    }
}

// Nested child module: its own bank group.
pub mod trig;
