use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, AttributeArgs, Error, Item, ItemEnum, Variant};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);
    let args = parse_macro_input!(args as AttributeArgs);

    let _ = args;

    match &input {
        Item::Enum(item) => {
            let sorted = sorted_variants(item);
            if sorted.is_err() {
                return sorted.err().unwrap().into();
            }
        }
        _ => {
            // TODO この部分は関数として切り出すこと
            return Error::new_spanned(quote! {"#[sorted]"}, "expected enum or match expression")
                .to_compile_error()
                .into();
        }
    }

    let q = quote! {
        #input
    };

    q.into()
}
// ソートされているかチェック
fn sorted_variants(item: &ItemEnum) -> Result<Option<&Variant>, TokenStream2> {
    item.variants.iter().fold(
        Ok(None),
        |acc_before: Result<Option<&Variant>, TokenStream2>, now| {
            acc_before.and_then(|x| match x {
                Some(before) => {
                    let before_indent = before.ident.to_string();
                    let now_indent = now.ident.to_string();
                    if before_indent <= now_indent {
                        Ok(Some(now))
                    } else {
                        Err(Error::new(
                            now.span(),
                            // TODO ここは変数から取得する
                            format!("SomethingFailed should sort before ThatFailed"),
                        )
                        .into_compile_error())
                    }
                }
                None => Ok(Some(now)),
            })
        },
    )
}
