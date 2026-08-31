//! Expansion logic for the `gb-bank` macros.

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::visit_mut::{self, VisitMut};
use syn::{
    parse2, parse_quote, Expr, FnArg, ImplItem, ImplItemFn, ItemFn, ItemImpl, ItemStatic,
    ItemTrait, Pat, Path, ReturnType, Stmt, TraitItem, Type,
};

/// The identifier an invoking crate reaches a dependency by.
///
/// `crate_name` answers with the package name where the dependency is not
/// renamed, and the HAL publishes as `rust-gb` while its library is `gb`.
fn extern_name(found: String) -> String {
    match found.as_str() {
        "rust-gb" | "rust_gb" => "gb".to_string(),
        _ => found.replace('-', "_"),
    }
}

/// Path to a runtime crate as the *invoking* crate can name it: directly when it
/// depends on that crate, otherwise through the `gb` facade, which re-exports the
/// runtime crates at `facade` for exactly this purpose. Falling back to the plain
/// name when neither is present lets the usual unresolved-import error name the
/// crate the user is missing.
fn runtime_root(krate: &str, facade: &str) -> TokenStream {
    use proc_macro_crate::{crate_name, FoundCrate};
    let path = match crate_name(krate) {
        Ok(FoundCrate::Itself) => "crate".to_string(),
        Ok(FoundCrate::Name(n)) => format!("::{}", extern_name(n)),
        Err(_) => match crate_name("rust-gb").or_else(|_| crate_name("gb")) {
            Ok(FoundCrate::Itself) => format!("crate::{facade}"),
            Ok(FoundCrate::Name(n)) => format!("::{}::{facade}", extern_name(n)),
            Err(_) => format!("::{}", krate.replace('-', "_")),
        },
    };
    path.parse().expect("runtime root path")
}

fn bank_root() -> TokenStream {
    runtime_root("gb-bank", "__bank")
}

fn rt_root() -> TokenStream {
    runtime_root("gb-rt", "rt")
}

/// The injected ambient bank token. `mixed_site` hygiene keeps it invisible to,
/// and unclashable with, user code.
fn token() -> Ident {
    Ident::new("__bank", Span::mixed_site())
}

// ===== body rewrite =====

/// The injected ambient `Anchor` ident. `mixed_site` hygiene, like [`token`].
fn anchor() -> Ident {
    Ident::new("__anchor", Span::mixed_site())
}

struct Rewrite {
    tok: Ident,
    /// The ambient [`Anchor`](gb_bank::Anchor) ident, present only in a *resident*
    /// body (`#[bank::main]` / `#[bank::zero]`). `None` in a `#[bank]` body, where
    /// `scope` / `there` are a compile error: their closure would run across a bank
    /// switch with the banked body's own code unmapped. This single field is the
    /// resident-vs-banked flag *and* the anchor to thread.
    anchor: Option<Ident>,
    /// When the token is already a `&mut Bank<_>` (a function parameter, as in
    /// `FixedFn::run`) we reborrow it (`&mut *tok`); when it is a `Bank<_>` value
    /// (an injected `let mut __bank = ..`) we borrow it (`&mut tok`).
    reborrow: bool,
    used: bool,
    /// Whether an `Anchor` was actually threaded (so the body must mint one).
    anchor_used: bool,
}

