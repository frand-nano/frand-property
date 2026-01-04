use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;
use super::parser;
use parser::{Direction, SlintModel};
use crate::common::{resolve_type, is_special_string_type, is_unit_ty};



pub fn generate(input: &SlintModel, doc_comment: TokenStream) -> TokenStream {
    let vis = &input.vis;
    let model_name = &input.model_name;
    let type_name = &input.type_name;
    let struct_data_name = type_name;
    let global_type_name = format_ident!("{}Global", type_name);

    let field_defs = generate_field_defs(input);
    let body_logic = generate_body_logic(input, &global_type_name, struct_data_name);

    // 반환 타입 결정
    let (return_type, return_expr) = if input.array_len.is_some() {
        (
            quote! { Vec<Self> },
            quote! { #body_logic }
        )
    } else {
        (
            quote! { Self },
            quote! { 
                { #body_logic }
            }
        )
    };

    quote! {
        #doc_comment
        #vis struct #model_name<C: slint::ComponentHandle> {
            #(#field_defs),*
        }

        impl<C: slint::ComponentHandle + 'static> #model_name<C> {
            pub fn new(component: &C) -> #return_type where for<'a> #global_type_name<'a>: slint::Global<'a, C> {
                use slint::Model as _;
                let weak = std::sync::Arc::new(component.as_weak());

                #return_expr
            }
        }
    }
}

