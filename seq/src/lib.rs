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
    range: &RepeatRange,
    next: Option<TokenTree>,
    peek: Option<TokenTree>,
    old: &str,
    new: u64,
) -> (Option<TokenTree>, bool, bool) {
    match &tt {
        TokenTree::Group(g) => {
            let (s, b) = parse(g.stream(), range, old, new);
            (Some(Group::new(g.delimiter(), s).into()), false, b)
        }
        TokenTree::Ident(i) => {
            let with_sharp = matches!(next, Some(TokenTree::Punct(p)) if p.to_string() == "#");
            let with_ident = matches!(&peek, Some(TokenTree::Ident(pi)) if pi.to_string() == old);
            if with_sharp && with_ident {
                let new_ident = format_ident!("{}{}", i.to_string(), new);
                (Some(TokenTree::from(new_ident)), true, false)
            } else if i.to_string() == old {
                (
                    Some(TokenTree::from(Literal::u64_unsuffixed(new))),
                    false,
                    false,
                )
            } else {
                (Some(tt), false, false)
            }
        }
        TokenTree::Punct(p) if p.to_string() == "#" => {
            // matches!(next,Some(TokenTree::Group(g)) if matches!(g.delimiter(), proc_macro2::Delimiter::Parenthesis) );
            match next {
                Some(TokenTree::Group(g))
                    if matches!(g.delimiter(), proc_macro2::Delimiter::Parenthesis) =>
                {
                    let s = parse_range(g.stream(), range, old);
                    (
                        Some(TokenTree::Group(proc_macro2::Group::new(
                            proc_macro2::Delimiter::None,
                            s,
                        ))),
                        false,
                        true,
                    )
                }

                _ => (Some(tt), false, false),
            }
        }
        _ => (Some(tt), false, false),
    }
}

fn parse(stream: TokenStream2, range: &RepeatRange, old: &str, new: u64) -> (TokenStream2, bool) {
    let mut stream = stream.into_iter().peekable();

    let mut v = Vec::new();

    let mut rage_break = false;
    while let Some(tt) = stream.next() {
        // 次とその次の要素から判定するため
        let mut c = stream.clone();
        let next = c.next();
        let peek = c.next();

        if let (Some(tt), consumed, rb) = parse_tree(tt, range, next, peek, old, new) {
            rage_break = rb;
            v.push(tt);
            if consumed {
                // 消費したので、パースの対象から外す
                stream.next();
                stream.next();
            }
            if rage_break {
                stream.next();
                stream.next();
            }
        }
    }
    (TokenStream2::from_iter(v), rage_break)
}

fn parse_range(stream: TokenStream2, range: &RepeatRange, old: &str) -> TokenStream2 {
    let mut out = quote! {};
    for i in range.start..range.end {
        let (stream, range_break) = parse(stream.clone(), range, old, i);
        out.extend(stream);
        if range_break {
            break;
        }
    }
    out
}
struct RepeatRange {
    start: u64,
    end: u64,
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let Seq {
        n_indet,
        start,
        end,
        stream,
    } = parse_macro_input!(input as Seq);

    let range = RepeatRange {
        start: start,
        end: end,
    };
    let out = parse_range(stream.clone(), &range, &n_indet.to_string());

    out.into()
}