impl Rewrite {
    /// The `&mut Bank<_>` expression to thread, matching how the token is held.
    fn bank_arg(&self) -> Expr {
        let tok = &self.tok;
        if self.reborrow {
            parse_quote!(&mut *#tok)
        } else {
            parse_quote!(&mut #tok)
        }
    }

    /// The shared `&Bank<_>` expression to thread (for [`Far::local`], a read).
    fn bank_arg_shared(&self) -> Expr {
        let tok = &self.tok;
        if self.reborrow {
            parse_quote!(&*#tok)
        } else {
            parse_quote!(&#tok)
        }
    }

    /// The ambient `Anchor` expression (a `Copy` value, threaded by name). Only
    /// called in a resident body, where [`anchor`](Self::anchor) is `Some`.
    fn anchor_expr(&self) -> Expr {
        let a = self.anchor.as_ref().expect("anchor present in a resident body");
        parse_quote!(#a)
    }
}

/// The diagnostic shown when a `#[bank]` function reaches for `scope` / `there`.
const BANKED_CLOSURE_MSG: &str = "`scope` / `there` run a closure across a bank \
switch, which a `#[bank]` function cannot survive: its own code lives in a \
switchable bank and is unmapped while the closure runs. Use `.local` for same-bank \
data, `.near` / `.drive` / `.invoke` for calls, or move the closure into a \
`#[bank::zero]` (bank 0) helper.";

/// If `e` is a `scope` or `there` call (the closure-across-a-switch operations), the
/// span to point a diagnostic at. Used to reject them in a banked body. Matches
/// *any* arity, not just the sugar forms, so the explicit `scope(anchor, outer, f)`
/// (which a banked body could otherwise reach with a smuggled `Anchor`) is rejected
/// too. A safe caller cannot build that `Anchor`, but blocking it keeps the gate from
/// resting on that alone.
fn banked_closure_op_span(e: &Expr) -> Option<Span> {
    match e {
        Expr::Call(call) if is_scope_call(&call.func) => Some(call.func.span()),
        Expr::MethodCall(mc) if mc.method == "there" => Some(mc.method.span()),
        _ => None,
    }
}

/// A spanned `::core::compile_error!("msg")` expression.
fn compile_error_expr(span: Span, msg: &str) -> Expr {
    parse2(syn::Error::new(span, msg).to_compile_error())
        .expect("compile_error! is a valid expression")
}

impl VisitMut for Rewrite {
    fn visit_expr_mut(&mut self, e: &mut Expr) {
        // In a banked body (`anchor` is `None`), `scope` / `there` cannot be threaded
        // (their closure would run across a switch unmapped). Replace them with a
        // clear diagnostic in place, before any recursion.
        if self.anchor.is_none() {
            if let Some(span) = banked_closure_op_span(e) {
                *e = compile_error_expr(span, BANKED_CLOSURE_MSG);
                return;
            }
        }

        // A `scope(|b| ..)` rebinds the ambient token to the closure's own `b` for
        // the duration of its body, so nested sugar threads the *innermost* token
        // rather than the outer one. This is both correct (the inner switch restores
        // `b`'s bank, not the outer one) and what keeps the borrow checker enforcing
        // safety: the inner scope reborrows `b` mutably, so a `local` borrow taken from
        // `b` cannot be held across it. Handle it before the generic recursion so the
        // body is visited under the rebound token.
        if let Expr::Call(call) = e {
            if is_scope_call(&call.func) {
                let inner = match call.args.last() {
                    Some(Expr::Closure(c)) => closure_token_ident(c),
                    _ => None,
                };
                if let Some(inner) = inner {
                    // 1-arg sugar injects the ambient anchor and token; the explicit
                    // `scope(anchor, outer, |b| ..)` (3 args) already names them.
                    if call.args.len() == 1 {
                        // `anchor` is `Some` here (banked bodies errored above).
                        call.args.insert(0, self.bank_arg());
                        call.args.insert(0, self.anchor_expr());
                        self.used = true;
                        self.anchor_used = true;
                    }
                    // The closure param `b: &mut Bank<G>` is itself a `&mut`, so it is
                    // threaded by reborrow (`&mut *b`).
                    let saved_tok = std::mem::replace(&mut self.tok, inner);
                    let saved_reborrow = std::mem::replace(&mut self.reborrow, true);
                    if let Some(Expr::Closure(c)) = call.args.last_mut() {
                        self.visit_expr_mut(&mut c.body);
                    }
                    self.tok = saved_tok;
                    self.reborrow = saved_reborrow;
                    return;
                }
            }
        }

        // Recurse first, so nested calls are rewritten before this one.
        visit_mut::visit_expr_mut(self, e);

        if let Expr::MethodCall(mc) = e {
            match mc.method.to_string().as_str() {
                // Same-bank, no-switch sugar (allowed in any body): thread the token.
                // `warp.drive()` / `warp.near()` -> `(&mut __bank)`; explicit
                // `warp.drive(b)` already names a token, so it is left alone.
                "drive" | "near" if mc.args.is_empty() => {
                    mc.args.push(self.bank_arg());
                    self.used = true;
                }
                // `far.local()` -> `far.local(&__bank)` (a shared borrow: same-bank read).
                "local" if mc.args.is_empty() => {
                    mc.args.push(self.bank_arg_shared());
                    self.used = true;
                }
                // `far.invoke(a, b)` -> `far.invoke(&mut __bank, (a, b))` (FarCall::invoke):
                // inject the token, then fold the call's own args into the tuple.
                "invoke" => {
                    let args = std::mem::take(&mut mc.args);
                    mc.args.push(self.bank_arg());
                    mc.args.push(if args.is_empty() {
                        parse_quote!(())
                    } else {
                        parse_quote!((#args,))
                    });
                    self.used = true;
                }
                // `data.there(f)` -> `data.there(__anchor, &mut __bank, f)` (FarWith::there).
                // Banked bodies were rejected above, so `anchor` is `Some` here.
                "there" => {
                    mc.args.insert(0, self.bank_arg());
                    mc.args.insert(0, self.anchor_expr());
                    self.used = true;
                    self.anchor_used = true;
                }
                _ => {}
            }
        }
    }

    // Nested item definitions are separate scopes; leave them alone.
    fn visit_item_mut(&mut self, _: &mut syn::Item) {}
}

/// Whether a call expression targets `scope` (by its last path segment), so the
/// ambient token can be threaded in as its implicit first argument.
fn is_scope_call(func: &Expr) -> bool {
    matches!(
        func,
        Expr::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == "scope")
    )
}

/// The single binding ident of a closure parameter, peeling a type annotation:
/// `b` from `|b|` or `|b: &mut Bank<G>|`. `None` if the closure does not take
/// exactly one simply-named argument, in which case the ambient token is left
/// unchanged.
fn closure_token_ident(closure: &syn::ExprClosure) -> Option<Ident> {
    if closure.inputs.len() != 1 {
        return None;
    }
    let mut pat = closure.inputs.first()?;
    if let Pat::Type(pt) = pat {
        pat = &pt.pat;
    }
    match pat {
        Pat::Ident(pi) => Some(pi.ident.clone()),
        _ => None,
    }
}

/// Rewrite `block` in place and, only where actually used, prepend the ambient
/// token binding for `group` and (for a resident body) the `Anchor`. `resident`
/// gates `scope` / `there`: a banked body (`false`) gets no `Anchor` and rejects
/// them. Bindings are emitted only if touched, so they never trip unused-var lints.
fn transform_body(block: &mut syn::Block, group: &TokenStream, resident: bool) {
    let gb_bank = bank_root();
    let tok = token();
    let anchor_id = anchor();
    let mut rw = Rewrite {
        tok: tok.clone(),
        anchor: resident.then(|| anchor_id.clone()),
        reborrow: false,
        used: false,
        anchor_used: false,
    };
    rw.visit_block_mut(block);
    if rw.anchor_used {
        let mint: Stmt = parse_quote! {
            let #anchor_id = unsafe { #gb_bank::Anchor::assume() };
        };
        block.stmts.insert(0, mint);
    }
    if rw.used {
        let inject: Stmt = parse_quote! {
            let mut #tok = unsafe { #gb_bank::Bank::<#group>::assume() };
        };
        block.stmts.insert(0, inject);
    }
}

/// Rewrite onto the ambient token, but inject *no* token: it is already an
/// in-scope binding (a function parameter). Used by `#[bank::zero]`, whose
/// `FixedFn::run` receives the caller's token as a param. The body is resident, so
/// `scope` / `there` are allowed; the `Anchor` is minted when one is threaded.
fn rewrite_only(block: &mut syn::Block) {
    let gb_bank = bank_root();
    let anchor_id = anchor();
    let mut rw = Rewrite {
        tok: token(),
        anchor: Some(anchor_id.clone()),
        reborrow: true,
        used: false,
        anchor_used: false,
    };
    rw.visit_block_mut(block);
    if rw.anchor_used {
        let mint: Stmt = parse_quote! {
            let #anchor_id = unsafe { #gb_bank::Anchor::assume() };
        };
        block.stmts.insert(0, mint);
    }
}

// ===== bank::module!() =====

pub fn bank_module(input: TokenStream) -> TokenStream {
    let gb_bank = bank_root();
    // Optional pin: `bank::module!(N)` fixes this module to bank `N` instead of
    // letting gb-bank-pack auto-assign. The marker's *initial value* carries the pin
    // (0 = auto); the packer reads it, reserves bank `N`, and leaves it as-is.
    let pin: u16 = if input.is_empty() {
        0
    } else {
        let lit = match parse2::<syn::LitInt>(input) {
            Ok(l) => l,
            Err(e) => return e.to_compile_error(),
        };
        match lit.base10_parse::<u16>() {
            Ok(0) => {
                return syn::Error::new_spanned(
                    &lit,
                    "bank 0 is the resident region; pin to bank 1 or higher",
                )
                .to_compile_error()
            }
            Ok(n) => n,
            Err(e) => return e.to_compile_error(),
        }
    };
    let pin = proc_macro2::Literal::u16_unsuffixed(pin);
    quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub struct __BankGroup;

        impl #gb_bank::Group for __BankGroup {
            // A ROM-ONLY cartridge maps this group at bank 0 like every other, so
            // switching to it is a no-op.
            const FIXED: bool = #gb_bank::FLAT_ROM;
            #[inline(always)]
            fn bank() -> #gb_bank::BankNumber {
                // A pin wider than the marker would be truncated in silence: the
                // overflowing-literal lint does not fire in macro-generated code.
                const _: () = assert!(
                    #pin <= #gb_bank::BankNumber::MAX,
                    "bank::module! pin is above the highest bank this build can select",
                );

                // gb-bank-pack finds this marker by module path (its mangled symbol
                // encodes the path) and patches it to the assigned bank number (or,
                // for a pinned module, leaves the pin in place). Its width follows
                // `BankRepr`, so a wide build needs no change here. The read must be
                // volatile so the value is loaded from memory, not const-folded. (We
                // cannot use the symbol's address as an immediate: this backend emits
                // a section-relative relocation for the marker, which ignores the
                // symbol's value.)
                #[used]
                static BANK: #gb_bank::BankRepr = #pin;
                // The marker stays for the packer either way, but on a flat ROM its
                // value is known, and a volatile read would survive into the binary.
                if #gb_bank::FLAT_ROM {
                    #gb_bank::BankNumber::new(0)
                } else {
                    unsafe {
                        #gb_bank::BankNumber::new_unchecked(::core::ptr::read_volatile(&BANK) as u16)
                    }
                }
            }
        }
    }
}

/// `bank::inherit!()`: a submodule joins its *parent* module's bank group instead of
/// forming its own. It re-exports the parent's `__BankGroup` (and emits no `BANK`
/// marker), so the child's `#[bank]` items are coloured with the parent's group and
/// gb-bank-pack places them in the parent's bank (the child's symbols fall under the
/// parent module's marker by longest-prefix). The Rust keyword `super` cannot name a
/// macro, hence `inherit`.
pub fn bank_inherit(_input: TokenStream) -> TokenStream {
    quote! {
        #[doc(hidden)]
        #[allow(unused_imports)]
        pub use super::__BankGroup;
    }
}

// ===== #[bank] =====

pub fn bank_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(func) = parse2::<ItemFn>(item.clone()) {
        return bank_fn(func);
    }
    if let Ok(s) = parse2::<ItemStatic>(item.clone()) {
        return bank_static(s);
    }
    if let Ok(im) = parse2::<ItemImpl>(item.clone()) {
        return bank_impl(im);
    }
    if let Ok(tr) = parse2::<ItemTrait>(item) {
        return bank_trait(tr);
    }
    syn::Error::new(
        Span::call_site(),
        "#[bank] expects a `fn`, `static`, `impl`, or `trait`",
    )
    .to_compile_error()
}

// --- shared helpers ---

/// A `-> R` return type as tokens (`()` for the default).
fn output_ty(ret: &ReturnType) -> TokenStream {
    match ret {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, t) => quote!(#t),
    }
}

/// Split the typed (non-receiver) arguments into their types and binding idents.
fn typed_args(inputs: &Punctuated<FnArg, Comma>) -> (Vec<Type>, Vec<Ident>) {
    let mut tys = Vec::new();
    let mut names = Vec::new();
    for a in inputs {
        if let FnArg::Typed(pt) = a {
            tys.push((*pt.ty).clone());
            if let Pat::Ident(pi) = &*pt.pat {
                names.push(pi.ident.clone());
            }
        }
    }
    (tys, names)
}

/// The receiver type expressed via `Self`, if `inputs` begins with one.
fn receiver_ty(inputs: &Punctuated<FnArg, Comma>) -> Option<TokenStream> {
    match inputs.first() {
        Some(FnArg::Receiver(r)) => Some(if r.reference.is_some() {
            if r.mutability.is_some() {
                quote!(&mut Self)
            } else {
                quote!(&Self)
            }
        } else {
            quote!(Self)
        }),
        _ => None,
    }
}

/// Build the `BankedWarp::<FnTy, __BankGroup, ArgsTy>::new(callee, args)` expression
/// shared by every banked-call surface (fn, method, trait method).
///
/// `callee` is the hidden body, which is an `unsafe fn` (its precondition is "this
/// group's bank is mapped"). `BankedWarp` only ever runs it inside `scope`, which
/// establishes exactly that, so the cast to a safe fn pointer is sound. Making the
/// body `unsafe` is what stops *safe* code from calling it directly with the wrong
/// bank mapped: an `unsafe fn` pointer does not implement `Fn`, hence the transmute
/// to its safe counterpart here.
fn warp_new(
    fn_ty: &TokenStream,
    args_ty: &TokenStream,
    callee: &TokenStream,
    args_val: &TokenStream,
) -> TokenStream {
    let gb_bank = bank_root();
    quote! {
        unsafe {
            #gb_bank::__private::BankedWarp::<#fn_ty, __BankGroup, #args_ty>::new(
                ::core::mem::transmute::<unsafe #fn_ty, #fn_ty>(#callee as unsafe #fn_ty),
                #args_val,
            )
        }
    }
}