fn generate_field_defs(input: &SlintModel) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_vis = &f.vis;
        let f_name = &f.name;
        let f_ty = &f.ty;
        let direction = &f.direction;
        let is_unit = is_unit_ty(f_ty);

        if let Type::Array(arr) = f_ty {
            let elem_ty = &arr.elem;
            let resolved_elem_ty = resolve_type(elem_ty);
            if *direction == Direction::Out {
                quote! { #f_vis #f_name: Vec<frand_property::Sender<slint::Weak<C>, #resolved_elem_ty>> }
            } else {
                 quote! { #f_vis #f_name: Vec<frand_property::Receiver<#resolved_elem_ty>> }
            }
        } else {
            if is_unit {
                if *direction == Direction::Out {
                     quote! { #f_vis #f_name: frand_property::Sender<slint::Weak<C>, ()> }
                } else {
                     quote! { #f_vis #f_name: frand_property::Receiver<()> }
                }
            } else {
                let resolved_ty = resolve_type(f_ty);
                if *direction == Direction::Out {
                    quote! { #f_vis #f_name: frand_property::Sender<slint::Weak<C>, #resolved_ty> }
                } else {
                    quote! { #f_vis #f_name: frand_property::Receiver<#resolved_ty> }
                }
            }
        }
    }).collect()
}

fn generate_struct_init_fields(input: &SlintModel) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        quote! { #f_name }
    }).collect()
}

fn generate_body_logic(input: &SlintModel, global_type_name: &syn::Ident, struct_data_name: &syn::Ident) -> TokenStream {
    let fields: Vec<_> = input.fields.iter().collect();
    let (data_fields, signal_fields): (Vec<_>, Vec<_>) = fields.into_iter()
        .partition(|f| !is_unit_ty(&f.ty));

    if let Some(array_len) = &input.array_len {
        generate_array_logic(array_len, struct_data_name, global_type_name, &data_fields, &signal_fields, input)
    } else {
        generate_scalar_logic(struct_data_name, global_type_name, &data_fields, &signal_fields, input)
    }


}

fn generate_array_logic(
    array_len: &syn::Expr,
    struct_data_name: &syn::Ident,
    global_type_name: &syn::Ident,
    data_fields: &[&parser::SlintModelField],
    signal_fields: &[&parser::SlintModelField],
    input: &SlintModel
) -> TokenStream {
    let struct_init_ids = generate_struct_init_fields(input);
    let mut loop_body = Vec::new();

    let mut slint_data_fields_init = Vec::new();
    let mut rust_struct_fields_init = Vec::new();

    let mut signal_init_block = Vec::new();

    for f in signal_fields {
        let f_name = &f.name;
        
        if f.direction == Direction::Out {
             proc_macro_error::abort!(f_name, "`()` type cannot be used with `out` direction");
        }
        
        let f_senders = format_ident!("{}_senders", f_name);
        let f_receivers = format_ident!("{}_receivers", f_name);
        let on_ident = format_ident!("on_{}", f_name);
        
        signal_init_block.push(quote! {
            let mut #f_senders = Vec::with_capacity(#array_len);
            let mut #f_receivers = Vec::with_capacity(#array_len);
            for _ in 0..#array_len {
                 let prop = frand_property::Property::new(weak.clone(), Default::default(), |_,_| {});
                 #f_senders.push(prop.sender().clone());
                 #f_receivers.push(prop.receiver().clone());
            }
            
            let senders_clone = #f_senders.clone();
            component.global::<#global_type_name>().#on_ident(move |idx| {
                if let Some(s) = senders_clone.get(idx as usize) {
                    s.notify();
                }
            });
        });
        
        loop_body.push(quote! {
            let #f_name = #f_receivers[i].clone();
        });
        
        rust_struct_fields_init.push(quote! { #f_name });
    }

    for f in data_fields {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let f_prop = format_ident!("{}_prop", f_name);
        
        if let Type::Array(arr) = f_ty {
            let len = &arr.len;
            let f_senders = format_ident!("{}_senders", f_name);

            
            if f.direction == Direction::In {
                // 배열 IN: 각 요소에 대해 Property 생성
                let resolved_elem_ty = resolve_type(&arr.elem);
                let (setup, init) = generate_in_array_setup(f_name, len, &resolved_elem_ty);
                loop_body.push(setup);
                rust_struct_fields_init.push(quote! { #f_name });
                slint_data_fields_init.push(init);
            } else {
                let resolved_elem_ty = resolve_type(&arr.elem);
                loop_body.push(quote! {
                    let mut #f_senders = Vec::with_capacity(#len);
                    for j in 0..#len {
                        let prop = frand_property::Property::<slint::Weak<C>, #resolved_elem_ty>::new(
                             weak.clone(),
                             Default::default(),
                             move |c, v| {
                                 c.upgrade_in_event_loop(move |c| {
                                     let global = c.global::<#global_type_name>();
                                     let model = global.get_data();
                                     if let Some(data) = model.row_data(i) {
                                         data.#f_name.set_row_data(j, v);
                                     }
                                 }).unwrap();
                             }
                        );
                        #f_senders.push(prop.sender().clone());
                    }
                    let #f_name = #f_senders;
                });
                rust_struct_fields_init.push(quote! { #f_name });
                slint_data_fields_init.push(quote! {
                    #f_name: slint::ModelRc::new(std::rc::Rc::new(slint::VecModel::from(vec![Default::default(); #len])))
                });
            }
        } else {
            // 스칼라 로직
            if f.direction == Direction::In {
                loop_body.push(quote! {
                    let #f_prop = frand_property::Property::new(weak.clone(), Default::default(), |_, _| {});
                    let #f_name = #f_prop.receiver().clone();
                });
                 rust_struct_fields_init.push(quote! { #f_name });
                 slint_data_fields_init.push(quote! { #f_name: Default::default() });
            } else {
                let setter = quote! {
                     if let Some(mut data) = model.row_data(i) {
                          data.#f_name = v;
                          model.set_row_data(i, data);
                     }
                };
                let resolved_ty = resolve_type(f_ty);
                let out_prop_logic = generate_out_property(global_type_name, setter, resolved_ty);
                loop_body.push(quote! {
                    let #f_name = #out_prop_logic.sender().clone();
                });
                rust_struct_fields_init.push(quote! { #f_name });
                slint_data_fields_init.push(quote! { #f_name: Default::default() });
            }
        }
    }

    // 스칼라 IN 속성 처리 (Outer Model 변경 감지)
    let mut scalar_in_senders_collect = Vec::new();
    let mut scalar_diff_checks = Vec::new();
    
    for f in data_fields {
        if let Type::Array(_) = f.ty { continue; }
        if f.direction == Direction::In {
            let f_name = &f.name;
            let f_prop = format_ident!("{}_prop", f_name);
            let vec_name = format_ident!("{}_senders", f_name);
            scalar_in_senders_collect.push(quote! {
                #vec_name.push(#f_prop.sender().clone());
            });
            scalar_diff_checks.push(quote! {
                if new_data.#f_name != old_data.#f_name {
                    if let Some(sender) = #vec_name.get(idx) {
                        sender.send(new_data.#f_name.clone());
                    }
                }
            });
        }
    }
    
    let mut scalar_vectors_init = Vec::new();
    let mut scalar_vectors_clone = Vec::new();
    for f in data_fields {
         if let Type::Array(_) = f.ty { continue; }
         if f.direction == Direction::In {
             let f_name = &f.name;
             let vec_name = format_ident!("{}_senders", f_name);
             scalar_vectors_init.push(quote! { let mut #vec_name = Vec::with_capacity(#array_len); });
             scalar_vectors_clone.push(quote! { let #vec_name = #vec_name.clone(); });
         }
    }

    quote! {
        let mut rust_models = Vec::with_capacity(#array_len);
        let mut slint_data = Vec::with_capacity(#array_len);
        #(#scalar_vectors_init)*
        
        // 시그널 필드 설정 (배열)
        #(#signal_init_block)*

        for i in 0..#array_len {
            #(#loop_body)*
            
            #(#scalar_in_senders_collect)*

            rust_models.push(Self {
                #(#struct_init_ids),*
            });
            
            slint_data.push(#struct_data_name {
                #(#slint_data_fields_init),*
            });
        }

        let inner_model = std::rc::Rc::new(slint::VecModel::from(slint_data.clone()));
        
        let old_data_vec = std::rc::Rc::new(std::cell::RefCell::new(slint_data));
        let old_data_vec_clone = old_data_vec.clone();
        
        #(#scalar_vectors_clone)*
        
        let notify_model = frand_property::SlintNotifyModel::new(inner_model, move |idx, new_data| {
             let mut old_data_guard = old_data_vec_clone.borrow_mut();
             if idx < old_data_guard.len() {
                 let old_data = &mut old_data_guard[idx];
                 #(#scalar_diff_checks)*
                 *old_data = new_data;
             }
        });

        component.global::<#global_type_name>().set_data(
             slint::ModelRc::new(std::rc::Rc::new(notify_model))
        );

        rust_models
    }
}

fn generate_scalar_logic(
    struct_data_name: &syn::Ident,
    global_type_name: &syn::Ident,
    data_fields: &[&parser::SlintModelField],
    signal_fields: &[&parser::SlintModelField],
    input: &SlintModel
) -> TokenStream {
    let mut setups = Vec::new();
    let mut slint_init_fields = Vec::new();
    let struct_init_fields = generate_struct_init_fields(input);
    let mut parent_diff_checks = Vec::new();
    

    for f in signal_fields {
         let f_name = &f.name;
         let f_prop = format_ident!("{}_prop", f_name);
         
         if f.direction == Direction::Out {
              proc_macro_error::abort!(f_name, "`()` type cannot be used with `out` direction");
         } else {
             let on_ident = format_ident!("on_{}", f_name);
             setups.push(quote! {
                 let #f_prop = frand_property::Property::new(
                     weak.clone(),
                     Default::default(),
                     |_, _| {}
                 );
                 let sender = #f_prop.sender().clone();
                 component.global::<#global_type_name>().#on_ident(move |_| { sender.notify(); });
                 let #f_name = #f_prop.receiver().clone();
             });
         }
    }

    for f in data_fields {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let f_prop = format_ident!("{}_prop", f_name);

        if let Type::Array(arr) = f_ty {
             let len = &arr.len;
             let f_senders = format_ident!("{}_senders", f_name);


             if f.direction == Direction::In {
                 let resolved_elem_ty = resolve_type(&arr.elem);
                 let (setup, init) = generate_in_array_setup(f_name, len, &resolved_elem_ty);
                 setups.push(setup);
                 slint_init_fields.push(init);
             } else {
                 // 스칼라 Out 배열
                 let resolved_elem_ty = resolve_type(&arr.elem);
                 setups.push(quote! {
                    let mut #f_senders = Vec::with_capacity(#len);
                    for j in 0..#len {
                        let prop = frand_property::Property::<slint::Weak<C>, #resolved_elem_ty>::new(
                             weak.clone(),
                             Default::default(),
                             move |c, v| {
                                 c.upgrade_in_event_loop(move |c| {
                                     let global = c.global::<#global_type_name>();
                                     let model = global.get_data();
                                     if let Some(data) = model.row_data(0) {
                                         data.#f_name.set_row_data(j, v);
                                     }
                                 }).unwrap();
                             }
                        );
                        #f_senders.push(prop.sender().clone());
                    }
                    let #f_name = #f_senders;
                 });
                 slint_init_fields.push(quote! {
                     #f_name: slint::ModelRc::new(std::rc::Rc::new(slint::VecModel::from(vec![Default::default(); #len])))
                 });
             }
        } else {
            if f.direction == Direction::In {
                let f_sender = format_ident!("{}_sender", f_name);
                let resolved_ty = resolve_type(f_ty);
                setups.push(quote! {
                    let #f_prop = frand_property::Property::<slint::Weak<C>, #resolved_ty>::new(weak.clone(), Default::default(), |_, _| {});
                    let #f_name = #f_prop.receiver().clone();
                    let #f_sender = #f_prop.sender().clone();
                });
                parent_diff_checks.push(if is_special_string_type(f_ty) {
                    quote! {
                        if new_data.#f_name != old_data.#f_name {
                            if let Ok(val) = frand_property::arraystring::ArrayString::try_from_str(new_data.#f_name.as_str()) {
                                #f_sender.send(val);
                            }
                        }
                    }
                } else {
                    quote! {
                        if new_data.#f_name != old_data.#f_name {
                            #f_sender.send(new_data.#f_name.clone());
                        }
                    }
                });
                slint_init_fields.push(quote! { #f_name: Default::default() });
            } else {
                 let setter = if is_special_string_type(f_ty) {
                      quote! {
                          if let Some(mut data) = model.row_data(0) {
                              data.#f_name = v.to_string().into();
                              model.set_row_data(0, data);
                          }
                      }
                 } else {
                      quote! {
                          if let Some(mut data) = model.row_data(0) {
                              data.#f_name = v;
                              model.set_row_data(0, data);
                          }
                      }
                 };

                 let resolved_ty = resolve_type(f_ty);
                 let out_prop = generate_out_property(global_type_name, setter, resolved_ty);
                 setups.push(quote! {
                     let #f_name = #out_prop.sender().clone();
                 });
                 slint_init_fields.push(quote! { #f_name: Default::default() });
            }
        }
    }

    quote! {
        #(#setups)*
        
        let initial_data = #struct_data_name {
            #(#slint_init_fields),*
        };
        
        let old_data = std::rc::Rc::new(std::cell::RefCell::new(initial_data.clone()));
        let old_data_clone = old_data.clone();
        
        let inner_model = std::rc::Rc::new(slint::VecModel::from(vec![initial_data]));
        
        let notify_model = frand_property::SlintNotifyModel::new(inner_model, move |_row, new_data| {
            let mut old_data = old_data_clone.borrow_mut();
            
            #(#parent_diff_checks)*
            
            *old_data = new_data;
        });
        
        component.global::<#global_type_name>().set_data(
             slint::ModelRc::new(std::rc::Rc::new(notify_model))
        );
        
        Self {
            #(#struct_init_fields),*
        }
    }
}

fn generate_out_property(global_type_name: &syn::Ident, setter_block: TokenStream, resolved_ty: TokenStream) -> TokenStream {
    quote! {
        frand_property::Property::<slint::Weak<C>, #resolved_ty>::new(
             weak.clone(),
             Default::default(),
             move |c, v| {
                 c.upgrade_in_event_loop(move |c| {
                     let global = c.global::<#global_type_name>();
                     let model = global.get_data();
                     #setter_block
                 }).unwrap();
             }
         )
    }
}




fn generate_in_array_setup(
    f_name: &syn::Ident,
    len: &syn::Expr,
    resolved_elem_ty: &TokenStream
) -> (TokenStream, TokenStream) {
    let f_senders = format_ident!("{}_senders", f_name);
    let f_receivers = format_ident!("{}_receivers", f_name);

    let setup = quote! {
        let mut #f_senders = Vec::with_capacity(#len);
        let mut #f_receivers = Vec::with_capacity(#len);

        for _ in 0..#len {
            let prop = frand_property::Property::<slint::Weak<C>, #resolved_elem_ty>::new(weak.clone(), Default::default(), |_, _| {});
            #f_senders.push(prop.sender().clone());
            #f_receivers.push(prop.receiver().clone());
        }
        let #f_name = #f_receivers;

        let inner_vec_model = std::rc::Rc::new(slint::VecModel::from(vec![Default::default(); #len]));
        let senders_clone = #f_senders.clone();
        
        let notify_model = frand_property::SlintNotifyModel::new(inner_vec_model, move |idx, val| {
            if let Some(sender) = senders_clone.get(idx) {
                 sender.send(val);
            }
        });
    };

    let init = quote! {
        #f_name: slint::ModelRc::new(std::rc::Rc::new(notify_model))
    };

    (setup, init)
}
