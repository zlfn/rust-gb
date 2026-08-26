//! Procedural macros for `gb-bank`: compile-time-safe Game Boy ROM banking.
//!
//! These are re-exported by the [`gb-bank`] facade and are meant to be used
//! through it (`use gb_bank::*;`). They turn ordinary-looking functions and
//! statics into bank-safe ones built on `gb-bank`:
//!
//! - [`macro@bank_module`] (`bank::module!()`) declares a module as a bank group.
//! - [`macro@bank`] (`#[bank]`) marks a function or static as living in that bank.
//! - [`macro@bank_zero`] (`#[bank::zero]`) marks bank-0 code that calls
//!   banked functions.
//! - [`macro@far`] (`far!`) takes a far pointer to a banked function for dispatch.
//!
//! The expansions name the `gb-bank` runtime by looking it up in the invoking
//! crate's manifest, so they work whether it depends on `gb-bank` directly or
//! reaches it through a facade such as `gb`.
//!
//! [`gb-bank`]: https://docs.rs/gb-bank

use proc_macro::TokenStream;

mod expand;

/// Declare the enclosing module as a ROM bank group.
///
/// Generates the group brand (`__BankGroup`) that the module's [`#[bank]`](macro@bank)
/// items are anchored to, its `Group` impl, and the link-time bank marker that
/// gb-bank-pack patches. Call it once, at the top of a banked module.
///
/// # Bank assignment
///
/// - `bank::module!()` lets gb-bank-pack **auto-assign** a bank (it bin-packs all
///   auto modules into 16 KiB banks).
/// - `bank::module!(N)` **pins** the module to bank `N` (1 or higher); the packer
///   reserves that bank for it instead of auto-assigning. Pin every module to a
///   distinct bank when you want a deterministic, one-module-per-bank layout (so a
///   call between two modules is always a real cross-bank switch). Bank 0 is the
///   always-mapped region and cannot be pinned to.
///
/// To make a *submodule* share its parent's bank, use `bank::inherit!()`
/// instead of declaring a separate group.
///
/// # Examples
///
/// ```ignore
/// mod sound {
///     use gb_bank::*;
///     bank::module!(3);            // pinned to bank 3
///
///     #[bank]
///     pub fn play(note: u8) -> u8 { note }
/// }
/// ```
#[proc_macro]
pub fn bank_module(input: TokenStream) -> TokenStream {
    expand::bank_module(input.into()).into()
}

/// Declare a submodule as part of its *parent* module's bank group.
///
/// Use `bank::inherit!()` (instead of `bank::module!()`) at the top of a child
/// module so its `#[bank]` items share the parent's bank rather than forming a
/// separate group. (Named `inherit` because `super` is a reserved keyword.)
///
/// ```ignore
/// mod world {
///     use gb_bank::*;
///     bank::module!(4);            // world -> bank 4
///     mod tiles {
///         use gb_bank::*;
///         bank::inherit!();        // tiles joins world -> also bank 4
///         #[bank] pub fn load(i: u8) -> u8 { i }
///     }
/// }
/// ```
#[proc_macro]
pub fn bank_inherit(input: TokenStream) -> TokenStream {
    expand::bank_inherit(input.into()).into()
}

