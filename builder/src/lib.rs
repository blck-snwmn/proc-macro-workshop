use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::error::Error;
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsNamed};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_indent = input.ident;
    let builder_name = format_ident!("{}Builder", input_indent);
    let f = quote_builder_fields(&input.data).unwrap();
    let setter = quote_setter(&input.data).unwrap();
    let init_builder = quote_init_for_builder_fields(&input.data).unwrap();
    let build_fields = quote_build_fields(&input.data).unwrap();
    let q = quote! {
        use std::error::Error;
        pub struct #builder_name {
            #f
        }
        impl #builder_name {
            #setter
            pub fn build(&mut self) -> Result<#input_indent, Box<dyn Error>> {
                std::result::Result::Ok(#input_indent {
                    #build_fields
                })
            }
        }

        impl #input_indent {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #init_builder
                }
            }
        }
    };
    q.into()
}

fn fields(fields: &Fields) -> Result<&FieldsNamed, Box<dyn Error>> {
    if let Fields::Named(fnamed) = fields {
        return Ok(&fnamed);
    }
    Err("a")?
}

fn quote_setter(data: &Data) -> Result<proc_macro2::TokenStream, Box<dyn Error>> {
    if let Data::Struct(ds) = data {
        return fields(&ds.fields).map(|fnamed| {
            let xx = fnamed.named.iter().map(|x| {
                let name = &x.ident;
                let ty = &x.ty;
                quote! {
                    pub fn #name(&mut self, #name: #ty) ->&mut Self{
                        self.#name =  std::option::Option::Some(#name);
                        self
                    }
                }
            });
            return quote! {
                #(#xx)*
            };
        });
    }
    Err("a")?
}

fn quote_builder_fields(data: &Data) -> Result<proc_macro2::TokenStream, Box<dyn Error>> {
    if let Data::Struct(ds) = data {
        return fields(&ds.fields).map(|fnamed| {
            let xx = fnamed.named.iter().map(|x| {
                let name = &x.ident;
                let ty = &x.ty;
                // このような記載はできないので、注意
                // quote! {
                //     #x.ident: std::option::Option<#x.ty>
                // }
                quote! {
                    #name: std::option::Option<#ty>
                }
            });
            return quote! {
                #(#xx,)*
            };
        });
    }
    Err("a")?
}

fn quote_init_for_builder_fields(data: &Data) -> Result<proc_macro2::TokenStream, Box<dyn Error>> {
    if let Data::Struct(ds) = data {
        return fields(&ds.fields).map(|fnamed| {
            let xx = fnamed.named.iter().map(|x| {
                let name = &x.ident;
                quote! {
                    #name: std::option::Option::None
                }
            });
            return quote! {
                #(#xx,)*
            };
        });
    }
    Err("a")?
}

fn quote_build_fields(data: &Data) -> Result<proc_macro2::TokenStream, Box<dyn Error>> {
    if let Data::Struct(ds) = data {
        return fields(&ds.fields).map(|fnamed| {
            let xx = fnamed.named.iter().map(|x| {
                let name = &x.ident;
                quote! {
                    #name: self.#name.take().unwrap()
                }
            });
            return quote! {
                #(#xx,)*
            };
        });
    }
    Err("a")?
}
