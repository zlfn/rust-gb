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

use super::{DynFar, Bank, Far, FarCall, Group, GroupZero, switch_run, switch_run_far};

pub use gb_bank_safe::BankSafe;

// Far pointers are the sanctioned way to carry a banked function out of its bank:
// calling one switches banks first, so a `Far` / `DynFar` is `BankSafe` regardless of
// what it points at. (This also stops the auto-derivation from looking through the
// inner `*const T` at a banked `fn`.)
unsafe impl<T: ?Sized, G: Group> BankSafe for Far<T, G> {}
unsafe impl<T: ?Sized> BankSafe for DynFar<T> {}

/// A deferred banked call: a function applied to its arguments, not yet run.
///
/// This is the [`Future`] of the banking world. A `#[bank]` function returns
/// `impl Warp<Output = R>`; [`drive`](Warp::drive) is the `.await` that drives it,
/// performing the bank switch, running the function, and restoring the caller.
///
/// Implemented by [`BankedWarp`] (what the macro builds) and, transitively, by any
/// function that returns one, so a banked function value is itself a [`FarCall`].
///
/// The attribute sits here rather than on [`BankedWarp`] / [`FixedWarp`] because a
/// banked function returns `impl Warp`, and that lint reads the trait.
#[must_use = "a Warp does nothing until `.drive()`"]
pub trait Warp {
    /// The value the call produces. It must be [`BankSafe`]: a banked call's result
    /// crosses the switch back to the caller, so it cannot embed a pointer to the
    /// callee's (now unmapped) banked code.
    type Output: BankSafe;

    /// The bank group this call targets (where its function lives).
    type Group: Group;

    /// Run the call in its own bank, given a token already there: the same-bank
    /// "near call". No switch happens, `here` proves the bank is mapped.
    ///
    /// This is the primitive; [`drive`](Warp::drive) is `near` wrapped in a
    /// [`scope`](crate::scope). Because it takes a `Bank<Self::Group>`, it pins the token's
    /// group: a run of `near` calls share one [`scope`](crate::scope), and `scope` over them
    /// infers the token's group with no annotation. (For data, see
    /// [`local`](crate::Far::local).)
    fn near(self, here: &mut Bank<Self::Group>) -> Self::Output;

    /// Run the call from any bank, leaving the caller's bank `C` as it was: the
    /// cross-bank counterpart of [`near`](Warp::near). A banked call switches into its
    /// bank and back (elided when `C` is already the target, or the target is
    /// [`GroupZero`]); a bank-0 helper instead forwards `C` with no switch.
    ///
    /// A resident caller inlines the switch, which lets the `Warp` collapse into a
    /// plain call. A banked caller cannot: inlined switch code would unmap itself, so
    /// it goes through a bank-0 trampoline instead.
    #[inline]
    fn drive<C: Group>(self, outer: &mut Bank<C>) -> Self::Output
    where
        Self: Sized,
    {
        let run = |b: &mut Bank<Self::Group>| self.near(b);
        if C::FIXED {
            switch_run(outer, run)
        } else {
            switch_run_far(outer, run)
        }
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

impl<F: Fn<Args>, G: Group, Args: Tuple + BankSafe> Warp for BankedWarp<F, G, Args>
where
    F::Output: BankSafe,
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
impl<T, Args: Tuple + BankSafe> FarCall<Args> for T
where
    T: Fn<Args>,
    T::Output: Warp,
{
    type Output = <T::Output as Warp>::Output;
    #[inline]
    fn invoke<C: Group>(&self, outer: &mut Bank<C>, args: Args) -> Self::Output {
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
pub trait FixedFn<Args: BankSafe> {
    /// The value the call produces. [`BankSafe`] for the same reason as
    /// [`Warp::Output`]: a bank-0 helper's result is handed back to the caller.
    type Output: BankSafe;

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

impl<M: FixedFn<Args>, Args: BankSafe> Warp for FixedWarp<M, Args> {
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
