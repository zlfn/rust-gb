//! Procedural macros for `gb-rt`, re-exported by the `gb-rt` crate and meant to
//! be used through it (`#[gb_rt::entry]`).

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ReturnType, Type, parse_macro_input, parse_quote, spanned::Spanned};

/// Mark the program entry point.
///
/// Applied to `fn main`, it exports the function as the Game Boy entry symbol
/// (`#[no_mangle] pub extern "C" fn main`) and forces the gb-rt startup (rrt0)
/// into the link. The signature must be `fn() -> !`: rrt0 jumps to it and never
/// expects it back, so the program loops forever. Use it once per program:
///
/// ```ignore
/// #[gb_rt::entry]
/// fn main() -> ! {
///     loop { /* ... */ }
/// }
/// ```
#[proc_macro_attribute]
pub fn entry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);

    if !matches!(&func.sig.output, ReturnType::Type(_, ty) if matches!(**ty, Type::Never(_))) {
        return syn::Error::new(
            func.sig.span(),
            "`#[gb_rt::entry]` requires the signature `fn() -> !`: the entry never returns",
        )
        .to_compile_error()
        .into();
    }

    func.vis = parse_quote!(pub);
    func.sig.abi = Some(parse_quote!(extern "C"));

    quote! {
        #[unsafe(no_mangle)]
        #func

        // `_reset` (gb-rt's startup entry) is named only by the linker's ENTRY
        // directive, which runs after the staticlib is assembled, so without a
        // reference here the startup would be dropped. This `#[used]` static lives
        // in the root crate (where `#[used]` is honored) and pulls it into the link.
        #[used]
        static __GB_RT_KEEP_RESET: unsafe extern "C" fn() = {
            unsafe extern "C" {
                // The SM83 target prefixes C symbols with `_`, so `reset` here
                // binds to the `_reset` startup symbol defined in rrt0.s.
                fn reset();
            }
            reset
        };
    }
    .into()
}
