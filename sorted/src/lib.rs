use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemEnum};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    let args = parse_macro_input!(args as AttributeArgs);

    let _ = args;

    let q = quote! {
        #input
    };

    q.into()
}
