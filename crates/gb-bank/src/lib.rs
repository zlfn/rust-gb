//! Compile-time-safe ROM bank switching for the Game Boy.
//!
//! `gb-bank` is the user-facing front end of the banking toolchain. It provides
//! the runtime types together with the macros that turn ordinary functions and
//! statics into bank-safe ones.
//!
//! # Overview
//!
//! A Game Boy cartridge maps one 16 KiB ROM **bank** at a time into the
//! `0x4000..0x8000` window; anything in another bank is unaddressable until it is
//! switched in. This crate makes that switching safe at compile time by tracking,
//! in the type system, which bank is mapped (see [`Bank`] and [`scope`] below).
//!
//! You normally write:
//!
//! ```ignore
//! use gb_bank::*;
//!
//! mod sound {
//!     use gb_bank::*;
//!     bank::module!();                 // this module is a bank group
//!
//!     #[bank]
//!     pub static NOTES: [u8; 4] = [60, 62, 64, 65];
//!
//!     #[bank]
//!     pub fn play(i: u8) -> u8 {
//!         NOTES.local()[i as usize] // read banked data (same bank, no switch)
//!     }
//! }
//!
//! #[bank::main]
//! pub fn main() {
//!     let note = sound::play(0).drive(); // cross-bank call from the bank-0 loop
//! }
//! ```
//!
//! # Components
//!
//! ## `Group` and `Bank<G>`
//!
//! A [`Group`] is the compile-time identity of one bank: a zero-sized type
//! ([`bank::module!()`](bank::module) generates one per banked module) plus its
//! runtime bank number [`bank()`](Group::bank). A [`Bank<G>`](Bank) is a zero-sized
//! token, and holding one witnesses that group `G`'s bank is currently mapped.
//!
//! This follows [GhostCell], which keeps a permission token (`GhostToken<'id>`)
//! apart from the data it guards, tied by a brand. gb-bank uses a group *type* `G`
//! as the brand where GhostCell uses a lifetime `'id`: a bank's identity is a fixed,
//! nameable type reused across many functions, which a per-scope lifetime brand
//! cannot express. The token is `!Send`, not `Clone`, and mintable only through
//! [`scope`] (or the unsafe [`assume`](Bank::assume)), so "this bank is mapped"
//! cannot be fabricated; [`bank()`](Group::bank) is read only for the actual switch.
//!
//! ## `scope` and `Anchor`
//!
//! [`scope`]`(anchor, outer, |b: &mut Bank<G>| ..)` is the switch primitive the macros
//! build on. It switches into group `G`, lends the closure a fresh [`Bank<G>`](Bank)
//! token, runs it, then restores the caller's bank. It is GhostCell's
//! `GhostToken::new`, except it also drives the hardware. The switch is elided when
//! `G` is already mapped, that is, the caller is the same group or the always-mapped
//! [`GroupZero`].
//!
//! Because the closure runs while a *different* bank is mapped, its own code must
//! stay reachable, which is only true for code in bank 0. [`scope`] proves
//! that with an [`Anchor`]: a zero-sized, [`Copy`] witness "this code is in bank 0".
//! `#[bank::main]` / `#[bank::zero]` bodies hold one; a `#[bank]` function does not,
//! so it cannot open a [`scope`] (its body would be switched out from under the
//! closure), it uses [`local`](Far::local) / [`near`](Warp::near) for same-bank work and
//! [`drive`](Warp::drive) / [`invoke`](FarCall::invoke) for cross-bank calls instead.
//!
//! ```ignore
//! # use gb_bank::*;
//! # struct Sound; impl Group for Sound { const FIXED: bool = false; fn bank() -> u8 { 2 } }
//! fn mix(anchor: Anchor, bank: &mut Bank<GroupZero>) -> u8 {
//!     scope(anchor, bank, |b: &mut Bank<Sound>| {
//!         let a = *NOTES.local(b);    // read banked data with the scope's token
//!         a.wrapping_add(1)         // no further switch inside the block
//!     })
//! }
//! ```
//!
//! ## `Far<T, G>`
//!
//! A [`Far<T, G>`](Far) points to a `T` living in group `G`'s bank. The pointer
//! itself is just an address plus the brand `G` (a plain value, not the banked
//! data it points at), so it is [`Copy`] and can be stored and passed around freely,
//! but it cannot be read without a [`Bank<G>`](Bank): [`local`](Far::local) returns
//! `&T` only by borrowing the token.
//!
//! Splitting the address (always safe to hold) from the access (needs the token)
//! lets a far pointer survive bank switches while keeping each read checked. The
//! returned reference borrows the token, and a switch needs that token `&mut`, so a
//! reference into a bank cannot be held across a switch away from it. It is the data
//! half of GhostCell's `GhostCell<'id, T>`.
//!
//! ## `Warp`
//!
//! A `#[bank]` function returns `impl `[`Warp`]`<Output = R>`. Naming `play(0)` does
//! not run the body; it captures the call as a [`Warp`], which [`drive`](Warp::drive)
//! then runs: switch into the target bank, run, restore the caller.
//!
//! This follows [`Future`]. An `async fn` returns an `impl Future` that stays inert
//! until `.await`; a banked call is deferred so the switch can take the *caller's*
//! own [`Bank<C>`](Bank) token at the drive point. [`drive`](Warp::drive) restores
//! the caller's bank from that token's type, and elides the switch when caller and
//! callee share a bank. A `Warp` is driven by passing the token, not polled by an
//! executor.
//!
//! ## Cross-bank vs same-bank
//!
//! [`drive`](Warp::drive) (run a [`Warp`]) and [`invoke`](FarCall::invoke) (call a far
//! function) are *cross-bank*: each switches into the target bank, runs the one
//! function, and switches back. They run no caller closure across the switch, so they
//! are safe from *any* body, banked or bank-0 (they compile to a bank-0
//! trampoline). A run of them into the same bank re-switches each time.
//!
//! [`there`](FarWith::there) (borrow far data) and [`scope`] are also cross-bank, but
//! they run *your* closure while the other bank is mapped, so they need an [`Anchor`]
//! and are available only in a bank-0 (`#[bank::main]` / `#[bank::zero]`) body.
//!
//! The *same-bank* operations take a token already in the target bank, so they never
//! switch: [`near`](Warp::near) for a call, [`local`](Far::local) for data. By taking a
//! `Bank<G>` of the exact group they also pin that group, and they work from any body.
//!
//! So to batch work in one bank, open a [`scope`] into it (from a bank-0 body) and
//! use `near` / `local` inside, sharing the single switch [`scope`] performed:
//!
//! ```ignore
//! # use gb_bank::*;
//! # #[bank::main] fn demo(v: u8) {
//! let r = scope(|b| {              // anchor + outer token injected (Macro sugar)
//!     let x = sound::play(v).near(b);  // no switch: already in sound's bank
//!     sound::play(x).near(b)           // `b` pins the group, so no annotation
//! });
//! # }
//! ```
//!
//! ## `DynFar<T>`
//!
//! A [`DynFar<T>`](DynFar) is a [`Far`] with its group erased to a runtime bank
//! number, produced by [`erase`](Far::erase). A `Far<T, G>` carries its group in the
//! type, so pointers into different banks have different types and cannot share a
//! collection. Erasing the group to an inline `u8` lets functions or data from
//! several banks sit in one `[DynFar<T>; N]` table for runtime dispatch. The trade
//! is that every access switches, with no compile-time group left to elide against;
//! the erasure itself is allocation- and vtable-free.
//!
//! ```ignore
//! # use gb_bank::*;
//! fn dispatch(bank: &mut Bank<GroupZero>, i: usize, state: u8) -> u8 {
//!     let table: [DynFar<fn(u8) -> u8>; 2] = [
//!         far!(enemy::ai).erase(),     // lives in some bank
//!         far!(hud_tick).erase(),      // bank-0 #[bank::zero] helper
//!     ];
//!     table[i].invoke(bank, (state,))    // switch to its bank, run, restore
//! }
//! ```
//!
//! [GhostCell]: https://plv.mpi-sws.org/rustbelt/ghostcell/
//!
//! # What the macros do
//!
//! - [`bank::module!()`](bank::module) declares the enclosing module as a
//!   bank group. `bank::module!(N)` instead *pins* it to bank `N` (the packer
//!   reserves that bank rather than auto-assigning one).
//! - [`bank::inherit!()`](bank::inherit) in a submodule makes it part of its
//!   *parent* module's bank group instead of forming a new one (`super` is a
//!   reserved keyword, hence the name).
//! - [`#[bank]`](macro@bank) on a `fn` rewrites it to return `impl Warp`, the
//!   deferred-call analog of a [`Future`]: `play(0)` captures the call,
//!   `.drive()` performs the bank switch and runs it. On a `static` it exposes a
//!   [`Far`] to the banked data. Also works on an `impl` or `trait`. Its return type
//!   must be [`WarpSafe`] (no bare `fn` pointer or `dyn` to banked code, which the
//!   caller would run with the bank unmapped, carry one as a [`Far`] / [`DynFar`]).
//! - [`#[bank::main]`](bank::main) marks the **entry point** in bank 0.
//!   It holds a [`GroupZero`] token and drives banked calls directly.
//! - [`#[bank::zero]`](bank::zero) marks a bank-0 **helper** that may
//!   be called from inside any bank: being [`Warp`]-colored like `#[bank]`, it
//!   forwards the caller's token, so its banked calls restore the caller's bank.
//! - [`far!`](macro@far) takes a [`Far`] to a banked function for heterogeneous
//!   dispatch; `.erase()` collects functions from different banks into one table.
//!   It also works on a bank-0 `#[bank::zero]` helper, so a table can mix banked
//!   and bank-0 functions.
//!
//! ## Bank layout
//!
//! By default gb-bank-pack bin-packs the banked modules into 16 KiB ROM banks. Two
//! knobs control the layout:
//!
//! - `bank::module!(N)` **pins** a module to ROM bank `N`. Pin each module to a
//!   distinct bank for a deterministic, one-module-per-bank layout, so a call from
//!   one module to another is always a genuine cross-bank switch.
//! - [`bank::inherit!()`](bank::inherit) in a submodule folds it into its *parent*
//!   module's bank instead of forming a new group.
//!
//! ```ignore
//! mod audio   { use gb_bank::*; bank::module!(1); /* ... */ }
//! mod physics {
//!     use gb_bank::*;
//!     bank::module!(2);
//!     pub mod trig { use gb_bank::*; bank::inherit!(); /* shares physics: bank 2 */ }
//! }
//! ```
//!
//! Bank 0 is the always-mapped region (`#[bank::main]` / `#[bank::zero]` live there)
//! and cannot be pinned to. With no MBC at all the whole 32 KiB ROM is fixed and
//! there is no switching.
//!
//! ## Macro sugar
//!
//! Inside any `#[bank]` / `#[bank::main]` / `#[bank::zero]` body the macro injects the
//! ambient bank token (and, in a bank-0 body, an [`Anchor`]), and these forms take
//! them as implicit leading arguments, so they never appear in your code:
//!
//! | you write | the macro emits | where |
//! |---|---|---|
//! | `enemy::ai(s).drive()` | `enemy::ai(s).drive(&mut __bank)` | any body |
//! | `enemy::ai(s).near()` | `enemy::ai(s).near(&mut __bank)` | any body |
//! | `table[i].invoke(s)` | `table[i].invoke(&mut __bank, (s,))` | any body |
//! | `NOTES.local()` | `NOTES.local(&__bank)` | any body |
//! | `NOTES.there(\|t\| ..)` | `NOTES.there(__anchor, &mut __bank, \|t\| ..)` | bank 0 only |
//! | `scope(\|b\| ..)` | `scope(__anchor, &mut __bank, \|b\| ..)` | bank 0 only |
//!
//! `there` / `scope` in a `#[bank]` body are a compile error (no [`Anchor`] there),
//! with a message pointing you to `local` / `near`. A [`scope`] rebinds the ambient
//! token to its own closure parameter `b`, so a nested `scope` or a `.drive()` /
//! `.near()` / `.local()` *inside* it threads `b` (the innermost bank), not the outer
//! token; the `Anchor`, being [`Copy`], flows in automatically. The ambient `__bank`
//! is hidden and cannot be named, but the token `b` that [`scope`] binds can.
//!
//! ### Threading is by method name, not type
//!
//! The rewrite runs before type checking, so it matches on the method *name* alone:
//! it threads the token into *every* `.drive()` / `.near()` / `.local()` /
//! `.invoke()` / `.there(..)` (and `scope(..)`) in the body, regardless of the
//! receiver's type. The names are deliberately uncommon to avoid colliding with an
//! unrelated method of the same name. If one does collide, call that method through
//! fully-qualified syntax: `Type::method(recv, args)` is a path call, not method-call
//! sugar, so the macro leaves it untouched.
//!
//! # Panics, unwinding, and interrupts
//!
//! The bank restore in [`scope`] (and the [`Far`] call / borrow paths) runs *after*
//! the closure returns, so an unwinding panic would skip it and leave the wrong
//! bank mapped. This is sound on the Game Boy because the target aborts on panic
//! (there is no unwinder), so a panic never resumes into a stale-bank token. These
//! types are not designed to be unwind-safe on a hosted, unwinding target.
//!
//! The safety model also assumes nothing changes the mapped bank *behind the
//! token's back*. An interrupt handler that switches banks (e.g. to read banked
//! data) must save [`current_bank`] on entry and restore it before returning, so the
//! interrupted code resumes with the bank its live token still claims is mapped. An
//! ISR that is not bank-transparent breaks the invariant, just like a raw
//! [`switch_bank`] not paired with a matching restore.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(target_arch = "sm83", feature(asm_experimental_arch))]
#![feature(negative_impls, with_negative_coherence, auto_traits)]
#![feature(fn_traits, unboxed_closures, tuple_trait, const_trait_impl, const_cmp)]

