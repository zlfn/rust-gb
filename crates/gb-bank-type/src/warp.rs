//! [`Warp`]: a deferred banked call, the [`Future`] analog of this crate.
//!
//! A `#[bank]` function does not run when you name it with arguments; like an
//! `async fn`, calling it just *captures* the work and hands back a value
//! implementing [`Warp`]. [`Warp::drive`] is the `.await`: it performs the bank
//! switch, runs the function, and restores the caller's bank.
//!
//! The concrete value returned is a [`BankedWarp`] (the analog of an `async fn`'s
//! anonymous state machine), but callers only ever see `impl Warp<Output = R>`.

use core::marker::{PhantomData, Tuple};

use crate::{DynFar, Bank, Far, FarCall, Group, GroupZero, switch_run};

/// A value safe to carry across a [`Warp`] (bank-switch) boundary: it embeds no
/// pointer to banked code that the switch would unmap.
///
/// This is the bank-switching analog of the standard library's `std::panic::UnwindSafe`,
/// and like it an *auto trait*: a type is `WarpSafe` unless it contains something that
/// is not, so the property propagates through structs, tuples, and enums.
///
/// A [`Warp`]'s [`Output`](Warp::Output) must be `WarpSafe`, so a `#[bank]` function
/// cannot hand back a bare `fn` pointer, a `dyn` trait object, or a value containing
/// one: the caller would invoke it with the callee's bank no longer mapped. Banked
/// functions carried as [`Far`] / [`DynFar`] (via `far!`) are exempt, because calling
/// one switches banks first.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not `WarpSafe`: it carries a pointer to banked code",
    label = "cannot cross a bank switch",
    note = "a `#[bank]` function cannot return a bare `fn` pointer, a `dyn` trait object, \
            or a value containing one: the caller would run it with the bank unmapped",
    note = "carry banked functions as `Far` / `DynFar` (built by `far!`) instead, \
            whose call switches banks first"
)]
pub auto trait WarpSafe {}

// Far pointers are the sanctioned way to carry a banked function out of its bank:
// calling one switches banks first, so a `Far` / `DynFar` is `WarpSafe` regardless of
// what it points at. (This also stops the auto-derivation from looking through the
// inner `*const T` at a banked `fn`.)
impl<T: ?Sized, G: Group> WarpSafe for Far<T, G> {}
impl<T: ?Sized> WarpSafe for DynFar<T> {}

// A bare `fn` pointer is *not* `WarpSafe`: calling it performs no bank switch, so if
// it targets banked code the caller runs it with that bank unmapped. One impl per
// arity, as the standard library does for the `Fn` family.
macro_rules! not_warp_safe_fn {
    ($($arg:ident),*) => {
        impl<Ret, $($arg),*> !WarpSafe for fn($($arg),*) -> Ret {}
    };
}
not_warp_safe_fn!();
not_warp_safe_fn!(A);
not_warp_safe_fn!(A, B);
not_warp_safe_fn!(A, B, C);
not_warp_safe_fn!(A, B, C, D);
not_warp_safe_fn!(A, B, C, D, E);
not_warp_safe_fn!(A, B, C, D, E, F);
not_warp_safe_fn!(A, B, C, D, E, F, G);
not_warp_safe_fn!(A, B, C, D, E, F, G, H);
not_warp_safe_fn!(A, B, C, D, E, F, G, H, I);
not_warp_safe_fn!(A, B, C, D, E, F, G, H, I, J);
not_warp_safe_fn!(A, B, C, D, E, F, G, H, I, J, K);
not_warp_safe_fn!(A, B, C, D, E, F, G, H, I, J, K, L);

/// A deferred banked call: a function applied to its arguments, not yet run.
///
/// This is the [`Future`] of the banking world. A `#[bank]` function returns
/// `impl Warp<Output = R>`; [`drive`](Warp::drive) is the `.await` that drives it,
/// performing the bank switch, running the function, and restoring the caller.
///
/// Implemented by [`BankedWarp`] (what the macro builds) and, transitively, by any
/// function that returns one, so a banked function value is itself a [`FarCall`].
pub trait Warp {
    /// The value the call produces. It must be [`WarpSafe`]: a banked call's result
    /// crosses the switch back to the caller, so it cannot embed a pointer to the
    /// callee's (now unmapped) banked code.
    type Output: WarpSafe;

    /// The bank group this call targets (where its function lives).
    type Group: Group;

    /// Run the call in its own bank, given a token already there: the same-bank
    /// "near call". No switch happens, `here` proves the bank is mapped.
    ///
    /// This is the primitive; [`drive`](Warp::drive) is `near` wrapped in a
    /// [`scope`](crate::scope). Because it takes a `Bank<Self::Group>`, it pins the token's
    /// group: a run of `near` calls share one [`scope`](crate::scope), and `scope` over them
    /// infers the token's group with no annotation. (For data, see
    /// [`get`](crate::Far::get).)
    fn near(self, here: &mut Bank<Self::Group>) -> Self::Output;

    /// Run the call from any bank, leaving the caller's bank `C` as it was: the
    /// cross-bank counterpart of [`near`](Warp::near). A banked call switches into its
    /// bank and back (elided when `C` is already the target, or the target is
    /// [`GroupZero`]); a bank-0 helper instead forwards `C` with no switch. Being
    /// `#[inline(never)]`, the switch lives in a bank-0 trampoline, so this is safe to
    /// drive even from a banked caller.
    #[inline(never)]
    fn drive<C: Group>(self, outer: &mut Bank<C>) -> Self::Output
    where
        Self: Sized,
    {
        switch_run(outer, |b: &mut Bank<Self::Group>| self.near(b))
    }
}

