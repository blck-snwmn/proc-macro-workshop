use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, spanned::Spanned, visit_mut::VisitMut, AttributeArgs, Error, ExprMatch,
    Item, ItemEnum, Path,
};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut output = input.clone();

    let input = parse_macro_input!(input as Item);
    let args = parse_macro_input!(args as AttributeArgs);

    let _ = args;

    match &input {
        Item::Enum(item) => {
            if let Err(e) = sorted_variants(item) {
                output.extend(TokenStream::from(e));
            }
        }
        _ => {
            let e = Error::new_spanned(quote! {"#[sorted]"}, "expected enum or match expression")
                .to_compile_error();
            output.extend(TokenStream::from(e));
        }
    }

    output
}

// ソートされているかチェック
fn sorted_variants(item: &ItemEnum) -> Result<(), TokenStream2> {
    let origin: Vec<&Ident> = item.variants.iter().map(|v| &v.ident).collect();

    let mut sorted = origin.clone();
    sorted.sort();

    origin.iter().zip(sorted.iter()).try_fold((), |_, (o, s)| {
        if o.to_string() == s.to_string() {
            Ok(())
        } else {
            Err(Error::new(
                s.span(),
                // TODO ここは変数から取得する
                format!("{} should sort before {}", s.to_string(), o.to_string()),
            )
            .into_compile_error())
        }
    })
}

struct Visitor {
    err: Option<TokenStream2>,
}
impl syn::visit_mut::VisitMut for Visitor {
    fn visit_expr_match_mut(&mut self, em: &mut ExprMatch) {
        if !em.attrs.iter().any(|x| x.path.is_ident("sorted")) {
            return syn::visit_mut::visit_expr_match_mut(self, em);
        }
        // remove sorted attribute
        em.attrs.retain(|a| !a.path.is_ident("sorted"));

        let original: Vec<&syn::Path> = em
            .arms
            .iter()
            .filter_map(|a| match &a.pat {
                syn::Pat::TupleStruct(pts) => Some(&pts.path),
                syn::Pat::Path(pp) => Some(&pp.path),
                _ => {
                    println!("no exptected");
                    None
                }
            })
            .collect();
        let mut sorted = original.clone();
        sorted.sort_by(|l, r| {
            let l = l.to_token_stream().to_string();
            let r = r.to_token_stream().to_string();
            l.cmp(&r)
        });

        if let Err(ts) = original
            .iter()
            .zip(sorted.iter())
            .try_fold((), |_, (o, s)| {
                let oo = show_path_str(o);
                let ss = show_path_str(s);
                if oo == ss {
                    Ok(())
                } else {
                    Err(Error::new_spanned(
                        s,
                        // TODO ここは変数から取得する
                        format!("{} should sort before {}", ss, oo),
                    )
                    .into_compile_error())
                }
            })
        {
            self.err = Some(ts);
        }

        syn::visit_mut::visit_expr_match_mut(self, em);
    }
}

#[proc_macro_attribute]
pub fn check(_: TokenStream, input: TokenStream) -> TokenStream {
    // let output = input.clone();

    let mut input = parse_macro_input!(input as syn::ItemFn);
    let mut v = Visitor { err: None };
    v.visit_item_fn_mut(&mut input);

    let mut output: TokenStream = (quote! {#input}).into();

    if let Some(e) = v.err {
        output.extend(TokenStream::from(e));
    }

    output
}

// Pathの文字列形式を返す
// `path.to_token_stream().to_string()`で表示すると以下のように、コロンと文字の間にスペースが入るため、自作
//      in: foo::Bar -> out: foo :: Bar
fn show_path_str(p: &Path) -> String {
    let leading_colon = p.leading_colon.map_or("", |_| "::").to_string();
    let x = p
        .segments
        .iter()
        .map(|x| x.to_token_stream().to_string())
        .collect::<Vec<String>>()
        .join("::");
    leading_colon + &x
}