// The runtime model (bank tokens, far pointers, the scope/switch primitives) lives
// in a private module and is re-exported flat; the facade below adds the macros.
mod model;
pub use model::*;

// `#[bank]` lives in the macro namespace; `bank::{main, module, zero}` in the
// module namespace. The two `bank`s coexist (different namespaces).
pub use gb_bank_macros::{bank, far};

/// Soundness regression (hidden from docs, run by `cargo test --doc`): the hidden
/// body a banked fn expands to is an `unsafe fn`, so a *direct* call from safe code
/// is a compile error, while driving it through the wrapper compiles.
///
/// ```compile_fail
/// mod sound { use gb_bank::*; bank::module!(); #[bank] pub fn play(i: u8) -> u8 { i } }
/// fn main() { let _ = sound::__bank_fn_play(0); } // ERROR: call to unsafe function
/// ```
///
/// ```
/// mod sound { use gb_bank::*; bank::module!(); #[bank] pub fn play(i: u8) -> u8 { i } }
/// let _warp = sound::play(0); // ok: the wrapper builds the deferred call
/// ```
///
/// A `#[bank]` function cannot return a value carrying a pointer to its own (banked)
/// code, e.g. a bare `fn` pointer: the [`Output`](Warp::Output) must be [`WarpSafe`],
/// so the caller can never invoke banked code with the bank unmapped.
///
/// ```compile_fail
/// mod m { use gb_bank::*; bank::module!();
///     #[bank] pub fn leak() -> fn(u8) -> u8 { fn inner(x: u8) -> u8 { x } inner }
/// }
/// ```
///
/// ```
/// // The sanctioned carrier is fine: a `Far` / `DynFar` call switches banks first.
/// mod m { use gb_bank::*; bank::module!();
///     #[bank] pub fn add(x: u8) -> u8 { x.wrapping_add(1) }
///     #[bank] pub fn carrier() -> DynFar<fn(u8) -> u8> { far!(add).erase() }
/// }
/// ```
#[doc(hidden)]
pub mod _soundness {}

/// The banking attribute macros, namespaced as `bank::*`.
///
/// [`bank::module!()`](bank::module) declares a bank group,
/// [`#[bank::main]`](macro@bank::main) the bank-0 entry point, and
/// [`#[bank::zero]`](macro@bank::zero) a bank-0 helper. (The bare
/// [`#[bank]`](macro@crate::bank) attribute is a sibling in the macro namespace,
/// not in this module.)
pub mod bank {
    pub use gb_bank_macros::{
        bank_inherit as inherit, bank_main as main, bank_module as module, bank_zero as zero,
    };
}

/// Everything needed to write banked code in one import.
pub mod prelude {
    pub use crate::bank;
    pub use crate::{
        far, scope, Anchor, Bank, DynFar, Far, FarCall, FarWith, Group, GroupZero, Warp,
        WarpSafe,
    };
}