// --- #[bank] fn ---

fn bank_fn(mut func: ItemFn) -> TokenStream {
    let gb_bank = bank_root();
    let vis = func.vis.clone();
    let name = func.sig.ident.clone();
    let generics = func.sig.generics.clone();
    let inputs = func.sig.inputs.clone();
    let output = output_ty(&func.sig.output);
    let (arg_tys, arg_names) = typed_args(&inputs);

    transform_body(&mut func.block, &quote!(__BankGroup), false);

    // The real body becomes a hidden `pub` fn so `far!` can reach it. It must
    // stay a standalone symbol (its own section) to be placed in a bank, so force
    // `#[inline(never)]` and drop any source `#[inline]` that would fight it.
    let impl_name = Ident::new(&format!("__bank_fn_{}", name), Span::call_site());
    func.sig.ident = impl_name.clone();
    func.vis = parse_quote!(pub);
    // `unsafe`: the body assumes its group's bank is mapped (it self-mints the
    // token), which only the `scope` in `BankedWarp`/`FarCall` guarantees. Marking it
    // unsafe makes a direct call from safe code a compile error, not silent UB.
    func.sig.unsafety = Some(Default::default());
    func.attrs.retain(|a| !a.path().is_ident("inline"));

    let fn_ty = quote!(fn(#(#arg_tys),*) -> #output);
    let args_ty = quote!((#(#arg_tys,)*));
    let args_val = quote!((#(#arg_names,)*));

    // Carry the source generics onto the wrapper and turbofish the cast, so the
    // monomorphic fn pointer is named per instantiation.
    let (ig, tg, wc) = generics.split_for_impl();
    let turbofish = tg.as_turbofish();
    let callee = quote!(#impl_name #turbofish);
    let warp = warp_new(&fn_ty, &args_ty, &callee, &args_val);

    // `far!(path::name)` expands to `path::__far_name()`, this constructor: it
    // hands back a `Far` to the hidden body, branded with the module's group.
    let far_name = Ident::new(&format!("__far_{}", name), Span::call_site());

    // Public surface: a real fn returning `impl Warp`, so `f(args)` has full
    // signature help and `.invoke()` drives it.
    quote! {
        #[doc(hidden)]
        #[inline(never)]
        #func

        #[inline]
        #vis fn #name #ig (#inputs) -> impl #gb_bank::Warp<Output = #output, Group = __BankGroup> #wc {
            #warp
        }

        #[doc(hidden)]
        #[inline]
        #vis fn #far_name #ig () -> #gb_bank::Far<#fn_ty, __BankGroup> #wc {
            unsafe { #gb_bank::Far::new(#callee as *const _) }
        }
    }
}

fn bank_static(s: ItemStatic) -> TokenStream {
    let gb_bank = bank_root();
    let vis = s.vis.clone();
    let name = s.ident.clone();
    let ty = (*s.ty).clone();
    let expr = (*s.expr).clone();
    let impl_name = Ident::new(&format!("__bank_static_{}", name), Span::call_site());

    quote! {
        #[doc(hidden)]
        static #impl_name: #ty = #expr;

        #[allow(non_upper_case_globals)]
        #vis const #name: #gb_bank::Far<#ty, __BankGroup> =
            unsafe { #gb_bank::Far::new(&#impl_name) };
    }
}

// --- #[bank] impl ---

/// Rewrite every method of an `impl` block into a banked surface. The wrapper
/// methods (returning `impl Warp`) replace the originals in the block as given
/// (inherent or trait impl); the real bodies move to a sibling inherent block as
/// hidden `__bank_*` methods, which is also where trait-impl bodies must live
/// (a trait impl may only define trait members).
fn bank_impl(im: ItemImpl) -> TokenStream {
    let ItemImpl { generics, self_ty, trait_, items, .. } = im;
    let (ig, _tg, wc) = generics.split_for_impl();
    // The hidden bodies move to a sibling inherent block, so a trait impl's
    // `Self::Assoc` references must be re-qualified against the trait.
    let trait_path: Option<Path> = trait_.as_ref().map(|(_, p, _)| p.clone());
    let trait_prefix = match &trait_ {
        Some((bang, path, for_)) => quote!(#bang #path #for_),
        None => quote!(),
    };

    let mut wrappers = Vec::new();
    let mut hiddens = Vec::new();
    let mut passthrough = Vec::new();
    for it in items {
        match it {
            ImplItem::Fn(m) => {
                let (w, h) = bank_method(m, trait_path.as_ref());
                wrappers.push(w);
                hiddens.push(h);
            }
            other => passthrough.push(other),
        }
    }

    quote! {
        impl #ig #trait_prefix #self_ty #wc {
            #(#passthrough)*
            #(#wrappers)*
        }

        impl #ig #self_ty #wc {
            #(#hiddens)*
        }
    }
}

/// In a hidden body lifted out of a trait impl, rewrite `Self::Assoc` to
/// `<Self as Trait>::Assoc`, which a bare inherent `impl` cannot otherwise name.
struct SelfQualify<'a> {
    trait_path: &'a Path,
}

impl VisitMut for SelfQualify<'_> {
    fn visit_type_path_mut(&mut self, tp: &mut syn::TypePath) {
        visit_mut::visit_type_path_mut(self, tp);
        let segs = &tp.path.segments;
        if tp.qself.is_none() && segs.len() >= 2 && segs[0].ident == "Self" {
            let tail: Vec<_> = segs.iter().skip(1).cloned().collect();
            let trait_path = self.trait_path;
            if let Type::Path(np) = parse_quote!(<Self as #trait_path>::#(#tail)::*) {
                *tp = np;
            }
        }
    }
}

/// One method into `(wrapper, hidden)`. The receiver, if any, becomes the first
/// argument of the underlying fn pointer (`fn(&Self, ..) -> R`).
fn bank_method(mut m: ImplItemFn, trait_path: Option<&Path>) -> (TokenStream, TokenStream) {
    let gb_bank = bank_root();
    let vis = m.vis.clone();
    let name = m.sig.ident.clone();
    let mgen = m.sig.generics.clone();
    let inputs = m.sig.inputs.clone();
    let output = output_ty(&m.sig.output);
    let (arg_tys, arg_names) = typed_args(&inputs);
    let recv = receiver_ty(&inputs);

    transform_body(&mut m.block, &quote!(__BankGroup), false);

    let hidden_name = Ident::new(&format!("__bank_{}", name), Span::call_site());
    let hidden = {
        let mut hm = m.clone();
        hm.sig.ident = hidden_name.clone();
        hm.vis = parse_quote!(pub);
        // `unsafe`: like the free-fn case, the body self-mints its group's token,
        // sound only when the bank is already mapped (which `scope` guarantees on
        // every real call path). Making it unsafe blocks direct calls from safe code.
        hm.sig.unsafety = Some(Default::default());
        hm.attrs.retain(|a| !a.path().is_ident("inline"));
        if let Some(tp) = trait_path {
            SelfQualify { trait_path: tp }.visit_signature_mut(&mut hm.sig);
        }
        quote! {
            #[doc(hidden)]
            #[inline(never)]
            #hm
        }
    };

    // Fold the receiver in as the leading parameter.
    let mut all_tys: Vec<TokenStream> = Vec::new();
    let mut all_vals: Vec<TokenStream> = Vec::new();
    if let Some(rty) = &recv {
        all_tys.push(rty.clone());
        all_vals.push(quote!(self));
    }
    for t in &arg_tys {
        all_tys.push(quote!(#t));
    }
    for n in &arg_names {
        all_vals.push(quote!(#n));
    }

    let fn_ty = quote!(fn(#(#all_tys),*) -> #output);
    let args_ty = quote!((#(#all_tys,)*));
    let args_val = quote!((#(#all_vals,)*));

    let (mig, mtg, mwc) = mgen.split_for_impl();
    let turbofish = mtg.as_turbofish();
    let callee = quote!(Self::#hidden_name #turbofish);
    let warp = warp_new(&fn_ty, &args_ty, &callee, &args_val);

    let wrapper = quote! {
        #[inline]
        #vis fn #name #mig (#inputs) -> impl #gb_bank::Warp<Output = #output> #mwc {
            #warp
        }
    };

    // `far!(Type::name)` -> `Type::__far_name()`: an associated constructor living
    // in the same inherent block as the hidden body. The receiver is folded into
    // the fn pointer as its leading argument (as in the wrapper above). Unlike the
    // wrapper (which sits in the trait impl), this lives in the inherent block, so
    // a trait-impl `Self::Assoc` in the signature must be fully qualified.
    let far_fn_ty = match trait_path {
        Some(tp) => {
            let mut t: Type = parse_quote!(#fn_ty);
            SelfQualify { trait_path: tp }.visit_type_mut(&mut t);
            quote!(#t)
        }
        None => fn_ty.clone(),
    };
    let far_name = Ident::new(&format!("__far_{}", name), Span::call_site());
    let far_ctor = quote! {
        #[doc(hidden)]
        #[inline]
        #vis fn #far_name #mig () -> #gb_bank::Far<#far_fn_ty, __BankGroup> #mwc {
            unsafe { #gb_bank::Far::new(#callee as *const _) }
        }
    };

    (wrapper, quote!(#hidden #far_ctor))
}

// --- #[bank] trait ---

/// Color a trait: every method `fn m(..) -> R` becomes `fn m(..) -> impl
/// Warp<Output = R>` (RPITIT), the same shape an `async fn` in a trait desugars
/// to. The matching `#[bank] impl` then supplies the `Warp`-returning bodies.
fn bank_trait(mut tr: ItemTrait) -> TokenStream {
    let gb_bank = bank_root();
    for it in &mut tr.items {
        if let TraitItem::Fn(m) = it {
            if m.default.is_some() {
                return syn::Error::new(
                    Span::call_site(),
                    "#[bank] trait does not support default method bodies yet",
                )
                .to_compile_error();
            }
            let out = output_ty(&m.sig.output);
            m.sig.output = parse_quote!(-> impl #gb_bank::Warp<Output = #out>);
        }
    }
    quote!(#tr)
}

// ===== #[bank::zero] (resident, bank-0 functions) =====

/// Inject the `GroupZero` token and rewrite the body, without moving the
/// function into a bank or wrapping it in a `Warp`. Used for all resident
/// (`#[bank::zero]`) code: the entry point and any helper.
/// `#[bank::main]`: the resident entry point. It is the root of the call stack,
/// reached by the runtime with no token, so it assumes a [`GroupZero`] token and
/// drives banked calls directly (no `Warp`). This is the original resident
/// expansion.
pub fn resident(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let gb_bank = bank_root();
    let mut func = match parse2::<ItemFn>(item) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };
    transform_body(&mut func.block, &quote!(#gb_bank::GroupZero), true);
    // The entry-point concerns (export as `main`, force the gb-rt startup into
    // the link) are owned by gb-rt's #[entry]; delegate to it.
    let gb_rt = rt_root();
    quote!(#[#gb_rt::entry] #func)
}

/// `#[bank::zero]`: a resident helper that *propagates the caller's bank*.
///
/// Unlike the entry point, a helper can be called from inside any bank, so it
/// must thread whatever token its caller holds (and restore that bank). The body
/// is encoded as a [`FixedFn`] impl on a per-function marker (the generic lives on
/// `run`'s `__C`), and the public name becomes a `fn -> impl Warp` invoked as
/// `helper(args).invoke()`.
/// A `PhantomData<..>` that uses every type and lifetime parameter, so the marker
/// struct is not rejected for unused generics (E0392).
fn phantom_data(generics: &syn::Generics) -> TokenStream {
    let mut parts: Vec<TokenStream> = Vec::new();
    for p in &generics.params {
        match p {
            syn::GenericParam::Lifetime(l) => {
                let lt = &l.lifetime;
                parts.push(quote!(& #lt ()));
            }
            syn::GenericParam::Type(t) => {
                let id = &t.ident;
                parts.push(quote!(#id));
            }
            // const params do not trigger the unused-parameter error.
            syn::GenericParam::Const(_) => {}
        }
    }
    quote!(::core::marker::PhantomData<(#(#parts,)*)>)
}

/// `#[bank::zero]` on a `fn`, `impl`, or `trait`. A trait is coloured exactly
/// like `#[bank] trait` (its method signatures become `-> impl Warp`); the
/// `#[bank::zero] impl` supplies the `FixedWarp` bodies.
pub fn resident_fixed(_attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Ok(func) = parse2::<ItemFn>(item.clone()) {
        return resident_fn(func);
    }
    if let Ok(im) = parse2::<ItemImpl>(item.clone()) {
        return resident_impl(im);
    }
    if let Ok(tr) = parse2::<ItemTrait>(item) {
        return bank_trait(tr);
    }
    syn::Error::new(
        Span::call_site(),
        "#[bank::zero] expects a `fn`, `impl`, or `trait`",
    )
    .to_compile_error()
}

fn resident_fn(func: ItemFn) -> TokenStream {
    let gb_bank = bank_root();
    let vis = func.vis.clone();
    let name = func.sig.ident.clone();
    let generics = func.sig.generics.clone();
    let inputs = func.sig.inputs.clone();
    let output = output_ty(&func.sig.output);
    let (arg_tys, arg_names) = typed_args(&inputs);

    let mut block = func.block;
    rewrite_only(&mut block);

    let tok = token();
    let marker = Ident::new(&format!("__fixed_{}", name), Span::call_site());
    let args_ty = quote!((#(#arg_tys,)*));
    let args_pat = quote!((#(#arg_names,)*));
    let args_val = quote!((#(#arg_names,)*));

    // Carry the helper's generics onto the marker, its `FixedFn` impl, and the
    // wrapper. The caller-group generic `__C` lives only on `run`.
    let (ig, tg, wc) = generics.split_for_impl();
    let marker_ty = quote!(#marker #tg);
    let marker_def = if generics.params.is_empty() {
        quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            pub struct #marker;
        }
    } else {
        let phantom = phantom_data(&generics);
        quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            pub struct #marker #ig (#phantom) #wc;
        }
    };

    // far! support: a resident helper has no plain fn pointer (its `run<C>` is
    // generic over the caller), so for dynamic dispatch we monomorphize at
    // GroupZero and restore the caller's bank at *runtime* instead of by type.
    let dyn_name = Ident::new(&format!("__dyn_{}", name), Span::call_site());
    let far_name = Ident::new(&format!("__far_{}", name), Span::call_site());
    let fn_ty = quote!(fn(#(#arg_tys),*) -> #output);
    let turbofish = tg.as_turbofish();

    quote! {
        #marker_def

        #[doc(hidden)]
        impl #ig #gb_bank::__private::FixedFn<#args_ty> for #marker_ty #wc {
            type Output = #output;
            #[inline]
            fn run<__C: #gb_bank::Group>(
                #args_pat: #args_ty,
                #tok: &mut #gb_bank::Bank<__C>,
            ) -> #output #block
        }

        // Public surface: a real fn returning `impl Warp`, driven with `.invoke()`,
        // which threads the caller's token into `run` (no bank switch).
        #[inline]
        #vis fn #name #ig (#inputs) -> impl #gb_bank::Warp<Output = #output> #wc {
            #gb_bank::__private::FixedWarp::<#marker_ty, #args_ty>::new(#args_val)
        }

        // Trampoline behind `far!`: save the caller's bank at runtime, run the body
        // anchored at GroupZero (inner calls leave their target mapped), then put the
        // saved bank back. Self-contained, so it is safe to call from any bank.
        #[doc(hidden)]
        #[inline(never)]
        #vis fn #dyn_name #ig (#inputs) -> #output #wc {
            let __saved = #gb_bank::current_bank();
            let mut __z = unsafe { #gb_bank::Bank::<#gb_bank::GroupZero>::assume() };
            let __r = <#marker_ty as #gb_bank::__private::FixedFn<#args_ty>>
                ::run::<#gb_bank::GroupZero>(
                #args_val, &mut __z,
            );
            unsafe { #gb_bank::switch_bank(__saved) };
            __r
        }

        #[doc(hidden)]
        #[inline]
        #vis fn #far_name #ig () -> #gb_bank::Far<#fn_ty, #gb_bank::GroupZero> #wc {
            unsafe { #gb_bank::Far::new(#dyn_name #turbofish as *const _) }
        }
    }
}

/// `#[bank::zero] impl`: each method becomes a resident helper that propagates
/// the caller's token. Generic impls/methods are not supported yet.
fn resident_impl(im: ItemImpl) -> TokenStream {
    if !im.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &im.self_ty,
            "#[bank::zero] generic impls are not supported yet",
        )
        .to_compile_error();
    }
    let self_ty = (*im.self_ty).clone();
    let trait_prefix = match &im.trait_ {
        Some((bang, path, for_)) => quote!(#bang #path #for_),
        None => quote!(),
    };
    let self_ident = match &self_ty {
        Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.clone()),
        _ => None,
    };
    let Some(self_ident) = self_ident else {
        return syn::Error::new_spanned(&self_ty, "#[bank::zero] impl requires a named type")
            .to_compile_error();
    };

    let mut wrappers = Vec::new();
    let mut hiddens = Vec::new();
    let mut markers = Vec::new();
    let mut passthrough = Vec::new();
    let mut errs = Vec::new();
    for it in im.items {
        match it {
            ImplItem::Fn(m) => {
                if !m.sig.generics.params.is_empty() {
                    errs.push(
                        syn::Error::new_spanned(
                            &m.sig.ident,
                            "#[bank::zero] generic methods are not supported yet",
                        )
                        .to_compile_error(),
                    );
                    continue;
                }
                let (w, h, mk) = resident_method(m, &self_ty, &self_ident);
                wrappers.push(w);
                hiddens.push(h);
                markers.push(mk);
            }
            other => passthrough.push(other),
        }
    }

    quote! {
        #(#errs)*

        impl #trait_prefix #self_ty {
            #(#passthrough)*
            #(#wrappers)*
        }

        impl #self_ty {
            #(#hiddens)*
        }

        #(#markers)*
    }
}

fn resident_method(
    mut m: ImplItemFn,
    self_ty: &Type,
    self_ident: &Ident,
) -> (TokenStream, TokenStream, TokenStream) {
    let gb_bank = bank_root();
    let vis = m.vis.clone();
    let name = m.sig.ident.clone();
    let inputs = m.sig.inputs.clone();
    let output = output_ty(&m.sig.output);
    let (arg_tys, arg_names) = typed_args(&inputs);

    rewrite_only(&mut m.block);
    let block = m.block;

    let tok = token();
    let hidden_name = Ident::new(&format!("__fixed_{}", name), Span::call_site());
    let marker = Ident::new(&format!("__fixed_{}_{}", self_ident, name), Span::call_site());

    // Concrete receiver type (with a fresh lifetime for `&self` / `&mut self`),
    // plus the `FixedFn` impl's lifetime list.
    // One shared lifetime token, so the `<'__r>` on the impl and the `&'__r`
    // receiver type carry the same hygiene (separate `quote!`s would not).
    // `recv_ty` carries the explicit `'__r` for the `FixedFn` impl; `elided_recv`
    // is the same receiver with an elided lifetime, for the `far!` trampoline and
    // fn-pointer type (where a higher-ranked `fn(&T, ..)` is what we want).
    let r_lt = syn::Lifetime::new("'__r", Span::mixed_site());
    let (impl_lt, recv_ty, elided_recv) = match inputs.first() {
        Some(FnArg::Receiver(r)) if r.reference.is_some() => {
            let m = &r.mutability;
            (quote!(<#r_lt>), Some(quote!(& #r_lt #m #self_ty)), Some(quote!(& #m #self_ty)))
        }
        Some(FnArg::Receiver(_)) => (quote!(), Some(quote!(#self_ty)), Some(quote!(#self_ty))),
        _ => (quote!(), None, None),
    };

    // The hidden inherent method keeps the real `self`, gaining the threaded token.
    let hidden = quote! {
        #[doc(hidden)]
        #[inline]
        pub fn #hidden_name<__C: #gb_bank::Group>(
            #inputs,
            #tok: &mut #gb_bank::Bank<__C>,
        ) -> #output #block
    };

    // Fold the receiver in as the first argument; `run` forwards everything to
    // the hidden method by UFCS.
    let recv_name = Ident::new("__recv", Span::mixed_site());
    let mut all_tys: Vec<TokenStream> = Vec::new();
    let mut pat: Vec<TokenStream> = Vec::new();
    let mut fwd: Vec<TokenStream> = Vec::new();
    let mut wrap_vals: Vec<TokenStream> = Vec::new();
    if let Some(rty) = &recv_ty {
        all_tys.push(rty.clone());
        pat.push(quote!(#recv_name));
        fwd.push(quote!(#recv_name));
        wrap_vals.push(quote!(self));
    }
    for (t, n) in arg_tys.iter().zip(arg_names.iter()) {
        all_tys.push(quote!(#t));
        pat.push(quote!(#n));
        fwd.push(quote!(#n));
        wrap_vals.push(quote!(#n));
    }
    let args_ty = quote!((#(#all_tys,)*));
    let args_pat = quote!((#(#pat,)*));
    let args_val = quote!((#(#wrap_vals,)*));

    let marker_full = quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub struct #marker;

        #[doc(hidden)]
        impl #impl_lt #gb_bank::__private::FixedFn<#args_ty> for #marker {
            type Output = #output;
            #[inline]
            fn run<__C: #gb_bank::Group>(
                #args_pat: #args_ty,
                #tok: &mut #gb_bank::Bank<__C>,
            ) -> #output {
                <#self_ty>::#hidden_name::<__C>(#(#fwd,)* #tok)
            }
        }
    };

    // The wrapper infers `Args` from the values (the receiver's real lifetime),
    // rather than naming the impl-only `'__r`.
    let wrapper = quote! {
        #[inline]
        #vis fn #name(#inputs) -> impl #gb_bank::Warp<Output = #output> {
            #gb_bank::__private::FixedWarp::<#marker, _>::new(#args_val)
        }
    };

    // far! support: the folded fn-pointer signature (receiver as first arg, elided
    // lifetime), a runtime save/restore trampoline calling the hidden body anchored
    // at GroupZero, and the `Type::__far_name()` constructor next to the wrapper.
    let mut dyn_tys: Vec<TokenStream> = Vec::new();
    if let Some(ert) = &elided_recv {
        dyn_tys.push(ert.clone());
    }
    for t in &arg_tys {
        dyn_tys.push(quote!(#t));
    }
    let dyn_name = Ident::new(&format!("__dyn_{}_{}", self_ident, name), Span::call_site());
    let far_name = Ident::new(&format!("__far_{}", name), Span::call_site());
    let dyn_fn_ty = quote!(fn(#(#dyn_tys),*) -> #output);

    let trampoline = quote! {
        #[doc(hidden)]
        #[inline(never)]
        #vis fn #dyn_name(#(#fwd: #dyn_tys),*) -> #output {
            let __saved = #gb_bank::current_bank();
            let mut __z = unsafe { #gb_bank::Bank::<#gb_bank::GroupZero>::assume() };
            let __r = <#self_ty>::#hidden_name::<#gb_bank::GroupZero>(#(#fwd,)* &mut __z);
            unsafe { #gb_bank::switch_bank(__saved) };
            __r
        }
    };
    let far_ctor = quote! {
        #[doc(hidden)]
        #[inline]
        #vis fn #far_name() -> #gb_bank::Far<#dyn_fn_ty, #gb_bank::GroupZero> {
            unsafe { #gb_bank::Far::new(#dyn_name as *const _) }
        }
    };

    (wrapper, quote!(#hidden #far_ctor), quote!(#marker_full #trampoline))
}

// ===== far!() =====

pub fn far(input: TokenStream) -> TokenStream {
    let path: Path = match parse2(input) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error(),
    };
    if path.segments.is_empty() {
        return syn::Error::new(Span::call_site(), "far!(): expected a path to a banked fn")
            .to_compile_error();
    }

    // `a::b::f`  ->  call `a::b::__far_f()`, the constructor emitted next to the
    // wrapper. It already pairs the hidden body's address with the right group, so
    // far! needs no knowledge of whether `f` is a banked fn or a resident helper.
    let mut far_path = path;
    let last = far_path.segments.len() - 1;
    let name = far_path.segments[last].ident.clone();
    far_path.segments[last].ident = Ident::new(&format!("__far_{}", name), name.span());

    quote! { #far_path() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    /// A nested `scope` must thread the *inner* closure's token, not the ambient
    /// resident one: the inner scope and the calls inside it bind to `b`/`c`, never
    /// to `__bank`. This is what keeps the bank restore correct and lets the borrow
    /// checker forbid holding a `local` borrow across the inner switch.
    #[test]
    fn nested_scope_threads_innermost_token() {
        let item = quote! {
            pub fn probe() -> u8 {
                scope(|b| {
                    let _ = foo().drive();
                    scope(|c| bar().drive())
                })
            }
        };
        let out = resident_fixed(quote!(), item).to_string();

        // Outer scope threads the ambient anchor and the resident token.
        assert!(out.contains("scope (__anchor , & mut * __bank , | b |"), "outer scope:\n{out}");
        // Call inside the outer scope threads `b`.
        assert!(out.contains("drive (& mut * b)"), "outer drive:\n{out}");
        // Inner scope reuses the (Copy) anchor but threads `b` (NOT `__bank`).
        assert!(out.contains("scope (__anchor , & mut * b , | c |"), "inner scope:\n{out}");
        assert!(!out.contains("& mut * __bank , | c |"), "inner scope used __bank:\n{out}");
        // Call inside the inner scope threads `c`.
        assert!(out.contains("drive (& mut * c)"), "inner drive:\n{out}");
        // The resident body mints exactly one anchor.
        assert!(out.contains("Anchor :: assume"), "anchor minted:\n{out}");
    }

    /// In a banked body `scope` / `there` are rejected with the diagnostic, since the
    /// closure would run across a switch with the banked code unmapped.
    #[test]
    fn banked_scope_and_with_are_rejected() {
        let scope_item = quote! {
            pub fn probe() -> u8 { scope(|b| foo().near(b)) }
        };
        let out = bank_attr(quote!(), scope_item).to_string();
        assert!(out.contains("compile_error"), "banked scope not rejected:\n{out}");

        let with_item = quote! {
            pub fn probe() -> u8 { DATA.there(|d| d[0]) }
        };
        let out = bank_attr(quote!(), with_item).to_string();
        assert!(out.contains("compile_error"), "banked with not rejected:\n{out}");
    }

    /// `bank::module!()` sets the marker to 0 (auto-assign); `bank::module!(N)` to N
    /// (pinned); `bank::module!(0)` is rejected.
    #[test]
    fn module_pin_marker() {
        let auto = bank_module(quote!()).to_string();
        assert!(auto.contains("static BANK : :: gb_bank :: BankRepr = 0"), "auto marker:\n{auto}");

        let pinned = bank_module(quote!(3)).to_string();
        let want = "static BANK : :: gb_bank :: BankRepr = 3";
        assert!(pinned.contains(want), "pinned marker:\n{pinned}");

        let zero = bank_module(quote!(0)).to_string();
        assert!(zero.contains("compile_error"), "bank 0 not rejected:\n{zero}");
    }

    /// `bank::inherit!()` re-exports the parent's group and emits no marker.
    #[test]
    fn inherit_reexports_parent_group() {
        let out = bank_inherit(quote!()).to_string();
        assert!(out.contains("use super :: __BankGroup"), "inherit re-export:\n{out}");
        assert!(!out.contains("BANK"), "inherit must emit no marker:\n{out}");
    }
}
