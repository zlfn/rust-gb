//! Procedural macro for gb-ram-fn (the `#[ram_fn]` attribute).

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{FnArg, Ident, ItemFn, LitInt, Token, parse_macro_input};

struct Args {
    max: LitInt,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if key != "max" {
            return Err(syn::Error::new(key.span(), "expected `max = N`"));
        }
        input.parse::<Token![=]>()?;
        Ok(Args {
            max: input.parse()?,
        })
    }
}

/// Define a RAM-copyable function with any non-generic signature.
///
/// `max` is the function's maximum compiled size, in bytes. `install` relies on
/// it for a compile-time buffer check, but the bound itself (actual code length
/// `<= max`) is verified only after compilation, by `cargo-gb` over the linked
/// ROM. A build path that skips that check does not guarantee it, leaving
/// `install` able to overflow its buffer (see
/// [`RamFn::install`](../gb_ram_fn/trait.RamFn.html#tymethod.install)).
///
/// Generates `NAME`, a zero-sized handle implementing `RamFn`: `NAME.rom()` gives
/// the ROM copy and `NAME.install(buf)` copies it into RAM. Bring `RamFn` into
/// scope to call these.
#[proc_macro_attribute]
pub fn ram_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as Args);
    let func = parse_macro_input!(item as ItemFn);
    let max = &args.max;

    if !func.sig.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &func.sig.generics,
            "#[ram_fn] does not support generic functions",
        )
        .to_compile_error()
        .into();
    }

    let vis = &func.vis;
    let name = &func.sig.ident;
    let inputs = &func.sig.inputs;
    let output = &func.sig.output;
    let block = &func.block;

    let argtys: Vec<_> = inputs
        .iter()
        .filter_map(|a| match a {
            FnArg::Typed(t) => Some((*t.ty).clone()),
            FnArg::Receiver(_) => None,
        })
        .collect();

    let sec0 = format!("ramfn_{name}_0");
    let sec1 = format!("ramfn_{name}_1");
    let sec2 = format!("ramfn_{name}_2");

    quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #vis mod #name {
            use super::*;

            // The function and an end marker go in adjacent, name-sorted sections
            // (`gb_ram_fn.ld` collects `ramfn_*` with SORT), so `END - run` is the
            // code length. `src()` returns `run`'s address, which both gives the
            // copy source and keeps the body alive: taking the address forces it to
            // be emitted even when only `install` is used.
            #[inline(never)]
            #[unsafe(link_section = #sec0)]
            pub fn run(#inputs) #output #block

            #[used]
            #[unsafe(link_section = #sec1)]
            static END: [u8; 0] = [];

            // The declared `max`, read from the ROM image by the post-build size
            // check (`cargo-gb`); it confirms `END - run <= max`.
            #[used]
            #[unsafe(link_section = #sec2)]
            static MAX_MARKER: u16 = #max;

            /// Handle to a RAM-copyable function. See [`RamFn`](::gb_ram_fn::RamFn).
            pub struct Handle;

            impl ::gb_ram_fn::RamFn for Handle {
                type Fn = fn(#(#argtys),*) #output;
                const MAX: usize = #max;

                #[inline]
                fn src(&self) -> *const u8 {
                    run as *const () as *const u8
                }

                #[inline]
                fn len(&self) -> usize {
                    (&raw const END as usize) - (self.src() as usize)
                }

                #[inline]
                fn rom(&self) -> Self::Fn {
                    run
                }

                #[inline]
                unsafe fn install<const N: usize>(&self, dst: *mut [u8; N]) -> Self::Fn {
                    const { ::core::assert!(N >= Self::MAX, "ram_fn buffer smaller than `max`") };
                    unsafe {
                        // Addresses as integers, copied with volatile loads/stores.
                        // The bytes sit past the zero-sized marker, so pointer
                        // arithmetic from it is out of bounds and the optimizer
                        // would fold every offset back to the base; integer math
                        // sidesteps that, and volatile forces the real load (the
                        // data is invisible) and store (read only as code).
                        let src = self.src() as usize;
                        let dst_addr = dst as usize;
                        let len = self.len();
                        let mut i = 0;
                        while i < len {
                            let b = ::core::ptr::read_volatile((src + i) as *const u8);
                            ::core::ptr::write_volatile((dst_addr + i) as *mut u8, b);
                            i += 1;
                        }
                        ::core::mem::transmute::<*const u8, Self::Fn>(dst as *const u8)
                    }
                }
            }
        }

        #[allow(non_upper_case_globals)]
        #vis const #name: #name::Handle = #name::Handle;
    }
    .into()
}
