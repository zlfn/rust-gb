//! The `#[sram]` attribute. See the `gb-pak` crate for what it expands to.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Ident, LitInt, Token, Type, Visibility, parse_macro_input};

/// Where the expansion looks for `gb-pak`, whether the invoking crate depends on
/// it directly or reaches it through the `gb` facade.
fn pak_root() -> TokenStream2 {
    use proc_macro_crate::{FoundCrate, crate_name};
    let path = match crate_name("gb-pak") {
        Ok(FoundCrate::Itself) => "crate".to_string(),
        Ok(FoundCrate::Name(n)) => format!("::{}", n.replace('-', "_")),
        Err(_) => match crate_name("rust-gb").or_else(|_| crate_name("gb")) {
            Ok(FoundCrate::Itself) => "crate::__pak".to_string(),
            Ok(FoundCrate::Name(n)) => format!("::{}::__pak", n.replace('-', "_")),
            Err(_) => "::gb_pak".to_string(),
        },
    };
    path.parse().expect("runtime root path")
}

/// `static NAME: Ty;`, with no initializer: the bytes are the cartridge's, not
/// the program's.
struct Decl {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    ty: Type,
}

impl Parse for Decl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        if input.peek(Token![=]) {
            return Err(input.error(
                "SRAM keeps what the cartridge already holds, so it has no initializer",
            ));
        }
        input.parse::<Token![;]>()?;
        Ok(Decl { attrs, vis, name, ty })
    }
}

/// Place a value at the base of an SRAM bank.
///
/// ```ignore
/// #[sram(0)]
/// static FILE: Save;
/// ```
///
/// The type must derive `zerocopy::FromBytes` and the bank number is required. See the
/// `sram` module of `gb-pak` for both.
#[proc_macro_attribute]
pub fn sram(args: TokenStream, item: TokenStream) -> TokenStream {
    if args.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[sram] needs the bank to place this in, as in `#[sram(0)]`",
        )
        .to_compile_error()
        .into();
    }
    let bank = parse_macro_input!(args as LitInt);
    let Decl { attrs, vis, name, ty } = parse_macro_input!(item as Decl);
    let pak = pak_root();

    quote! {
        #(#attrs)*
        #vis static #name: #pak::sram::Sram<#ty, #bank> = {
            const _: () = assert!(
                #pak::sram::BANKS > 0,
                "this cartridge has no SRAM; see `ram_size` in header.toml",
            );
            const _: () = assert!(
                #pak::sram::BANKS == 0 || #bank < #pak::sram::BANKS,
                "this cartridge has no such SRAM bank",
            );
            const _: () = assert!(
                ::core::mem::size_of::<#ty>() <= #pak::sram::BANK_LEN,
                "value is larger than one 8 KiB SRAM bank",
            );
            unsafe { #pak::sram::Sram::declare() }
        };
    }
    .into()
}
