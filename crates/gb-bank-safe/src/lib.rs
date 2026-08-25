//! The [`BankSafe`] marker.
//!
//! It lives in its own crate so that `gb-bank`'s own dependencies can bound their
//! APIs on it. Use it through `gb_bank::BankSafe`, which re-exports it.

#![no_std]
#![feature(auto_traits, negative_impls)]

/// A value safe to carry across a bank switch: it embeds no pointer to banked code
/// that the switch would unmap.
///
/// This is the bank-switching analog of the standard library's `std::panic::UnwindSafe`,
/// and like it an *auto trait*: a type is `BankSafe` unless it contains something that
/// is not, so the property propagates through structs, tuples, and enums. A generic
/// type parameter that reaches a switch therefore needs an explicit `BankSafe` bound,
/// the same way crossing a thread boundary needs `Send`.
///
/// # What is not `BankSafe`
///
/// A bare `fn` pointer and a `dyn` trait object, because calling either performs no
/// bank switch: if it targets banked code, the caller runs it with that bank unmapped.
/// `Far` and `DynFar` are the sanctioned carriers and are exempt, because calling
/// one switches banks first (and costs nothing when the target is bank 0).
///
/// `Anchor` is excluded for a different reason: it witnesses that the current code
/// is in bank 0, which reaching a banked callee would falsify.
///
/// # Where it is enforced
///
/// Every path that crosses a switch requires it on the values that cross:
/// `Warp` and `FarCall` on both their arguments and their output, `scope` and
/// `FarWith::there` on what the closure returns.
///
/// # Examples
///
/// Handing a banked function pointer back to the caller is rejected:
///
/// ```compile_fail
/// #[bank]
/// pub fn pick() -> fn() {
///     fn helper() { /* lives in this bank */ }
///     helper // ERROR: `fn()` is not `BankSafe`
/// }
/// ```
///
/// So is passing one into another bank, where it would no longer be mapped:
///
/// ```compile_fail
/// #[bank]
/// pub fn register(tick: fn()) { // ERROR: `fn()` is not `BankSafe`
///     tick()
/// }
/// ```
///
/// Carry the function as a `Far` instead; calling it maps its bank first:
///
/// ```ignore
/// #[bank]
/// fn pick() -> Far<fn(), Sound> {
///     far!(sound::helper)
/// }
/// ```
///
/// A generic banked item needs the bound on any parameter that crosses:
///
/// ```ignore
/// #[bank]
/// impl<A: Copy + BankSafe> Summary for Pair<A> {
///     fn summarize(&self) -> u8 { /* `&Pair<A>` crosses as the receiver */ }
/// }
/// ```
///
/// # Globals are not covered
///
/// The bound only reaches values that pass through an API. A banked function may
/// still park a pointer to its own code in a global and leave it there after the
/// switch, with no `unsafe` anywhere:
///
/// ```ignore
/// static CB: Mutex<Cell<Option<fn()>>> = Mutex::new(Cell::new(None));
///
/// #[bank]
/// pub fn install(cs: CriticalSection) {
///     fn helper() { /* lives in this bank */ }
///     CB.borrow(cs).set(Some(helper)); // accepted
/// }
///
/// // elsewhere, once this bank is no longer mapped
/// critical_section::with(|cs| CB.borrow(cs).get().unwrap()()); // undefined behaviour
/// ```
///
/// Calling such a pointer runs whatever bytes the currently mapped bank holds at
/// that address. Unlike `UnwindSafe`, whose violations only expose inconsistent
/// state, a violation here can execute arbitrary code.
///
/// A crate that targets the Game Boy and hands out a place to keep global state
/// should bound what goes in it on this trait. General-purpose wrappers cannot, for
/// example `critical_section::Mutex`, so a banked code pointer still reaches a
/// global through any of them.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be carried across a bank switch",
    label = "cannot cross a bank switch",
    note = "a bare `fn` pointer, a `dyn` trait object, or a value containing one cannot \
            cross a bank switch: it would be called with its bank unmapped",
    note = "carry banked functions as `Far` / `DynFar` (built by `far!`) instead, \
            whose call switches banks first"
)]
pub unsafe auto trait BankSafe {}

// A bare `fn` pointer is *not* `BankSafe`: calling it performs no bank switch, so if
// it targets banked code the caller runs it with that bank unmapped. One impl per
// arity and safety/ABI combination, as the standard library does for the `Fn` family.
macro_rules! not_bank_safe_fn {
    ($($arg:ident),*) => {
        impl<Ret, $($arg),*> !BankSafe for fn($($arg),*) -> Ret {}
        impl<Ret, $($arg),*> !BankSafe for unsafe fn($($arg),*) -> Ret {}
        impl<Ret, $($arg),*> !BankSafe for extern "C" fn($($arg),*) -> Ret {}
        impl<Ret, $($arg),*> !BankSafe for unsafe extern "C" fn($($arg),*) -> Ret {}
    };
}
not_bank_safe_fn!();
not_bank_safe_fn!(A);
not_bank_safe_fn!(A, B);
not_bank_safe_fn!(A, B, C);
not_bank_safe_fn!(A, B, C, D);
not_bank_safe_fn!(A, B, C, D, E);
not_bank_safe_fn!(A, B, C, D, E, F);
not_bank_safe_fn!(A, B, C, D, E, F, G);
not_bank_safe_fn!(A, B, C, D, E, F, G, H);
not_bank_safe_fn!(A, B, C, D, E, F, G, H, I);
not_bank_safe_fn!(A, B, C, D, E, F, G, H, I, J);
not_bank_safe_fn!(A, B, C, D, E, F, G, H, I, J, K);
not_bank_safe_fn!(A, B, C, D, E, F, G, H, I, J, K, L);
