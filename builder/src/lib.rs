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
    let q = quote! {
        use std::error::Error;
        pub struct #builder_name {
            #f
        }
        impl #builder_name {
            #setter
            pub fn build(&mut self) -> Result<#input_indent, Box<dyn Error>> {
                match (self.executable.take(), self.args.take(), self.env.take(), self.current_dir.take()){
                    (Some(ex),Some(a),Some(ev),Some(cd)) =>Ok(
                        #input_indent{
                            executable: ex,
                            args: a,
                            env: ev,
                            current_dir: cd,
                        }
                    ),
                    _ =>  Err("a")?, // 手抜き
                }
            }
        }

        impl #input_indent {
            pub fn builder() -> #builder_name {
                #builder_name {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
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