/// The concrete deferred call: a function pointer with its arguments captured.
///
/// The analog of the anonymous state machine an `async fn` returns. You normally
/// see it only as `impl Warp`; the `#[bank]` macro constructs it.
///
/// Like a [`Future`], it is inert if dropped without being run (`#[must_use]`).
#[must_use = "a Warp does nothing until `.drive()`"]
pub struct BankedWarp<F, G, Args> {
    f: F,
    args: Args,
    _g: PhantomData<G>,
}

impl<F, G, Args> BankedWarp<F, G, Args> {
    /// Capture a function and its arguments. Emitted by the `#[bank]` macro.
    ///
    /// # Safety
    ///
    /// `f` must be a function living in group `G`'s bank, so that the switch
    /// performed by [`Warp::drive`] maps the bank its code actually resides in.
    #[doc(hidden)]
    #[inline(always)]
    pub const unsafe fn new(f: F, args: Args) -> Self {
        BankedWarp { f, args, _g: PhantomData }
    }
}

impl<F: Fn<Args>, G: Group, Args: Tuple> Warp for BankedWarp<F, G, Args>
where
    F::Output: WarpSafe,
{
    type Output = F::Output;
    type Group = G;
    #[inline]
    fn near(self, _here: &mut Bank<G>) -> F::Output {
        // `_here` proves G is mapped; just run the function (no switch). `drive`
        // is the default (scope into G, then `near`).
        self.f.call(self.args)
    }
}

// `Far`/`DynFar` are not callable functions; stating it lets the blanket impl
// below (for Warp-returning functions) coexist with their own `FarCall` impls
// under `with_negative_coherence`.
impl<F, G, Args: Tuple> !Fn<Args> for Far<F, G> {}
impl<F, Args: Tuple> !Fn<Args> for DynFar<F> {}

/// A [`Warp`]-returning function is itself a [`FarCall`], usable as a dispatch
/// target like a [`Far`] / [`DynFar`].
impl<T, Args: Tuple> FarCall<Args> for T
where
    T: Fn<Args>,
    T::Output: Warp,
{
    type Output = <T::Output as Warp>::Output;
    #[inline]
    fn call<C: Group>(&self, outer: &mut Bank<C>, args: Args) -> Self::Output {
        Fn::call(self, args).drive(outer)
    }
}

/// The body of a bank-0 (`#[bank::zero]`) function, generic over the *caller's*
/// bank group `C`.
///
/// A bank-0 function lives in the always-mapped region, so reaching it needs no
/// bank switch. But when it drives banked calls of its own, each one must switch
/// back to whatever bank the *caller* was in, otherwise the caller resumes with
/// the wrong bank mapped. So the body has to be parametric over the caller's
/// group `C`, threading a [`Bank<C>`](Bank) token through its `.drive()`s.
///
/// A function pointer cannot be generic over a *type* (`for<C: Group> fn(..)` is
/// not expressible), so the body is encoded as this trait instead, implemented on
/// a per-function marker type by the `#[bank::zero]` macro. The generic lives on
/// the [`run`](FixedFn::run) *method*, which is allowed. [`FixedWarp`] is the
/// [`Warp`] that invokes it.
pub trait FixedFn<Args> {
    /// The value the call produces. [`WarpSafe`] for the same reason as
    /// [`Warp::Output`]: a bank-0 helper's result is handed back to the caller.
    type Output: WarpSafe;

    /// Run the body in the caller's bank `C`, threading its `bank` token so that
    /// any banked call inside restores `C` on the way out.
    fn run<C: Group>(args: Args, bank: &mut Bank<C>) -> Self::Output;
}

/// The concrete [`Warp`] a bank-0 (`#[bank::zero]`) function returns.
///
/// Where a [`BankedWarp`] switches into a *fixed* group `G`, a `FixedWarp` performs
/// no switch at all: its body is in bank 0 and always reachable, so it just
/// threads the caller's token straight through. Any banked call inside the body
/// then switches `C -> target -> C` and leaves the caller's bank exactly as it
/// found it. That is what makes a bank-0 helper safe to call from *any* bank,
/// not only from bank 0.
///
/// `M` is a per-function marker carrying the body via its [`FixedFn`] impl; the
/// macro builds it. Callers only ever see `impl Warp<Output = R>` and drive it
/// with `helper(args).drive()`. Like any [`Warp`], it is inert until run
/// (`#[must_use]`).
#[must_use = "a Warp does nothing until `.drive()`"]
pub struct FixedWarp<M, Args> {
    args: Args,
    _m: PhantomData<M>,
}

impl<M, Args> FixedWarp<M, Args> {
    /// Capture the arguments. Emitted by the `#[bank::zero]` macro.
    ///
    /// Safe: the [`Warp`] impl below requires `M: FixedFn<Args>`, so a marker and
    /// its argument tuple can never be mismatched.
    #[doc(hidden)]
    #[inline(always)]
    pub const fn new(args: Args) -> Self {
        FixedWarp { args, _m: PhantomData }
    }
}

impl<M: FixedFn<Args>, Args> Warp for FixedWarp<M, Args> {
    type Output = M::Output;
    // A bank-0 helper is caller-generic; anchor it at bank 0 so the
    // `Warp` contract is satisfied. Resident helpers are normally run with `drive`.
    type Group = GroupZero;
    #[inline]
    fn near(self, here: &mut Bank<GroupZero>) -> M::Output {
        M::run(self.args, here)
    }
    // Override the default `drive`: a bank-0 helper forwards the *actual* caller
    // `C` unchanged (so its inner calls restore `C`) rather than scoping to
    // GroupZero. Resident code is always mapped, so there is no switch to make.
    #[inline]
    fn drive<C: Group>(self, outer: &mut Bank<C>) -> M::Output {
        M::run(self.args, outer)
    }
}
