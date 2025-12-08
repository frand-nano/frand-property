use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;
use crate::parser::{Direction, SlintModel};
use proc_macro_error::abort;

pub fn generate(input: &SlintModel, doc_comment: TokenStream) -> TokenStream {
    let vis = &input.vis;
    let model_name = &input.model_name;
    let type_name = &input.type_name;

    let field_defs = generate_field_defs(input);
    let field_inits = generate_field_inits(input, type_name);
    let sender_defs = generate_sender_defs(input);
    let bindings = generate_bindings(input, type_name);
    let struct_init_fields = generate_struct_init_fields(input);

    quote! {
        #doc_comment
        #vis struct #model_name<C: slint::ComponentHandle> {
            #(#field_defs),*
        }

        impl<C: slint::ComponentHandle + 'static> #model_name<C> {
            pub fn new(component: &C) -> Self where for<'a> #type_name<'a>: slint::Global<'a, C> {
                let weak = std::sync::Arc::new(component.as_weak());

                #(#field_inits)*

                #(#sender_defs)*

                #(#bindings)*

                Self {
                    #(#struct_init_fields),*
                }
            }
        }
    }
}

fn generate_field_defs(input: &SlintModel) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_vis = &f.vis;
        let f_name = &f.name;
        let f_ty = &f.ty;

        let is_unit = if let Type::Tuple(t) = f_ty {
            t.elems.is_empty()
        } else {
            false
        };

        if is_unit {
            quote! {
                #f_vis #f_name: Property<slint::Weak<C>, ()>
            }
        } else {
            quote! {
                #f_vis #f_name: Property<slint::Weak<C>, #f_ty>
            }
        }
    }).collect()
}

fn generate_field_inits(input: &SlintModel, type_name: &syn::Ident) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let direction = &f.direction;

        let is_unit = if let Type::Tuple(t) = f_ty {
            t.elems.is_empty()
        } else {
            false
        };

        if is_unit && (*direction == Direction::Out || *direction == Direction::InOut) {
            abort!(f_name, "`()` type cannot be used with `out` or `in-out` direction");
        }

        let setter = if *direction == Direction::Out || *direction == Direction::InOut {
             if is_unit {
                 quote! { |_, _| {} }
             } else {
                 let set_ident = format_ident!("set_{}", f_name);
                 quote! {
                    |c, v| {
                        c.upgrade_in_event_loop(move |c| {
                            c.global::<#type_name>().#set_ident(v)
                        }).unwrap() // TODO: 에러 처리
                    }
                 }
             }
        } else {
            quote! { |_, _| {} }
        };

        quote! {
            let #f_name = Property::new(
                weak.clone(),
                Default::default(),
                #setter,
            );
        }
    }).collect()
}

fn generate_sender_defs(input: &SlintModel) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let direction = &f.direction;
        let sender_name = format_ident!("{}_sender", f_name);
        
        if *direction == Direction::Out {
            quote! {}
        } else {
            quote! {
                let #sender_name = #f_name.sender().clone();
            }
        }
    }).collect()
}

fn generate_bindings(input: &SlintModel, type_name: &syn::Ident) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let direction = &f.direction;
        let sender_name = format_ident!("{}_sender", f_name);

        if *direction == Direction::In || *direction == Direction::InOut {
            let is_unit = if let Type::Tuple(t) = f_ty {
                t.elems.is_empty()
            } else {
                false
            };

            if is_unit {
                let on_ident = format_ident!("on_{}", f_name);
                quote! {
                    component.global::<#type_name>().#on_ident(move || #sender_name.send(()));
                }
            } else {
                let on_changed_ident = format_ident!("on_changed_{}", f_name);
                quote! {
                    component.global::<#type_name>().#on_changed_ident(move |v| #sender_name.send(v));
                }
            }
        } else {
            quote! {}
        }
    }).collect()
}

fn generate_struct_init_fields(input: &SlintModel) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        quote! {
            #f_name
        }
    }).collect()
}
