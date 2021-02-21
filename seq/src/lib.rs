use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let _ = input;

    let out = quote! {};
    out.into()
}
