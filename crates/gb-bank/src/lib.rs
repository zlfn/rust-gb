//! Compile-time-safe ROM bank switching for the Game Boy.
//!
//! `gb-bank` is the user-facing front end of the banking toolchain: the runtime
//! types, plus the macros that turn ordinary functions and statics into bank-safe
//! ones.
//!
//! # The problem
//!
//! A Game Boy cartridge maps one 16 KiB ROM **bank** at a time into the
//! `0x4000..0x8000` window. Anything in another bank is unaddressable until it is
//! switched in, and reading it anyway yields whichever bytes happen to be mapped,
//! with nothing to fault on the way.
//!
//! This crate makes that a type error. Reaching banked code or data requires a
//! token proving its bank is mapped, and the only way to get one is to perform the
//! switch.
//!
//! # Getting started
//!
//! ## Put code in a bank
//!
//! One module is one bank group. `bank::module!()` declares it, `#[bank]` marks
//! what goes in the bank.
//!
//! ```ignore
//! mod sound {
//!     use gb_bank::*;
//!     bank::module!();
//!
//!     #[bank]
//!     pub static NOTES: [u8; 4] = [60, 62, 64, 65];
//!
//!     #[bank]
//!     pub fn play(i: u8) -> u8 {
//!         NOTES.local()[i as usize]
//!     }
//! }
//! ```
//!
//! `gb-bank-pack` decides at link time which ROM bank the module lands in. Naming
//! one yourself is optional (see [Bank layout](#bank-layout)).
//!
//! ## Call it
//!
//! Your entry point is `#[bank::main]`, which lives in bank 0 and is always mapped.
//! It stands in for `#[gb_rt::entry]`, so the signature is the one that macro
//! wants, `fn() -> !`.
//!
//! ```ignore
//! #[bank::main]
//! pub fn main() -> ! {
//!     loop {
//!         let note = sound::play(0).drive();
//!     }
//! }
//! ```
//!
//! `sound::play(0)` does not run the body. It captures the call, and `.drive()`
//! runs it: switch into sound's bank, call, switch back.
//!
//! ## Read its data
//!
//! `#[bank] static` gives you a [`Far`] pointer. Holding one is always fine;
//! reading through it is what needs the bank mapped.
//!
//! ```ignore
//! #[bank::main]
//! pub fn main() -> ! {
//!     let first = sound::NOTES.there(|n| n[0]);
//!     loop {}
//! }
//! ```
//!
//! `.there()` switches, lends your closure `&[u8; 4]`, and switches back. Whatever
//! the closure returns is copied out, so it must not be a pointer into the bank you
//! just left; that is what [`BankSafe`] checks.
//!
//! ## Inside a banked function
//!
//! A `#[bank]` body has one restriction, and it comes from the hardware: its own
//! code sits in the switchable window, so it cannot perform a switch. The next
//! instruction fetch would come from whatever bank replaced it.
//!
//! Calls are fine, because the switch happens in a bank-0 trampoline rather than in
//! your function:
//!
//! ```ignore
//! #[bank]
//! pub fn tick() -> u8 {
//!     NOTES.local()[0]            // same bank: no switch at all
//!         + other::helper().drive() // another bank: switches in bank 0
//! }
//! ```
//!
//! `.there()` and [`scope`] are not, because they run *your* closure while the
//! other bank is mapped. Both are a compile error in a `#[bank]` body. Route them
//! through a `#[bank::zero]` helper, which lives in bank 0 and can be called from
//! any bank:
//!
//! ```ignore
//! #[bank::zero]
//! fn notes() -> [u8; 4] {
//!     sound::NOTES.there(|n| *n)   // bank 0: allowed
//! }
//!
//! #[bank]
//! pub fn tick() -> u8 {
//!     let n = notes().drive();     // copied out; this bank is mapped again
//!     n[0].wrapping_add(n[3])
//! }
//! ```
//!
//! ## Runtime dispatch
//!
//! A [`Far<T, G>`](Far) carries its bank in the type, so pointers into different
//! banks are different types. [`erase`](Far::erase) drops the type for a runtime
//! number, letting a table hold entries from several banks.
//!
//! ```ignore
//! let table: [DynFar<fn(u8) -> u8>; 2] = [
//!     far!(enemy::ai).erase(),
//!     far!(hud_tick).erase(),      // a bank-0 helper works here too
//! ];
//! table[i].invoke(state)
//! ```
//!
//! # Where the switch happens
//!
//! Six operations reach banked code or data. Which one you can use depends on where
//! you are; what it costs depends on whether it switches. The ones that do switch
//! restore the caller's bank on the way out.
//!
//! | | switches | usable from |
//! |---|---|---|
//! | [`local`](Far::local) | no | any body, with a token for that bank |
//! | [`near`](Warp::near) | no | any body, with a token for that bank |
//! | [`drive`](Warp::drive) | yes | any body |
//! | [`invoke`](FarCall::invoke) | yes | any body |
//! | [`there`](FarWith::there) | yes | bank 0 only |
//! | [`scope`] | yes | bank 0 only |
//!
//! `drive` and `invoke` run one whole function across the switch, so no code of
//! yours is unmapped while it happens and they compile to a bank-0 trampoline.
//! `there` and `scope` lend your closure instead, which is why they need to be in
//! bank 0 to begin with.
//!
//! `local` and `near` take a token for the exact group, so they never switch, and
//! reaching the wrong bank through one is a type error. Use them to batch: open one
//! [`scope`] and work inside it.
//!
//! ```ignore
//! let r = scope(|b| {
//!     let x = sound::play(v).near(b);   // already in sound's bank
//!     sound::play(x).near(b)            // still there: no second switch
//! });
//! ```
//!
//! A switch is also elided when it would be a no-op: when the target group is the
//! caller's own, and when either side is the always-mapped [`GroupZero`].
//!
//! # The model
//!
//! A [`Group`] is the compile-time identity of one bank, a zero-sized type that
//! `bank::module!()` generates per module. A [`Bank<G>`](Bank) is a zero-sized
//! token, and holding one witnesses that `G`'s bank is mapped right now. The token
//! is `!Send`, not `Clone`, and mintable only through [`scope`] or the unsafe
//! [`assume`](Bank::assume), so it cannot be fabricated.
//!
//! [`scope`] is the switch primitive everything else is built on. It also takes an
//! [`Anchor`]: a witness that the *calling* code is in bank 0, which is what makes
//! it safe to run a closure while another bank is mapped. `#[bank::main]` and
//! `#[bank::zero]` bodies hold one; a `#[bank]` body does not.
//!
//! A [`Far<T, G>`](Far) splits the address from the access. The address is a plain
//! [`Copy`] value that survives any switch; reading it borrows a [`Bank<G>`](Bank).
//! Since a switch needs that token by `&mut`, a reference into a bank cannot outlive
//! the switch away from it.
//!
//! A [`Warp`] is a call captured but not yet run, so that the switch can take the
//! *caller's* token at the point it is driven.
//!
//! ## Prior art
//!
//! [`Bank`] and [`Far`] follow [GhostCell], which keeps a permission token apart
//! from the data it guards, tied by a brand. gb-bank brands with a group *type*
//! where GhostCell uses a lifetime, because a bank's identity is fixed and reused
//! across many functions, which a per-scope lifetime cannot express.
//!
//! [`Warp`] follows [`Future`]: an `async fn` returns something inert until
//! `.await`, and a banked call is deferred for the same reason, except that
//! [`drive`](Warp::drive) takes a token rather than an executor.
//!
//! [GhostCell]: https://plv.mpi-sws.org/rustbelt/ghostcell/
//!
//! # The macros
//!
//! - [`bank::module!()`](bank::module) declares the enclosing module as a bank
//!   group. `bank::module!(N)` pins it to bank `N` instead of auto-assigning.
//! - [`bank::inherit!()`](bank::inherit) in a submodule folds it into its *parent*
//!   module's group (`super` is a keyword, hence the name).
//! - [`#[bank]`](macro@bank) on a `fn` rewrites it to return `impl Warp`; on a
//!   `static` it exposes a [`Far`]. Also works on an `impl` or `trait`. The return
//!   type must be [`BankSafe`].
//! - [`#[bank::main]`](bank::main) marks the entry point in bank 0. It wraps
//!   `#[gb_rt::entry]`, so that attribute must not be applied as well.
//! - [`#[bank::zero]`](bank::zero) marks a bank-0 helper callable from any bank: it
//!   forwards the caller's token, so its own banked calls restore the caller's bank.
//! - [`far!`](macro@far) takes a [`Far`] to a banked function for dispatch tables.
//!   It works on a `#[bank::zero]` helper too, so a table can mix the two.
//!
//! ## Sugar
//!
//! Inside any `#[bank]` / `#[bank::main]` / `#[bank::zero]` body the macro injects
//! the ambient bank token (and, in a bank-0 body, an [`Anchor`]) as implicit leading
//! arguments, so they never appear in your code:
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
//! A [`scope`] rebinds the ambient token to its own closure parameter `b`, so a
//! nested `scope` or a `.drive()` / `.near()` / `.local()` *inside* it threads `b`,
//! not the outer token; the `Anchor`, being [`Copy`], flows in automatically. The
//! ambient `__bank` is hidden and cannot be named, but `b` can.
//!
//! ### Threading is by method name, not type
//!
//! The rewrite runs before type checking, so it matches on the method *name* alone
//! and threads the token into every `.drive()` / `.near()` / `.local()` /
//! `.invoke()` / `.there(..)` (and `scope(..)`) in the body, whatever the receiver.
//! The names are deliberately uncommon. If one does collide with an unrelated
//! method, call that one as `Type::method(recv, args)`: a path call is not
//! method-call sugar, so the macro leaves it alone.
//!
//! ## Bank layout
//!
//! By default gb-bank-pack bin-packs the banked modules into 16 KiB banks. Pin each
//! module with `bank::module!(N)` for a deterministic one-module-per-bank layout, so
//! a call from one module to another is always a genuine switch.
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
//! Bank 0 is the always-mapped region and cannot be pinned to. With no MBC at all
//! the whole 32 KiB is fixed and there is no switching.
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
//!
//! # Cartridges
//!
//! Which banks exist at all is the cartridge's business. `cargo-gb` reads the type
//! from `header.toml` and refuses a build that needs more banks than it can map.
//!
//! | Cartridge | Switchable banks | ROM | with `wide_banks = true` |
//! |---|---|---|---|
//! | ROM ONLY | none, the 32 KiB is flat | 32 KiB | |
//! | MBC1 | 1-31 | 512 KiB | 1-127 minus `0x20`, `0x40`, `0x60`, 2 MiB |
//! | MBC2 | 1-15 | 256 KiB | |
//! | MBC3 | 1-127 | 2 MiB | |
//! | MBC5 | 1-255 | 4 MiB | 1-511, 8 MiB |
//! | MBC7 | 1-127 | 2 MiB | |
//!
//! The ranges without `wide_banks` are what a single write to the register at
//! `0x2000` selects, and that is all the runtime writes by default. `wide_banks`
//! brings in the cartridge's second bank register, which only MBC1 and MBC5 have;
//! see [`BankNumber`] for what that costs. The three banks MBC1 has to skip are the
//! ones whose low five bits are zero, which it reads as bank 1, and its upper bits
//! only reach the ROM in banking mode 0, so selecting mode 1 for RAM banking gives
//! up banks 32 and above.
//!
//! MBC6, MMM01, HuC1, HuC3, and the Pocket Camera and TAMA5 mappers are not built
//! in for now. A custom cartridge that selects banks its own way implements
//! [`Mapper`] and names it with [`set_mapper!`].
//!

