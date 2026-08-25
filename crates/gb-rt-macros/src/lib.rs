//! Procedural macros for `gb-rt`, re-exported by the `gb-rt` crate and meant to
//! be used through it (`#[gb_rt::entry]`).

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, ItemFn, ReturnType, Type, parse_macro_input, parse_quote, spanned::Spanned};

/// Path to `gb-rt` as the *invoking* crate can name it: directly when it depends
/// on that crate, otherwise through the `gb` facade, which re-exports it at `rt`.
/// Falling back to the plain name when neither is present lets the usual
/// unresolved-import error name the crate the user is missing.
fn rt_root() -> TokenStream2 {
    use proc_macro_crate::{crate_name, FoundCrate};
    let path = match crate_name("gb-rt") {
        Ok(FoundCrate::Itself) => "crate".to_string(),
        Ok(FoundCrate::Name(n)) => format!("::{}", n.replace('-', "_")),
        Err(_) => match crate_name("rust-gb").or_else(|_| crate_name("gb")) {
            Ok(FoundCrate::Itself) => "crate::rt".to_string(),
            Ok(FoundCrate::Name(n)) => format!("::{}::rt", n.replace('-', "_")),
            Err(_) => "::gb_rt".to_string(),
        },
    };
    path.parse().expect("gb-rt root path")
}

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

/// Install an interrupt handler directly at its vector.
///
/// Takes the vector name and binds the function to the matching `_on_*` symbol:
///
/// | Argument   | Symbol         |
/// |------------|----------------|
/// | `VBlank`   | `_on_vblank`   |
/// | `LcdStat`  | `_on_lcd_stat` |
/// | `Timer`    | `_on_timer`    |
/// | `Serial`   | `_on_serial`   |
/// | `Joypad`   | `_on_joypad`   |
///
/// The vector jumps straight to the handler, which the `z80-interrupt` calling
/// convention compiles to save only the register pairs it clobbers and to return
/// with `reti`. It returns nothing. The exported symbol is strong, overriding the
/// weak or PROVIDE default, so this handler runs in place of any default for that
/// vector. The defining crate needs `#![feature(abi_z80_interrupt)]`:
///
/// ```ignore
/// #![feature(abi_z80_interrupt)]
///
/// #[gb_rt::interrupt(LcdStat)]
/// fn wobble() {
///     // runs on each STAT interrupt
/// }
/// ```
///
/// # Critical section
///
/// A handler may take one `CriticalSection` parameter.
/// The CPU clears IME when it dispatches an interrupt, so the handler already runs
/// with interrupts off; the token records that and unlocks anything guarding state
/// shared with the main loop, such as a wide `gb_hram` cell. It is zero-sized and
/// the wrapper inlines away, so taking it costs nothing.
///
/// ```ignore
/// #[gb_rt::interrupt(Timer)]
/// fn tick(cs: CriticalSection) {
///     TICKS.set_cs(cs, TICKS.get_cs(cs).wrapping_add(1));
/// }
/// ```
///
/// Enabling interrupts inside such a handler invalidates the token while it is
/// still in scope, which is why doing so is `unsafe`.
#[proc_macro_attribute]
pub fn interrupt(attr: TokenStream, item: TokenStream) -> TokenStream {
    let vector = parse_macro_input!(attr as Ident);
    let mut func = parse_macro_input!(item as ItemFn);

    let symbol = match vector.to_string().as_str() {
        "VBlank" => "on_vblank",
        "LcdStat" => "on_lcd_stat",
        "Timer" => "on_timer",
        "Serial" => "on_serial",
        "Joypad" => "on_joypad",
        _ => {
            return syn::Error::new(
                vector.span(),
                "unknown interrupt vector; expected VBlank, LcdStat, Timer, Serial, or Joypad",
            )
            .to_compile_error()
            .into();
        }
    };

    if !func.sig.generics.params.is_empty() {
        return syn::Error::new(
            func.sig.generics.span(),
            "`#[gb_rt::interrupt]` handlers cannot be generic",
        )
        .to_compile_error()
        .into();
    }
    if func.sig.inputs.len() > 1 {
        return syn::Error::new(
            func.sig.inputs.span(),
            "`#[gb_rt::interrupt]` handlers take no parameters, or one \
             `CriticalSection`",
        )
        .to_compile_error()
        .into();
    }
    let takes_cs = func.sig.inputs.len() == 1;
    if let ReturnType::Type(_, ty) = &func.sig.output {
        if !matches!(**ty, Type::Tuple(ref t) if t.elems.is_empty()) {
            return syn::Error::new(
                func.sig.output.span(),
                "`#[gb_rt::interrupt]` handlers return `()`",
            )
            .to_compile_error()
            .into();
        }
    }

    if !takes_cs {
        func.sig.abi = Some(parse_quote!(extern "z80-interrupt"));
        return quote! {
            #[unsafe(export_name = #symbol)]
            #func
        }
        .into();
    }

    // The CPU clears IME on dispatch, so the handler already runs with
    // interrupts off and the token is sound. Minting it here keeps that
    // assertion out of the handler body.
    let gb_rt = rt_root();
    let attrs = core::mem::take(&mut func.attrs);
    let vis = func.vis.clone();
    let name = func.sig.ident.clone();
    func.sig.ident = parse_quote!(__gb_rt_isr_body);
    func.vis = syn::Visibility::Inherited;

    quote! {
        #(#attrs)*
        #[unsafe(export_name = #symbol)]
        #vis extern "z80-interrupt" fn #name() {
            #[inline(always)]
            #func
            __gb_rt_isr_body(unsafe { #gb_rt::CriticalSection::new() })
        }
    }
    .into()
}
