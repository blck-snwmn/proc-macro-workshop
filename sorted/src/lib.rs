use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Error, Item};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Item);
    let args = parse_macro_input!(args as AttributeArgs);

    let _ = args;

    match &input {
        Item::Enum(_) => {}
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
