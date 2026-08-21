//! Procedural macros for `gb-rt`, re-exported by the `gb-rt` crate and meant to
//! be used through it (`#[gb_rt::entry]`).

use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemFn, ReturnType, Type, parse_macro_input, parse_quote, spanned::Spanned};

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
/// with `reti`. It takes no parameters and returns nothing. The exported symbol is
/// strong, overriding the weak or PROVIDE default, so this handler runs in place
/// of any default for that vector. The defining crate needs
/// `#![feature(abi_z80_interrupt)]`:
///
/// ```ignore
/// #![feature(abi_z80_interrupt)]
///
/// #[gb_rt::interrupt(LcdStat)]
/// fn wobble() {
///     // runs on each STAT interrupt
/// }
/// ```
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
    if !func.sig.inputs.is_empty() {
        return syn::Error::new(
            func.sig.inputs.span(),
            "`#[gb_rt::interrupt]` handlers take no parameters",
        )
        .to_compile_error()
        .into();
    }
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

    func.sig.abi = Some(parse_quote!(extern "z80-interrupt"));

    quote! {
        #[unsafe(export_name = #symbol)]
        #func
    }
    .into()
}
