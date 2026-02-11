use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;
use frand_property_build::parser::{Direction, SlintModel, SlintModelField};
use crate::common::{resolve_type, is_special_string_type, is_unit_ty, generate_vec_init_tokens};

pub fn generate(input: &SlintModel, doc_comment: TokenStream) -> TokenStream {
    let vis = &input.vis;
    let model_name = &input.model_name;
    let type_name = &input.type_name;
    let global_type_name = type_name;
    let instances_ident = format_ident!("{}_INSTANCES", model_name.to_string().to_uppercase());

    let field_defs = generate_field_defs(input);
    
    let (array_len_tokens, ret_ty, downcast_ty, return_stmt, init_method) = if let Some(len) = &input.len {
        (
            quote! { #len },
            quote! { std::sync::Arc<[Self]> },
            quote! { std::sync::Arc<[Self]> },
            quote! { rust_models.into() },
            quote! {
                pub fn init_singleton(index: usize, init: impl FnOnce(&Self)) -> Self where C: frand_property::slint::SlintSingleton, for<'a> #global_type_name<'a>: slint::Global<'a, C> {
                    let models = Self::clone_singleton();
                    let model = models[index].clone();
                    init(&model);
                    model
                }
            }
        )
    } else {
        (
            quote! { 1 },
            quote! { std::sync::Arc<Self> },
            quote! { std::sync::Arc<Self> },
            quote! { std::sync::Arc::new(rust_models.pop().expect("Should have created at least one model")) },
            quote! {
                pub fn init_singleton(init: impl FnOnce(&Self)) -> Self where C: frand_property::slint::SlintSingleton, for<'a> #global_type_name<'a>: slint::Global<'a, C> {
                    let model_arc = Self::clone_singleton();
                    let model = (*model_arc).clone();
                    init(&model);
                    model
                }
            }
        )
    };
    
    // 배열 로직 (길이 = LEN 혹은 1)
    let body_logic_array = generate_logic_impl(array_len_tokens.clone(), global_type_name, input);

    let field_names_for_clone: Vec<_> = input.fields.iter().map(|f| {
        let name = &f.name;
        quote! { #name: self.#name.clone() }
    }).collect();

    let field_names_for_debug: Vec<_> = input.fields.iter().map(|f| {
        let name = &f.name;
        quote! { .field(stringify!(#name), &self.#name) }
    }).collect();

    // 통합 싱글톤 패턴
    quote! {
        static #instances_ident: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any + Send + Sync>>>> = std::sync::OnceLock::new();

        #doc_comment
        #vis struct #model_name<C: slint::ComponentHandle> {
            _handle: slint::Weak<C>,
            #(#field_defs),*
        }

        impl<C: slint::ComponentHandle> Clone for #model_name<C> {
            fn clone(&self) -> Self {
                Self {
                    _handle: self._handle.clone(),
                    #(#field_names_for_clone),*
                }
            }
        }

        impl<C: slint::ComponentHandle> std::fmt::Debug for #model_name<C> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#model_name))
                    #(#field_names_for_debug)*
                    .finish()
            }
        }

        impl<C: slint::ComponentHandle + 'static> #model_name<C> {
            pub fn clone_singleton() -> #ret_ty where C: frand_property::slint::SlintSingleton, for<'a> #global_type_name<'a>: slint::Global<'a, C> {
                    let map = #instances_ident.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
                    let mut map = map.lock().unwrap();
                    let type_id = std::any::TypeId::of::<C>();

                    if let Some(any_val) = map.get(&type_id) {
                        return any_val.downcast_ref::<#downcast_ty>().expect("Type mismatch in singleton store").clone();
                    }

                    use slint::Model as _;
                    let weak = C::clone_singleton();
                    let component = weak.upgrade().expect("Failed to upgrade singleton instance");

                    let mut rust_models = {
                        #body_logic_array
                    };

                    let result: #ret_ty = #return_stmt;
                    map.insert(type_id, Box::new(result.clone()));
                    result
            }

            #init_method
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

        let (is_array, elem_ty) = if let Type::Array(arr) = f_ty {
            (true, arr.elem.as_ref())
        } else if let Type::Slice(slice) = f_ty {
            (true, slice.elem.as_ref())
        } else {
            (false, f_ty)
        };

        if *direction == Direction::Model {
             // 중첩 모델: 단순 Rust 타입으로 포함
             let resolved_ty = resolve_type(elem_ty);
             if is_array {
                 quote! { #f_vis #f_name: std::sync::Arc<[#resolved_ty]> }
             } else {
                 quote! { #f_vis #f_name: std::sync::Arc<#resolved_ty> }
             }
        } else if is_array {
            let resolved_elem_ty = resolve_type(elem_ty);
            if *direction == Direction::Out {
                quote! { #f_vis #f_name: std::sync::Arc<[frand_property::Sender<#resolved_elem_ty, slint::Weak<C>>]> }
            } else {
                 quote! { #f_vis #f_name: std::sync::Arc<[frand_property::Receiver<#resolved_elem_ty>]> }
            }
        } else {
            if is_unit {
                if *direction == Direction::Out {
                     quote! { #f_vis #f_name: frand_property::Sender<(), slint::Weak<C>> }
                } else {
                     quote! { #f_vis #f_name: frand_property::Receiver<()> }
                }
            } else {
                let resolved_ty = resolve_type(f_ty);
                if *direction == Direction::Out {
                    quote! { #f_vis #f_name: frand_property::Sender<#resolved_ty, slint::Weak<C>> }
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

fn generate_logic_impl(
    array_len_tokens: TokenStream,
    global_type_name: &syn::Ident,
    input: &SlintModel
) -> TokenStream {
    let fields: Vec<_> = input.fields.iter().collect();
    let (data_fields, signal_fields): (Vec<_>, Vec<_>) = fields.into_iter()
        .partition(|f| !is_unit_ty(&f.ty));

    let struct_init_ids = generate_struct_init_fields(input);
    let mut loop_body = Vec::new();

    let mut slint_data_assignments = Vec::new();
    let mut rust_struct_fields_init = Vec::new();

    let mut signal_init_block = Vec::new();

    for f in &signal_fields {
        let (init, body, struct_id) = process_signal_field(f, &array_len_tokens, global_type_name);
        signal_init_block.push(init);
        loop_body.push(body);
        rust_struct_fields_init.push(struct_id);
    }

    for f in &data_fields {
        let (body, struct_id, assign) = process_data_field(f, global_type_name);
        loop_body.push(body);
        rust_struct_fields_init.push(struct_id);
        slint_data_assignments.push(assign);
    }

    // 스칼라 IN 속성 처리 (Outer Model 변경 감지)
    let mut scalar_in_senders_collect = Vec::new();
    let mut scalar_diff_checks = Vec::new();
    
    for f in &data_fields {
        if let Type::Array(_) = f.ty { continue; }
        if f.direction == Direction::In {
            let f_name = &f.name;
            let f_prop = format_ident!("{}_prop", f_name);
            let vec_name = format_ident!("{}_senders", f_name);
            scalar_in_senders_collect.push(quote! {
                #vec_name.push(#f_prop.sender().clone());
            });
            
            if crate::common::is_array_string_type(&f.ty) {
                scalar_diff_checks.push(quote! {
                    if new_data.#f_name != old_data.#f_name {
                        if let Some(sender) = #vec_name.get(idx) {
                             if let Ok(val) = ArrayString::try_from_str(new_data.#f_name.as_str()) {
                                 sender.send(val);
                             }
                        }
                    }
                });
            } else if crate::common::is_std_string_type(&f.ty) {
                 scalar_diff_checks.push(quote! {
                    if new_data.#f_name != old_data.#f_name {
                        if let Some(sender) = #vec_name.get(idx) {
                             sender.send(new_data.#f_name.to_string());
                        }
                    }
                });
            } else {
                scalar_diff_checks.push(quote! {
                    if new_data.#f_name != old_data.#f_name {
                        if let Some(sender) = #vec_name.get(idx) {
                            sender.send(new_data.#f_name.clone());
                        }
                    }
                });
            }
        }
    }
    
    let mut scalar_vectors_init = Vec::new();
    let mut scalar_vectors_clone = Vec::new();
    for f in &data_fields {
         if let Type::Array(_) = f.ty { continue; }
         if f.direction == Direction::In {
             let f_name = &f.name;
             let vec_name = format_ident!("{}_senders", f_name);
             scalar_vectors_init.push(quote! { let mut #vec_name = Vec::with_capacity(#array_len_tokens); });
             scalar_vectors_clone.push(quote! { let #vec_name = #vec_name.clone(); });
         }
    }

    quote! {
        let mut rust_models = Vec::with_capacity(#array_len_tokens);
        let mut slint_data = Vec::with_capacity(#array_len_tokens);
        #(#scalar_vectors_init)*
        
        // 시그널 필드 설정 (배열)
        #(#signal_init_block)*

        let template_data = component.global::<#global_type_name>().get_data().row_data(0).expect("Global data should have at least 1 element from Slint initialization");

        for i in 0..#array_len_tokens {
            #(#loop_body)*
            
            #(#scalar_in_senders_collect)*

            rust_models.push(Self {
                _handle: weak.clone(),
                #(#struct_init_ids),*
            });

            #[allow(unused_mut)]
            let mut slint_row_data = template_data.clone();
            #(#slint_data_assignments)*
            slint_data.push(slint_row_data);
        }

        let inner_model = std::rc::Rc::new(slint::VecModel::from(slint_data.clone()));
        
        let old_data_vec = std::rc::Rc::new(std::cell::RefCell::new(slint_data));
        let old_data_vec_clone = old_data_vec.clone();
        
        #(#scalar_vectors_clone)*
        
        let notify_model = frand_property::slint::SlintNotifyModel::new(inner_model, move |idx, new_data| {
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

fn generate_out_property(global_type_name: &syn::Ident, setter_block: TokenStream, resolved_ty: TokenStream) -> TokenStream {
    quote! {
        frand_property::Property::<#resolved_ty, slint::Weak<C>>::new(
             weak.clone(),
             <#resolved_ty as Default>::default(),
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

    let vec_init = generate_vec_init_tokens(len, resolved_elem_ty);

    let setup = quote! {
        let mut #f_senders: Vec<frand_property::Sender<#resolved_elem_ty, slint::Weak<C>>> = Vec::with_capacity(#len);
        let mut #f_receivers: Vec<frand_property::Receiver<#resolved_elem_ty>> = Vec::with_capacity(#len);

        for _ in 0..#len {
            let prop = frand_property::Property::<#resolved_elem_ty, slint::Weak<C>>::new(weak.clone(), <#resolved_elem_ty as Default>::default(), |_, _| {});
            #f_senders.push(prop.sender().clone());
            #f_receivers.push(prop.receiver().clone());
        }
        let #f_name: std::sync::Arc<[frand_property::Receiver<#resolved_elem_ty>]> = #f_receivers.into();

        let inner_vec_model: std::rc::Rc<slint::VecModel<#resolved_elem_ty>> = std::rc::Rc::new(slint::VecModel::from(
             #vec_init
        ));
        let senders_clone = #f_senders.clone();
        
        let notify_model = frand_property::slint::SlintNotifyModel::new(inner_vec_model, move |idx, val| {
            if let Some(sender) = senders_clone.get(idx) {
                 sender.send(val);
            }
        });
    };

    let init = quote! {
        slint_row_data.#f_name = slint::ModelRc::<#resolved_elem_ty>::new(std::rc::Rc::new(notify_model));
    };

    (setup, init)
}

fn process_signal_field(
    f: &SlintModelField,
    array_len_tokens: &TokenStream,
    global_type_name: &syn::Ident,
) -> (TokenStream, TokenStream, TokenStream) {
    let f_name = &f.name;

    if f.direction == Direction::Out {
         proc_macro_error::abort!(f_name, "`()` type cannot be used with `out` direction");
    }

    let f_senders = format_ident!("{}_senders", f_name);
    let f_receivers = format_ident!("{}_receivers", f_name);
    let on_ident = format_ident!("on_{}", f_name);

    let signal_init = quote! {
        let mut #f_senders = Vec::with_capacity(#array_len_tokens);
        let mut #f_receivers = Vec::with_capacity(#array_len_tokens);
        for _ in 0..#array_len_tokens {
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
    };

    let loop_body = quote! {
        let #f_name = #f_receivers[i].clone();
    };

    let struct_init = quote! { #f_name };

    (signal_init, loop_body, struct_init)
}

fn process_data_field(
    f: &SlintModelField,
    global_type_name: &syn::Ident,
) -> (TokenStream, TokenStream, TokenStream) {
    let f_name = &f.name;
    let f_ty = &f.ty;
    let f_prop = format_ident!("{}_prop", f_name);

    let (is_array, elem_ty, array_len) = if let Type::Array(arr) = f_ty {
         (true, arr.elem.as_ref(), Some(&arr.len))
    } else if let Type::Slice(slice) = f_ty {
         (true, slice.elem.as_ref(), None)
    } else {
         (false, f_ty, None)
    };

    let resolved_elem_ty = resolve_type(elem_ty);

    if is_array {
        if f.direction == Direction::In {
            // 배열 IN: 각 요소에 대해 Property 생성
            let len = array_len.expect("Array length required for 'in' property fields");
            let (setup, init) = generate_in_array_setup(f_name, len, &resolved_elem_ty);
            (setup, quote! { #f_name }, init)
        } else if f.direction == Direction::Model {
             // 모델은 반드시 [] (Type::Slice) 여야 함. Type::Array(길이 명시)는 허용하지 않음.
             if let Some(_len) = array_len {
                 proc_macro_error::abort!(f_name, "Model fields must use implicit length syntax `[]`. Explicit length `[N]` is not allowed for models.");
             }
             
             let loop_body = quote! {
                 let #f_name = #resolved_elem_ty::clone_singleton();
             };
             (loop_body, quote! { #f_name }, quote!{})
        } else {
             // Out: 반드시 [N] (Type::Array) 여야 함. Type::Slice(길이 생략)는 허용하지 않음.
             // array_len이 None이면 Type::Slice라는 의미
             if array_len.is_none() {
                 proc_macro_error::abort!(f_name, "Value fields (in/out) must use explicit length syntax `[N]`. Implicit length `[]` is not allowed.");
             }

             let len = array_len.expect("Array length required for 'out' property fields");
             let f_senders = format_ident!("{}_senders", f_name);
             let vec_init = generate_vec_init_tokens(len, &resolved_elem_ty);
             let loop_body = quote! {
                 let mut #f_senders = Vec::with_capacity(#len);
                 for j in 0..#len {
                    let prop = frand_property::Property::<#resolved_elem_ty, slint::Weak<C>>::new(
                         weak.clone(),
                         <#resolved_elem_ty as Default>::default(),
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
                let #f_name: std::sync::Arc<[frand_property::Sender<#resolved_elem_ty, slint::Weak<C>>]> = #f_senders.into();
            };
            let struct_init = quote! { #f_name };
            let slint_assignment = quote! {
                slint_row_data.#f_name = slint::ModelRc::<#resolved_elem_ty>::new(std::rc::Rc::new(slint::VecModel::from(
                    #vec_init
                )));
            };
            (loop_body, struct_init, slint_assignment)
        }
    } else {
        // 스칼라 로직
        if f.direction == Direction::In {
            let loop_body = quote! {
                let #f_prop = frand_property::Property::new(weak.clone(), Default::default(), |_, _| {});
                let #f_name = #f_prop.receiver().clone();
            };
            (loop_body, quote! { #f_name }, quote!{})
        } else if f.direction == Direction::Model {
             let loop_body = quote! {
                 let #f_name = #resolved_elem_ty::clone_singleton();
             };
             (loop_body, quote! { #f_name }, quote!{})
        } else {
            // Out Scalar
            let setter = if is_special_string_type(f_ty) {
                 quote! {
                     if let Some(mut data) = model.row_data(i) {
                          data.#f_name = v.to_string().into();
                          model.set_row_data(i, data);
                     }
                 }
            } else {
                 quote! {
                     if let Some(mut data) = model.row_data(i) {
                          data.#f_name = v.into();
                          model.set_row_data(i, data);
                     }
                 }
            };

            let out_prop_logic = generate_out_property(global_type_name, setter, resolved_elem_ty);
            let loop_body = quote! {
                let #f_name = #out_prop_logic.sender().clone();
            };
            (loop_body, quote! { #f_name }, quote!{})
        }
    }
}