#![no_std]
#![feature(asm_experimental_arch)]
#![feature(negative_impls)]
#![feature(fn_traits, unboxed_closures, tuple_trait, const_trait_impl, const_cmp)]

// The runtime model (bank tokens, far pointers, the scope/switch primitives) lives
// in a private module and is re-exported flat; the facade below adds the macros.
mod model;
pub use model::*;

// `#[bank]` lives in the macro namespace; `bank::{main, module, zero}` in the
// module namespace. The two `bank`s coexist (different namespaces).
pub use gb_bank_macros::{bank, far};

/// The types the macros name in their expansion. Not an API.
#[doc(hidden)]
pub mod __private {
    pub use crate::model::warp::{BankedWarp, FixedFn, FixedWarp};
}


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

/// The [`mod@bank`] attributes, also at the root.
///
/// A facade re-exporting this crate as its own `bank` module then reaches them as
/// `bank::main` rather than `bank::bank::main`.
pub use gb_bank_macros::{
    bank_inherit as inherit, bank_main as main, bank_module as module, bank_zero as zero,
};

/// Everything needed to write banked code in one import.
pub mod prelude {
    pub use crate::bank;
    pub use crate::{
        far, scope, Anchor, Bank, DynFar, Far, FarCall, FarWith, Group, GroupZero, Warp,
        BankSafe,
    };
}
