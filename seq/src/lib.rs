use proc_macro::TokenStream;
use proc_macro2::{Group, Ident, Literal};
use quote::quote;
use syn::parse_macro_input;
use syn::{parse::Parse, Token};

struct Seq {
    n_indet: Ident,
    start: Literal,
    end: Literal,
    group: Group,
}

impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let n_ident: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let start: Literal = input.parse()?;
        input.parse::<Token![..]>()?;
        let end: Literal = input.parse()?;
        let group: Group = input.parse()?;

        Ok(Seq {
            n_indet: n_ident,
            start: start,
            end: end,
            group: group,
        })
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let Seq {
        n_indet,
        start,
        end,
        group,
    } = parse_macro_input!(input as Seq);

    let _ = n_indet;
    let _ = start;
    let _ = end;
    let _ = group;

    let out = quote! {};
    out.into()
}
