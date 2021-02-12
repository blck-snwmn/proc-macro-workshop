use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, AttributeArgs, Error, Item, ItemEnum};

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
