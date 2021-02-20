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

        let mut x: Vec<&syn::Pat> = em.arms.iter().map(|a| &a.pat).collect();
        if let Some((last, elems)) = x.split_last() {
            let wild = elems.iter().find_map(|pat| match pat {
                syn::Pat::Wild(w) => Some(w),
                _ => None,
            });
            match wild {
                Some(w) => {
                    // 最後以外にwildがあるので、エラー
                    self.err = Some(w.to_token_stream());
                    return syn::visit_mut::visit_expr_match_mut(self, em);
                }
                None => match last {
                    syn::Pat::Wild(_) => x = elems.into(),
                    _ => {}
                },
            }
        }
        let mut original = x.iter().map(|p| match p {
            syn::Pat::TupleStruct(_) => Ok(p),
            syn::Pat::Path(_) => Ok(p),
            syn::Pat::Ident(_) => Ok(p),
            // syn::Pat::Wild(w) => Ok(&w.),
            _p => Err(_p.span()),
        });
        let xxx = x.iter().map(|p| matches!(p, syn::Pat::TupleStruct(_)));
        let original: Result<Vec<&syn::Pat>, proc_macro2::Span> =
            original.try_fold(Vec::new(), |mut acc, x| {
                x.and_then(|xx| {
                    acc.push(*xx);
                    Ok(acc)
                })
            });

        let r = match original {
            Ok(original) => {
                let mut sorted = original.clone();
                sorted.sort_by(|l, r| {
                    let l = l.to_token_stream().to_string();
                    let r = r.to_token_stream().to_string();
                    l.cmp(&r)
                });

                original
                    .iter()
                    .zip(sorted.iter())
                    .try_fold((), |_, (o, s)| {
                        let oo = show_str(o);
                        let ss = show_str(s);
                        if oo == ss {
                            Ok(())
                        } else {
                            Err((s, format!("{} should sort before {}", ss, oo)))
                        }
                    })
                    .map_err(|(p, s)| {
                        let e = match p {
                            syn::Pat::TupleStruct(pts) => Some(Error::new_spanned(&pts.path, s)),
                            syn::Pat::Path(pp) => Some(Error::new_spanned(&pp.path, s)),
                            syn::Pat::Ident(i) => Some(Error::new_spanned(i, s)),
                            _ => None,
                        };
                        e.unwrap().into_compile_error()
                    })
            }
            Err(s) => {
                Err(Error::new(
                    s,
                    // TODO ここは変数から取得する
                    format!("unsupported by #[sorted]"),
                )
                .into_compile_error())
            }
        };
        if let Err(e) = r {
            self.err = Some(e);
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

fn show_str(p: &syn::Pat) -> String {
    match p {
        syn::Pat::TupleStruct(pts) => show_path_str(&pts.path),
        syn::Pat::Path(pp) => show_path_str(&pp.path),
        syn::Pat::Ident(i) => i.ident.to_string(),
        _ => String::new(),
    }
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
