use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::Type;
use frand_property_build::parser::Model;
use crate::common::resolve_type;

pub fn generate(input: &Model) -> TokenStream {
    let vis = &input.vis;
    let model_name = &input.model_name;

    let field_defs = generate_field_defs(input);
    let init_fields = generate_init_fields(input);
    let init_logic = quote! { #(#init_fields),* };

    let sender_name = format_ident!("{}Sender", model_name);
    let receiver_name = format_ident!("{}Receiver", model_name);
    let instance_ident = format_ident!("{}_INSTANCE", model_name.to_string().to_uppercase());

    let sender_field_defs = generate_sender_field_defs(input);
    let receiver_field_defs = generate_receiver_field_defs(input);
    
    let clone_sender_logic = generate_clone_sender_logic(input);
    let clone_receiver_logic = generate_clone_receiver_logic(input);

    let (new_ret_ty, static_ret_ty, new_body) = if let Some(len) = &input.len {
        (
            quote! { std::sync::Arc<[Self]> },
            quote! { std::sync::Arc<[#model_name]> },
            quote! {
                let weak = ();
                let mut models = std::vec::Vec::with_capacity(#len);
                for _ in 0..#len {
                    models.push(Self {
                        #init_logic
                    });
                }
                models.into()
            }
        )
    } else {
        (
            quote! { std::sync::Arc<Self> },
            quote! { std::sync::Arc<#model_name> },
            quote! {
                let weak = ();
                std::sync::Arc::new(Self {
                    #init_logic
                })
            }
        )
    };

    if let syn::Visibility::Inherited = vis {
        // Private: Singleton Pattern
        quote! {
            static #instance_ident: std::sync::OnceLock<#static_ret_ty> = std::sync::OnceLock::new();

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
                pub fn clone_singleton() -> #new_ret_ty {
                    #instance_ident.get_or_init(|| {
                        #new_body
                    }).clone()
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
    } else {
        // Public: New Instance Pattern
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
                 pub fn new() -> #new_ret_ty {
                    #new_body
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
}

fn generate_sender_field_defs(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let f_vis = &f.vis;

        let (is_array, elem_ty) = if let Type::Array(arr) = f_ty {
            (true, arr.elem.as_ref())
        } else if let Type::Slice(slice) = f_ty {
            (true, slice.elem.as_ref())
        } else {
            (false, f_ty)
        };
        
        let resolved_ty = resolve_type(elem_ty);

        if f.is_model {
             if is_array {
                quote! { #f_vis #f_name: std::sync::Arc<[<#resolved_ty as frand_property::Model>::Sender]> }
             } else {
                 quote! { #f_vis #f_name: <#resolved_ty as frand_property::Model>::Sender }
             }
        } else {
            if is_array {
                quote! { #f_vis #f_name: std::sync::Arc<[frand_property::Sender<#resolved_ty>]> }
            } else {
                quote! { #f_vis #f_name: frand_property::Sender<#resolved_ty> }
            }
        }
    }).collect()
}

fn generate_receiver_field_defs(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let f_vis = &f.vis;

        let (is_array, elem_ty) = if let Type::Array(arr) = f_ty {
            (true, arr.elem.as_ref())
        } else if let Type::Slice(slice) = f_ty {
            (true, slice.elem.as_ref())
        } else {
            (false, f_ty)
        };

        let resolved_ty = resolve_type(elem_ty);

        if f.is_model {
             if is_array {
                quote! { #f_vis #f_name: std::sync::Arc<[<#resolved_ty as frand_property::Model>::Receiver]> }
             } else {
                 quote! { #f_vis #f_name: <#resolved_ty as frand_property::Model>::Receiver }
             }
        } else {
            if is_array {
                quote! { #f_vis #f_name: std::sync::Arc<[frand_property::Receiver<#resolved_ty>]> }
            } else {
                quote! { #f_vis #f_name: frand_property::Receiver<#resolved_ty> }
            }
        }
    }).collect()
}

fn generate_clone_sender_logic(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        
        let is_array = matches!(f_ty, Type::Array(_) | Type::Slice(_));

        if f.is_model {
            if is_array {
                quote! {
                    #f_name: self.#f_name.iter().map(|p| p.clone_sender()).collect::<std::vec::Vec<_>>().into()
                }
            } else {
                quote! {
                    #f_name: self.#f_name.clone_sender()
                }
            }
        } else {
            if is_array {
                quote! {
                    #f_name: self.#f_name.iter().map(|p| p.sender().clone()).collect::<std::vec::Vec<_>>().into()
                }
            } else {
                quote! {
                    #f_name: self.#f_name.sender().clone()
                }
            }
        }
    }).collect()
}

fn generate_clone_receiver_logic(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        
        let is_array = matches!(f_ty, Type::Array(_) | Type::Slice(_));

        if f.is_model {
            if is_array {
                quote! {
                    #f_name: self.#f_name.iter().map(|p| p.clone_receiver()).collect::<std::vec::Vec<_>>().into()
                }
            } else {
                quote! {
                    #f_name: self.#f_name.clone_receiver()
                }
            }
        } else {
            if is_array {
                quote! {
                    #f_name: self.#f_name.iter().map(|p| p.receiver().clone()).collect::<std::vec::Vec<_>>().into()
                }
            } else {
                quote! {
                    #f_name: self.#f_name.receiver().clone()
                }
            }
        }
    }).collect()
}

fn generate_field_defs(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let f_vis = &f.vis;

        let (is_array, elem_ty) = if let Type::Array(arr) = f_ty {
            (true, arr.elem.as_ref())
        } else if let Type::Slice(slice) = f_ty {
            (true, slice.elem.as_ref())
        } else {
            (false, f_ty)
        };
        
        let resolved_ty = resolve_type(elem_ty);

        if f.is_model {
             if is_array {
                if let Type::Array(_) = f_ty {
                      proc_macro_error::abort!(f_name, "Model fields must use implicit length syntax `[]`. Explicit length `[N]` is not allowed for models.");
                }
                quote! { #f_vis #f_name: std::sync::Arc<[#resolved_ty]> }
             } else {
                 quote! { #f_vis #f_name: std::sync::Arc<#resolved_ty> }
             }
        } else {
            if is_array {
                if let Type::Slice(_) = f_ty {
                     proc_macro_error::abort!(f_name, "Value fields must use explicit length syntax `[N]`. Implicit length `[]` is not allowed.");
                }
                quote! { #f_vis #f_name: std::sync::Arc<[frand_property::Property<#resolved_ty>]> }
            } else {
                quote! { #f_vis #f_name: frand_property::Property<#resolved_ty> }
            }
        }
    }).collect()
}

fn generate_init_fields(input: &Model) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        
        let (is_array, elem_ty, array_len) = if let Type::Array(arr) = f_ty {
             (true, arr.elem.as_ref(), Some(&arr.len))
        } else if let Type::Slice(slice) = f_ty {
             (true, slice.elem.as_ref(), None)
        } else {
             (false, f_ty, None)
        };
        
        let resolved_ty = resolve_type(elem_ty);

        if f.is_model {
             if is_array {
                if let Some(len) = array_len {
                    quote! {
                        #f_name: {
                            let mut models = std::vec::Vec::with_capacity(#len);
                            for _ in 0..#len {
                                models.push((*#resolved_ty::new()).clone());
                            }
                            models.into()
                        }
                    }
                } else {
                    quote! {
                        #f_name: #resolved_ty::new()
                    }
                }
             } else {
                 quote! {
                     #f_name: #resolved_ty::new()
                 }
             }
        } else {
            if is_array {
                let len = array_len.unwrap();
                quote! {
                    #f_name: {
                        let mut props = std::vec::Vec::with_capacity(#len);
                        for _ in 0..#len {
                            props.push(frand_property::Property::<#resolved_ty>::new(
                                weak.clone(),
                                Default::default(),
                                |_, _| {}
                            ));
                        }
                        props.into()
                    }
                }
            } else {
                quote! {
                    #f_name: frand_property::Property::<#resolved_ty>::new(
                        weak.clone(),
                        Default::default(),
                        |_, _| {}
                    )
                }
            }
        }
    }).collect()
}
