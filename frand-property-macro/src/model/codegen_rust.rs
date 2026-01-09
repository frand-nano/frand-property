use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::Type;
use super::parser::Model;
use crate::common::resolve_type;

pub fn generate(input: &Model) -> TokenStream {
    let vis = &input.vis;
    let model_name = &input.model_name;

    let field_defs = generate_field_defs(input);
    let init_fields = generate_init_fields(input);
    let init_logic = quote! { #(#init_fields),* };

    let sender_name = format_ident!("{}Sender", model_name);
    let receiver_name = format_ident!("{}Receiver", model_name);

    let sender_field_defs = generate_sender_field_defs(input);
    let receiver_field_defs = generate_receiver_field_defs(input);
    
    let clone_sender_logic = generate_clone_sender_logic(input);
    let clone_receiver_logic = generate_clone_receiver_logic(input);

    quote! {
        #[derive(Debug, Clone)]
        #vis struct #model_name {
            #(#field_defs),*
        }
        
        #[derive(Debug, Clone)]
        #vis struct #sender_name {
            #(#sender_field_defs),*
        }

        #[derive(Debug, Clone)]
        #vis struct #receiver_name {
            #(#receiver_field_defs),*
        }

        impl #model_name {
            pub fn new() -> Self {
                Self::new_vec::<1>().pop().expect("Should have created at least one model")
            }

            pub fn new_vec<const LEN: usize>() -> Vec<Self> {
                let weak = ();
                let mut models = Vec::with_capacity(LEN);
                for _ in 0..LEN {
                    models.push(Self {
                        #init_logic
                    });
                }
                models
            }
            
            pub fn clone_sender(&self) -> #sender_name {
                #sender_name {
                    #(#clone_sender_logic),*
                }
            }

            pub fn clone_receiver(&self) -> #receiver_name {
                #receiver_name {
                    #(#clone_receiver_logic),*
                }
            }
        }

        impl frand_property::Model for #model_name {
            type Sender = #sender_name;
            type Receiver = #receiver_name;

            fn clone_sender(&self) -> Self::Sender {
                self.clone_sender()
            }

            fn clone_receiver(&self) -> Self::Receiver {
                self.clone_receiver()
            }
        }
    }
}

fn generate_sender_field_defs(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let f_vis = &f.vis;

        if let Type::Array(arr) = f_ty {
            let elem_ty = &arr.elem;
            let resolved_elem_ty = resolve_type(elem_ty);
            quote! { #f_vis #f_name: Vec<frand_property::Sender<#resolved_elem_ty>> }
        } else {
            let resolved_ty = resolve_type(f_ty);
            quote! { #f_vis #f_name: frand_property::Sender<#resolved_ty> }
        }
    }).collect()
}

fn generate_receiver_field_defs(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let f_vis = &f.vis;

        if let Type::Array(arr) = f_ty {
            let elem_ty = &arr.elem;
            let resolved_elem_ty = resolve_type(elem_ty);
            quote! { #f_vis #f_name: Vec<frand_property::Receiver<#resolved_elem_ty>> }
        } else {
            let resolved_ty = resolve_type(f_ty);
            quote! { #f_vis #f_name: frand_property::Receiver<#resolved_ty> }
        }
    }).collect()
}

fn generate_clone_sender_logic(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;

        if let Type::Array(_) = f_ty {
            quote! {
                #f_name: self.#f_name.iter().map(|p| p.sender().clone()).collect()
            }
        } else {
            quote! {
                #f_name: self.#f_name.sender().clone()
            }
        }
    }).collect()
}

fn generate_clone_receiver_logic(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;

        if let Type::Array(_) = f_ty {
            quote! {
                #f_name: self.#f_name.iter().map(|p| p.receiver().clone()).collect()
            }
        } else {
            quote! {
                #f_name: self.#f_name.receiver().clone()
            }
        }
    }).collect()
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
