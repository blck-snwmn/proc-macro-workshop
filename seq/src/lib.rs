use std::iter::FromIterator;

use proc_macro::TokenStream;
use proc_macro2::{Group, Ident, Literal, TokenStream as TokenStream2, TokenTree};
use quote::{format_ident, quote};
use syn::{__private::str, parse_macro_input};
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
        // eprintln!("{:?}", stream);
        Ok(Seq {
            n_indet: n_ident,
            start: start,
            end: end,
            stream: stream,
        })
    }
}

fn parse_tree(
    tt: TokenTree,
    next: Option<TokenTree>,
    peek: Option<TokenTree>,
    old: &str,
    new: u64,
) -> (Option<TokenTree>, bool) {
    match &tt {
        TokenTree::Group(g) => (
            Some(Group::new(g.delimiter(), parse(g.stream(), old, new)).into()),
            false,
        ),
        TokenTree::Ident(i) => {
            let with_sharp = matches!(next, Some(TokenTree::Punct(p)) if p.to_string() == "#");
            let with_ident = matches!(&peek, Some(TokenTree::Ident(pi)) if pi.to_string() == old);
            if with_sharp && with_ident {
                let new_ident = format_ident!("{}{}", i.to_string(), new);
                (Some(TokenTree::from(new_ident)), true)
            } else if i.to_string() == old {
                (Some(TokenTree::from(Literal::u64_unsuffixed(new))), false)
            } else {
                (Some(tt), false)
            }
        }
        _ => (Some(tt), false),
    }
}

fn parse(stream: TokenStream2, old: &str, new: u64) -> TokenStream2 {
    let mut stream = stream.into_iter().peekable();

    let mut v = Vec::new();
    while let Some(tt) = stream.next() {
        // 次とその次の要素から判定するため
        let mut c = stream.clone();
        let next = c.next();
        let peek = c.next();

        if let (Some(tt), consumed) = parse_tree(tt, next, peek, old, new) {
            v.push(tt);
            if consumed {
                // 消費したので、パースの対象から外す
                stream.next();
                stream.next();
            }
        }
    }
    TokenStream2::from_iter(v)
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

    // eprintln!("{}", out);

    out.into()
}
