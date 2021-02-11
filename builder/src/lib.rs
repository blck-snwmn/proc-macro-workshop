use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::result::Result;
use syn::{Data, DeriveInput, Error, Field, Fields, FieldsNamed, GenericArgument, Lit, Meta, PathArguments, Type, parse_macro_input};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_indent = input.ident;
    let builder_name = format_ident!("{}Builder", input_indent);
    let builder_fields = quote_builder_fields(&input.data);
    if builder_fields.is_err() {
        return builder_fields.err().unwrap().into();
    }
    let builder_fields = builder_fields.unwrap();
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

            pub fn build(&mut self) -> std::result::Result<#input_indent,  std::boxed::Box<dyn Error>> {
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

fn fields(fields: &Fields) -> Result<&FieldsNamed, proc_macro2::TokenStream> {
    if let Fields::Named(fnamed) = fields {
        return Ok(&fnamed);
    }
    Err(Error::new_spanned(quote! {"err"}, "a").into_compile_error())
}

fn quote_setter(data: &Data) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    if let Data::Struct(ds) = data {
        return fields(&ds.fields).map(|fnamed| {
            let xx = fnamed.named.iter().map(|x| {
                let name = &x.ident;
                let ty = &x.ty;

                let meta = extract_each_by_mae_name_value(x);
                let inner_type = extract_vector_generics_type(ty);
                if let (Some(m), Ok(t)) = (meta, inner_type) {
                    let filed_name = format_ident!("{}", m);
                    return quote! {
                        pub fn #filed_name(&mut self, #filed_name: #t) ->&mut Self{
                            self.#filed_name.push(#filed_name);
                            self
                        }
                    };
                }

                let args_type = if contain_option_type(&x.ty) {
                    extract_option_generics_type(&x.ty).unwrap()
                } else {
                    ty
                };
                quote! {
                    pub fn #name(&mut self, #name: #args_type) ->&mut Self{
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
    Err(Error::new_spanned(quote! {"err"}, "a").into_compile_error())
}

fn quote_builder_fields(data: &Data) -> Result<proc_macro2::TokenStream,proc_macro2::TokenStream> {
    if let Data::Struct(ds) = data {
        return fields(&ds.fields).and_then(|fnamed| {
            let xx = fnamed.named.iter().map(|x| {
                // let ty = &x.ty;

                // let meta = extract_each_by_mae_name_value(x);
                // let inner_type = extract_vector_generics_type(ty);
                // if let (Some(m), Ok(t)) = (meta, inner_type) {
                //     if !contain_each_by_mae_name_value(x).unwrap() {
                //         return Err(Error::new(x.span(), "test").to_compile_error());
                //     }
                //     let filed_name = format_ident!("{}", m);
                //     return Ok(quote! {
                //         #filed_name: Vec<#t>
                //     });
                // }

                // let name = &x.ident;
                // // このような記載はできないので、注意
                // // quote! {
                // //     #x.ident: std::option::Option<#x.ty>
                // // }
                // if contain_option_type(&x.ty) {
                //     // builderのフィールドはすべてOption型にする
                //     // 対象のフィールドがOptionの場合、builderのフィールドをOptionで包む必要がないので、そのまま指定
                //     Ok(quote! {
                //         #name: #ty
                //     })
                // } else {
                //     Ok(quote! {
                //         #name: std::option::Option<#ty>
                //     })
                // }
                define_fileds(x)
            });
            let xx = xx.fold(
                Ok(Vec::new()),
                 | acc: std::result::Result<
                    Vec<proc_macro2::TokenStream>,
                    proc_macro2::TokenStream,
                >,
                      x: std::result::Result<
                    proc_macro2::TokenStream,
                    proc_macro2::TokenStream,
                >| {
                    // if acc.is_err() {
                    //     return acc;
                    // }
                    // // if x.is_err() {
                    // //     return Err(x.err().unwrap());
                    // // }
                    x.and_then(|xx|{
                        acc.map(|mut v|{
                            v.push(xx);
                            v
                        })
                    })
                    
                    // let mut z = acc.unwrap();
                    // z.push(x.ok().unwrap());
                    // Ok(z)
                },
            );
           return xx.map(|r|quote! {
                #(#r,)*
            })
            // let r = xx.ok().unwrap();
            // return Ok(quote! {
            //     #(#r,)*
            // });
        });
    }
    Err(Error::new_spanned(quote! {"err"}, "a").into_compile_error())
}

fn quote_init_for_builder_fields(data: &Data) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    if let Data::Struct(ds) = data {
        return fields(&ds.fields).map(|fnamed| {
            let xx = fnamed.named.iter().map(|x| {
                let meta = extract_each_by_mae_name_value(x);
                if let Some(m) = meta {
                    let filed_name = format_ident!("{}", m);
                    return quote! {
                        #filed_name: Vec::new()
                    };
                }

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
    Err(Error::new_spanned(quote! {"err"}, "a").into_compile_error())
}

fn quote_build_fields(data: &Data) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    if let Data::Struct(ds) = data {
        return fields(&ds.fields).map(|fnamed| {
            let xx = fnamed.named.iter().map(|x| {
                let name = &x.ident;

                let meta = extract_each_by_mae_name_value(x);
                if let Some(m) = meta {
                    let filed_name = format_ident!("{}", m);
                    return quote! {
                        #name: self.#filed_name.clone()
                    };
                }

                // TODO ここはOptionを返すようにする
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
    Err(Error::new_spanned(quote! {"err"}, "a").into_compile_error())
}

fn contain_type_by(ty: &Type, ident: String) -> bool {
    if let Type::Path(p) = ty {
        return p.path.segments.iter().any(|ps| ps.ident == ident);
    }
    false
}

fn contain_option_type(ty: &Type) -> bool {
    contain_type_by(ty, "Option".to_owned())
}

fn extract_generics_type(ty: &Type, ident: String) -> Result<&Type, proc_macro2::TokenStream> {
    if let Type::Path(p) = ty {
        let arg = p.path.segments.iter().find(|ps| ps.ident == ident);
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
    Err(Error::new_spanned(quote! {"err"}, "a").into_compile_error())
}

fn extract_option_generics_type(ty: &Type) -> Result<&Type, proc_macro2::TokenStream> {
    extract_generics_type(ty, "Option".to_owned())
}

fn extract_vector_generics_type(ty: &Type) -> Result<&Type, proc_macro2::TokenStream> {
    extract_generics_type(ty, "Vec".to_owned())
}

fn extract_meta(field: &Field) -> Option<Meta> {
    field.attrs.iter().find_map(|atr| {
        let r = atr.parse_meta();

        match r {
            Ok(r) => Some(r),
            _ => None,
        }
    })
}

fn extract_each_by_mae_name_value(field: &Field) -> Option<String> {
    extract_meta(field).and_then(|meta| match meta {
        Meta::List(mlist) => mlist.nested.iter().find_map(|x| match x {
            syn::NestedMeta::Meta(m) => match m {
                Meta::NameValue(mnv) => match mnv.lit {
                    Lit::Str(ref s) => Some(s.value()),
                    _ => None,
                },
                _ => None,
            },
            syn::NestedMeta::Lit(_) => None,
        }),
        _ => None,
    })
}

fn define_fileds(field: &Field) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream>{
    match field.attrs.first(){
        Some(atr) => {
            let x = atr.parse_meta().map_err(|e|e.to_compile_error()).and_then(|meta| match meta {
                Meta::List(mlist) => {
                    let nested_meta = mlist.nested.first();
                    match nested_meta {
                        Some( syn::NestedMeta::Meta(syn::Meta::NameValue(meta_name_value))) if meta_name_value.path.is_ident("each") =>{
                            match meta_name_value.lit{
                            Lit::Str(ref s) => {
                                let filed_name = format_ident!("{}", s.value()); 
                                let inner_type = extract_vector_generics_type(&field.ty).unwrap();
                                Ok(quote! {
                                    #filed_name: Vec<#inner_type>
                                })
                            },
                            _ => Err(Error::new_spanned(quote! {"err"}, "a").into_compile_error()),
                        }},
                        _ => {
                            Err(Error::new_spanned(mlist, "expected `builder(each = \"...\")`").into_compile_error())
                        },
                    }
                },
                _ => Err(Error::new_spanned(quote! {"err"}, "a").into_compile_error()),
            });
            x
        },
        None => {
            let name = &field.ident;
            let ty = &field.ty;
            if contain_option_type(&ty) {
                // builderのフィールドはすべてOption型にする
                // 対象のフィールドがOptionの場合、builderのフィールドをOptionで包む必要がないので、そのまま指定
                Ok(quote! {
                    #name: #ty
                })
            } else {
                Ok(quote! {
                    #name: std::option::Option<#ty>
                })
            }   
        },
    }
}
