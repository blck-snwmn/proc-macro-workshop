use proc_macro::TokenStream;
use proc_macro2::{Group, Ident, Literal, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::parse_macro_input;
use syn::{parse::Parse, Token};

struct Seq {
    n_indet: Ident,
    start: u64,
    end: u64,
    stream: TokenStream2,
}

impl Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let n_ident: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let start: Literal = input.parse::<Literal>()?;
        let start = start
            .to_string()
            .parse::<u64>()
            .map_err(|e| syn::Error::new(start.span(), e.to_string()))?;
        input.parse::<Token![..]>()?;
        let end: Literal = input.parse()?;
        let end = end
            .to_string()
            .parse::<u64>()
            .map_err(|e| syn::Error::new(end.span(), e.to_string()))?;
        let stream = input.parse::<Group>()?.stream();
        Ok(Seq {
            n_indet: n_ident,
            start: start,
            end: end,
            stream: stream,
        })
    }
}
// struct Visitor;
// impl syn::visit_mut::VisitMut for Visitor {
//     fn visit_ident_mut(&mut self, i: &mut Ident) {

//         syn::visit_mut::visit_ident_mut(self, i)
//     }
// }

fn parse_tree(tt: TokenTree, old: &str, new: u64) -> TokenTree {
    match tt {
        TokenTree::Group(g) => Group::new(g.delimiter(), parse(g.stream(), old, new)).into(),
        TokenTree::Ident(ref i) => {
            let ident_str = i.to_string();
            if ident_str == old {
                TokenTree::from(Literal::u64_unsuffixed(new))
            } else {
                tt
            }
        }
        TokenTree::Punct(_) => tt,
        TokenTree::Literal(_) => tt,
    }
}

fn parse(stream: TokenStream2, old: &str, new: u64) -> TokenStream2 {
    stream
        .into_iter()
        .map(|tt| parse_tree(tt, old, new))
        .collect()
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let Seq {
        n_indet,
        start,
        end,
        stream,
    } = parse_macro_input!(input as Seq);

    let mut out = quote! {};
    for i in start..end {
        let stream = parse(stream.clone(), &n_indet.to_string(), i);
        out.extend(stream);
    }

    out.into()
}