/// Mark a function, static, impl block, or trait as banked.
///
/// Everything banked is *`Warp`-colored*, the same way `async` colors code: a
/// `-> R` becomes `-> impl Warp<Output = R>`, captured by the call and run with
/// `.drive()` (the `Future` / `.await` model). The forms:
///
/// - On a **function** `fn f(a: A) -> R` (generic or not): the body moves to the
///   bank and `f(a)` returns `impl Warp<Output = R>`. Generics are carried onto
///   the wrapper, so each instantiation is banked independently.
/// - On a **static**: the data moves to the bank and a `Far` to it is exposed.
/// - On an **impl block**: every method is rewritten the same way, the receiver
///   becoming the first argument. Call as `value.method(args).drive()`.
/// - On a **trait**: each method signature is colored to `-> impl Warp<..>`
///   (RPITIT), exactly as an `async fn` in a trait desugars. A `#[bank] impl` of
///   that trait then supplies the bodies.
///
/// Inside any banked body the ambient bank token is injected, and `.drive` (run a
/// `Warp`), `.invoke` (a far dispatch), and `.there` (borrow far data) are threaded
/// onto it automatically.
///
/// # Examples
///
/// ```ignore
/// # mod m { use gb_bank::*; bank::module!();
/// #[bank]
/// pub static TABLE: [u8; 4] = [1, 2, 3, 4];
///
/// #[bank]
/// pub fn play(note: u8) -> u8 {
///     let base = TABLE.there(|t| t[0]);   // borrow banked data
///     note.wrapping_add(base)
/// }
///
/// pub struct Point { pub x: u8, pub y: u8 }
///
/// #[bank]
/// impl Point {
///     pub fn shift(&self, dx: u8) -> u8 { self.x.wrapping_add(dx) }
/// }
/// // call site: `p.shift(3).drive()`
/// # }
/// ```
#[proc_macro_attribute]
pub fn bank(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand::bank_attr(attr.into(), item.into()).into()
}

/// Mark the **entry point** (bank 0, the root of the call stack).
///
/// The runtime calls the entry with no token, so it assumes a `GroupZero` token
/// and drives banked code directly. Because it is the root, leaving a bank mapped
/// after a call is harmless (the next call switches again).
///
/// It expands to `#[gb_rt::entry]` around the rewritten body, so do not apply that
/// attribute as well, and give the function the signature `gb-rt` requires:
/// `fn() -> !`.
///
/// # Examples
///
/// ```ignore
/// # use gb_bank::*;
/// #[bank::main]
/// pub fn main() -> ! {
///     sound::play(60).drive();    // banked function: driven with `.drive()`
///     update(0).drive();          // bank-0 helper: also `.drive()`
///     loop {}
/// }
/// ```
#[proc_macro_attribute]
pub fn bank_main(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand::resident(attr.into(), item.into()).into()
}

/// Mark a bank-0 **helper** that drives banked code.
///
/// Unlike the entry point, a helper may be called from inside any bank, so it
/// cannot assume `GroupZero`: it must thread whatever token its caller holds and
/// restore that bank. So `#[bank::zero]` is `Warp`-colored like `#[bank]` (called
/// `helper(args).drive()`), but its `Warp` performs *no* bank switch, just
/// forwards the caller's token. Any banked call inside then switches `C ->
/// target -> C`, leaving the caller's bank exactly as it found it.
///
/// Works on a `fn` (incl. generics), an `impl` block (each method becomes such a
/// helper, `value.method(args).drive()`), or a `trait` (its signatures are colored
/// `-> impl Warp`, like a banked trait; a `#[bank::zero] impl` supplies bodies).
///
/// # Examples
///
/// ```ignore
/// # use gb_bank::*;
/// #[bank::zero]
/// pub fn update(state: u8) -> u8 {
///     enemy::ai(state).drive()    // banked call inside; restores the caller's bank
/// }
/// // from anywhere with a bank token: `update(0).drive()`
/// ```
#[proc_macro_attribute]
pub fn bank_zero(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand::resident_fixed(attr.into(), item.into()).into()
}

/// Take a far pointer to a banked function, for heterogeneous dispatch.
///
/// `far!(path::to::f)` yields a `Far` to the banked function `f` (resolved
/// through the module's group). Call `.erase()` on it to drop the static group
/// and collect functions from *different* banks into a single `[DynFar<..>; N]`
/// table, then dispatch with runtime arguments.
///
/// # Examples
///
/// ```ignore
/// # use gb_bank::*;
/// # fn f(bank: &mut Bank<GroupZero>) {
/// let table: [DynFar<fn(u8)>; 2] = [
///     far!(sound::play).erase(),
///     far!(enemy::ai).erase(),
/// ];
/// for entry in &table {
///     entry.invoke(bank, (level,));   // switch to each one's bank, run, restore
/// }
/// # }
/// ```
#[proc_macro]
pub fn far(input: TokenStream) -> TokenStream {
    expand::far(input.into()).into()
}
