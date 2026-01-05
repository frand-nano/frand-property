use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;
use super::parser::Model;
use crate::common::resolve_type;

pub fn generate(input: &Model) -> TokenStream {
    let vis = &input.vis;
    let model_name = &input.model_name;

    let field_defs = generate_field_defs(input);
    let init_fields = generate_init_fields(input);
    let init_logic = quote! { #(#init_fields),* };

    quote! {
        #[derive(Debug, Clone)]
        #vis struct #model_name {
            #(#field_defs),*
        }

        impl #model_name {
            pub fn new() -> Self {
                let weak = std::sync::Arc::new(());
                Self {
                    #init_logic
                }
            }

            pub fn new_array<const LEN: usize>() -> Vec<Self> {
                let weak = std::sync::Arc::new(());
                let mut models = Vec::with_capacity(LEN);
                for _ in 0..LEN {
                    models.push(Self {
                        #init_logic
                    });
                }
                models
            }
        }
    }
}

fn generate_field_defs(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let f_vis = &f.vis;

        if let Type::Array(arr) = f_ty {
            let elem_ty = &arr.elem;
            let resolved_elem_ty = resolve_type(elem_ty);
            quote! { #f_vis #f_name: Vec<frand_property::Property<#resolved_elem_ty>> }
        } else {
            let resolved_ty = resolve_type(f_ty);
            quote! { #f_vis #f_name: frand_property::Property<#resolved_ty> }
        }
    }).collect()
}

fn generate_init_fields(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;

        if let Type::Array(arr) = f_ty {
            let len = &arr.len;
            let elem_ty = &arr.elem;
            let resolved_elem_ty = resolve_type(elem_ty);

            quote! {
                #f_name: {
                    let mut props = Vec::with_capacity(#len);
                    for _ in 0..#len {
                        props.push(frand_property::Property::<#resolved_elem_ty>::new(
                            weak.clone(),
                            Default::default(),
                            |_, _| {}
                        ));
                    }
                    props
                }
            }
        } else {
            let resolved_ty = resolve_type(f_ty);
            quote! {
                #f_name: frand_property::Property::<#resolved_ty>::new(
                    weak.clone(),
                    Default::default(),
                    |_, _| {}
                )
            }
        }
    }).collect()
}
