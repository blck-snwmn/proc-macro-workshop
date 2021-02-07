use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::error::Error;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, FieldsNamed, GenericArgument, PathArguments, Type,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_indent = input.ident;
    let builder_name = format_ident!("{}Builder", input_indent);
    let builder_fields = quote_builder_fields(&input.data).unwrap();
    let setters = quote_setter(&input.data).unwrap();
    let init_builder = quote_init_for_builder_fields(&input.data).unwrap();
    let build_fields = quote_build_fields(&input.data).unwrap();
    let q = quote! {
        use std::error::Error;
        pub struct #builder_name {
            #builder_fields
        }
        impl #builder_name {
            #setters

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
                if contain_option_type(&x.ty) {
                    let inty = extract_option_generics_type(&x.ty).unwrap();
                    quote! {
                        pub fn #name(&mut self, #name: #inty) ->&mut Self{
                            self.#name =  std::option::Option::Some(#name);
                            self
                        }
                    }
                } else {
                    quote! {
                        pub fn #name(&mut self, #name: #ty) ->&mut Self{
                            self.#name =  std::option::Option::Some(#name);
                            self
                        }
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
                if contain_option_type(&x.ty) {
                    // builderのフィールドはすべてOption型にする
                    // 対象のフィールドがOptionの場合、builderのフィールドをOptionで包む必要がないので、そのまま指定
                    quote! {
                        #name: #ty
                    }
                } else {
                    quote! {
                        #name: std::option::Option<#ty>
                    }
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
                if contain_option_type(&x.ty) {
                    quote! {

                        #name: self.#name.take()
                    }
                } else {
                    quote! {
                        #name: self.#name.take().unwrap()
                    }
                }
            });
            return quote! {
                #(#xx,)*
            };
        });
    }
    Err("a")?
}

fn contain_option_type(ty: &Type) -> bool {
    if let Type::Path(p) = ty {
        return p.path.segments.iter().any(|ps| ps.ident == "Option");
    }
    false
}

fn extract_option_generics_type(ty: &Type) -> Result<&Type, Box<dyn Error>> {
    if let Type::Path(p) = ty {
        let arg = p.path.segments.iter().find(|ps| ps.ident == "Option");
        let ex = arg
            .map(|x| {
                if let PathArguments::AngleBracketed(garg) = &x.arguments {
                    if let GenericArgument::Type(t) = garg.args.first().unwrap() {
                        return Some(t);
                    }
                }
                None
            })
            .flatten();
        if let Some(t) = ex {
            return Ok(t);
        }
    }
    Err("a")?
}
