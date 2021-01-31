use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;

    let q = quote! {
        impl Command {
            pub fn builder() {}
        }
    };
    q.into()
}
