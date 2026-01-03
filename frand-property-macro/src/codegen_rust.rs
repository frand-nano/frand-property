use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;
use crate::parser::{Direction, SlintModel};
use proc_macro_error::abort;

pub fn generate(input: &SlintModel, doc_comment: TokenStream) -> TokenStream {
    let vis = &input.vis;
    let model_name = &input.model_name;
    let type_name = &input.type_name;
    let struct_data_name = type_name; // 구조체 이름으로 원본 이름 사용
    let global_type_name = format_ident!("{}Global", type_name); // Global 타입 이름에 'Global' 접미사 추가

    // 1. 데이터 필드 생성
    let field_defs = generate_field_defs(input);
    // 2. 구조체 초기화 필드 생성
    let struct_init_fields = generate_struct_init_fields(input);
    
    // 3. 본문 로직 생성
    let body_logic = generate_body_logic(input, &global_type_name, struct_data_name);

    let (return_type, return_expr) = if input.array_len.is_some() {
        (
            quote! { Vec<Self> },
            quote! { #body_logic } // body_logic이 직접 Vec을 반환
        )
    } else {
        (
            quote! { Self },
            quote! { 
                #body_logic 
                Self {
                    #(#struct_init_fields),*
                }
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

fn generate_body_logic(input: &SlintModel, global_type_name: &syn::Ident, struct_data_name: &syn::Ident) -> TokenStream {
    let mut setup_lines = Vec::new();

    // 1. 데이터 필드와 시그널 필드 분리
    let fields: Vec<_> = input.fields.iter().collect();
    let (data_fields, signal_fields): (Vec<_>, Vec<_>) = fields.into_iter()
        .partition(|f| !is_unit_ty(&f.ty));

    // 모델 배열 여부에 따라 로직 분기
    let main_logic = if let Some(array_len) = &input.array_len {
        generate_array_logic(array_len, struct_data_name, global_type_name, &data_fields, input)
    } else {
        generate_scalar_logic(struct_data_name, global_type_name, &data_fields)
    };
    
    setup_lines.push(main_logic);

    // 시그널 필드 (유닛)
    for f in &signal_fields {
         let f_name = &f.name;
         let f_prop = format_ident!("{}_prop", f_name);
         
         if f.direction == Direction::Out {
              abort!(f_name, "`()` type cannot be used with `out` direction");
         } else {
             let on_ident = format_ident!("on_{}", f_name);
             setup_lines.push(quote! {
                 let #f_prop = frand_property::Property::new(
                     weak.clone(),
                     Default::default(),
                     |_, _| {}
                 );
                 let sender = #f_prop.sender().clone();
                 component.global::<#global_type_name>().#on_ident(move || { sender.notify(); });
                 let #f_name = #f_prop.receiver().clone();
             });
         }
    }

    quote! { #(#setup_lines)* }
}

fn generate_field_defs(input: &SlintModel) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_vis = &f.vis;
        let f_name = &f.name;
        let f_ty = &f.ty;
        let direction = &f.direction;

        if let Type::Array(arr) = f_ty {
            let elem_ty = &arr.elem;
            if *direction == Direction::Out {
                quote! { #f_vis #f_name: Vec<frand_property::Sender<slint::Weak<C>, #elem_ty>> }
            } else {
                 quote! { #f_vis #f_name: Vec<frand_property::Receiver<#elem_ty>> }
            }
        } else {
            let is_unit = is_unit_ty(f_ty);

            if is_unit {
                if *direction == Direction::Out {
                     quote! { #f_vis #f_name: frand_property::Sender<slint::Weak<C>, ()> }
                } else {
                     quote! { #f_vis #f_name: frand_property::Receiver<()> }
                }
            } else {
                if *direction == Direction::Out {
                    quote! { #f_vis #f_name: frand_property::Sender<slint::Weak<C>, #f_ty> }
                } else {
                    quote! { #f_vis #f_name: frand_property::Receiver<#f_ty> }
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

fn is_unit_ty(ty: &Type) -> bool {
    if let Type::Tuple(t) = ty {
        t.elems.is_empty()
    } else {
        false
    }
}

fn generate_array_logic(
    array_len: &syn::Expr,
    struct_data_name: &syn::Ident,
    global_type_name: &syn::Ident,
    data_fields: &[&crate::parser::SlintModelField],
    input: &SlintModel
) -> TokenStream {
    let mut setup_lines = Vec::new();
    
    // 1. 배열 단위 초기 데이터 준비
    setup_lines.push(quote! {
        let initial_data = #struct_data_name::default();
        let model_vec = vec![initial_data; #array_len];
        let inner_model = std::rc::Rc::new(slint::VecModel::from(model_vec));
    });

    // 2. NotifyModel 설정
    setup_lines.push(quote! {
         let old_data_vec = std::rc::Rc::new(std::cell::RefCell::new(vec![#struct_data_name::default(); #array_len]));
         let old_data_vec_clone = old_data_vec.clone();
         
         let notify_model = frand_property::NotifyModel::new(inner_model, move |idx, new_data| {
             let mut old_data_guard = old_data_vec_clone.borrow_mut();
             if let Some(old_data) = old_data_guard.get_mut(idx) {
                 *old_data = new_data;
             }
         });

         component.global::<#global_type_name>().set_data(
              slint::ModelRc::new(std::rc::Rc::new(notify_model))
         );
    });
    
    let mut field_diff_checks = Vec::new();

    for f in data_fields {
        let f_name = &f.name;
         if f.direction == Direction::In {
             let f_senders_vec_name = format_ident!("{}_senders_vec", f_name);
             field_diff_checks.push(quote! {
                 if new_data.#f_name != old_data.#f_name {
                     if let Some(sender) = #f_senders_vec_name.get(idx) {
                         sender.send(new_data.#f_name.clone());
                     }
                 }
             });
         }
    }
    
    let mut sender_collectors_init = Vec::new();
    let mut sender_move_to_callback = Vec::new();
    
    for f in data_fields {
         let f_name = &f.name;
         let f_senders_vec_name = format_ident!("{}_senders_vec", f_name);
         
         if f.direction == Direction::In {
             sender_collectors_init.push(quote! { let mut #f_senders_vec_name = Vec::with_capacity(#array_len); });
             sender_move_to_callback.push(quote! { let #f_senders_vec_name = #f_senders_vec_name.clone(); });
         }
    }

    let mut per_row_field_inits = Vec::new();
    
    for f in data_fields {
         let f_name = &f.name;
         let f_prop = format_ident!("{}_prop", f_name);
         
         if f.direction == Direction::In {
             let f_senders_vec_name = format_ident!("{}_senders_vec", f_name);
             let in_prop_logic = generate_in_property();
             
             per_row_field_inits.push(quote! {
                 let #f_prop = #in_prop_logic;
                 #f_senders_vec_name.push(#f_prop.sender().clone());
                 let #f_name = #f_prop.receiver().clone();
             });
         } else {
             let setter = quote! {
                  if let Some(mut data) = model.row_data(i) {
                       data.#f_name = v;
                       model.set_row_data(i, data);
                  }
             };
             let out_prop_logic = generate_out_property(global_type_name, setter);
             
             per_row_field_inits.push(quote! {
                 let #f_name = #out_prop_logic.sender().clone();
             });
         }
    }

    let struct_init_block = generate_struct_init_fields(input);
    
    // 루프 및 Sender 로직을 포함하여 설정 정의 재작성
    setup_lines = Vec::new();
    
    setup_lines.push(quote! {
         let mut result_structs = Vec::with_capacity(#array_len);
         
         #(#sender_collectors_init)*
         
         for i in 0..#array_len {
             #(#per_row_field_inits)*
             
             result_structs.push(Self {
                 #(#struct_init_block),*
             });
         }
    });
    
    setup_lines.push(quote! {
         let initial_data = #struct_data_name::default();
         let inner_model = std::rc::Rc::new(slint::VecModel::from(vec![initial_data; #array_len]));
         
         let old_data_vec = std::rc::Rc::new(std::cell::RefCell::new(vec![#struct_data_name::default(); #array_len]));
         let old_data_vec_clone = old_data_vec.clone();
         
         #(#sender_move_to_callback)*
         
         let notify_model = frand_property::NotifyModel::new(inner_model, move |idx, new_data| {
             let mut old_data_guard = old_data_vec_clone.borrow_mut();
             if idx < old_data_guard.len() {
                let old_data = &mut old_data_guard[idx];
                #(#field_diff_checks)*
                *old_data = new_data;
             }
         });
         
         component.global::<#global_type_name>().set_data(
              slint::ModelRc::new(std::rc::Rc::new(notify_model))
         );
         
         result_structs
    });
    
    quote! { #(#setup_lines)* }
}

fn generate_scalar_logic(
    struct_data_name: &syn::Ident,
    global_type_name: &syn::Ident,
    data_fields: &[&crate::parser::SlintModelField]
) -> TokenStream {
    let mut initial_field_assigns = Vec::new();
    let mut receiver_extracts = Vec::new();
    let mut parent_diffing_checks = Vec::new();
    let mut array_setups = Vec::new();
    let mut out_prop_setups = Vec::new();

    for f in data_fields {
        let f_name = &f.name;
        if let Type::Array(arr) = &f.ty {
             let len = &arr.len;
             let f_prop_vec = format_ident!("{}_props", f_name);
             let f_senders = format_ident!("{}_senders", f_name);
             let f_senders_clone = format_ident!("{}_senders_clone", f_name);
             
             if f.direction == Direction::In {
                 let inner_array_ident = format_ident!("{}_inner_array", f_name);
                 let array_model_ident = format_ident!("{}_array_model", f_name);
                 let in_prop_logic = generate_in_property();

                 array_setups.push(quote! {
                     let #f_prop_vec: Vec<_> = (0..#len).map(|_| {
                         #in_prop_logic
                     }).collect();
                     let #f_senders: Vec<_> = #f_prop_vec.iter().map(|p| p.sender().clone()).collect();
                     
                     let #inner_array_ident = std::rc::Rc::new(slint::VecModel::from(vec![Default::default(); #len]));
                     let #f_senders_clone = #f_senders.clone();
                     let #array_model_ident = frand_property::NotifyModel::new(#inner_array_ident, move |idx, val| {
                         if let Some(sender) = #f_senders_clone.get(idx) {
                             sender.send(val);
                         }
                     });
                 });
                 initial_field_assigns.push(quote! { 
                     #f_name: slint::ModelRc::new(std::rc::Rc::new(#array_model_ident)) 
                 });
                 
                 receiver_extracts.push(quote! {
                     let #f_name: Vec<_> = #f_prop_vec.iter().map(|p| p.receiver().clone()).collect();
                 });
                 
             } else {
                 let setter = quote! {
                     if let Some(data) = model.row_data(0) {
                         data.#f_name.set_row_data(idx, v);
                     }
                 };
                 let out_prop_logic = generate_out_property(global_type_name, setter);
                 
                 out_prop_setups.push(quote! {
                     let #f_name: Vec<_> = (0..#len).map(|idx| {
                         #out_prop_logic.sender().clone()
                     }).collect();
                 });
                 
                 initial_field_assigns.push(quote! {
                     #f_name: slint::ModelRc::new(std::rc::Rc::new(slint::VecModel::from(vec![Default::default(); #len])))
                 });
             }
             
        } else {
            let f_prop = format_ident!("{}_prop", f_name);
            
            if f.direction == Direction::In {
                let in_prop_logic = generate_in_property();
                
                array_setups.push(quote! {
                    let #f_prop = #in_prop_logic;
                    let #f_name = #f_prop.sender().clone();
                });
                
                initial_field_assigns.push(quote! { #f_name: Default::default() });
                
                parent_diffing_checks.push(quote! {
                    if new_data.#f_name != old_data.#f_name {
                        #f_name.send(new_data.#f_name.clone());
                    }
                });
                
                receiver_extracts.push(quote! {
                    let #f_name = #f_prop.receiver().clone();
                });
            } else {
                let setter = quote! {
                     if let Some(mut data) = model.row_data(0) {
                          data.#f_name = v;
                          model.set_row_data(0, data);
                     }
                };
                let out_prop_logic = generate_out_property(global_type_name, setter);
                
                out_prop_setups.push(quote! {
                     let #f_name = #out_prop_logic.sender().clone();
                });
                initial_field_assigns.push(quote! { #f_name: Default::default() });
            }
        }
    }

    let mut setup_lines = Vec::new();
    if !data_fields.is_empty() {
        setup_lines.push(quote! {
            #(#array_setups)*
            
            let initial_data = #struct_data_name {
                #(#initial_field_assigns),*
            };
            
            let old_data = std::rc::Rc::new(std::cell::RefCell::new(initial_data.clone()));
            let old_data_clone = old_data.clone();
            
            let inner_model = std::rc::Rc::new(slint::VecModel::from(vec![initial_data]));
            let notify_model = frand_property::NotifyModel::new(inner_model, move |_row, new_data| {
                let mut old_data = old_data_clone.borrow_mut();
                
                #(#parent_diffing_checks)*
                
                *old_data = new_data;
            });
            
            component.global::<#global_type_name>().set_data(
                 slint::ModelRc::new(std::rc::Rc::new(notify_model))
            );
            
            #(#out_prop_setups)*
        });
    }
    
    setup_lines.push(quote! { #(#receiver_extracts)* });
    quote! { #(#setup_lines)* }
}

fn generate_in_property() -> TokenStream {
    quote! {
        frand_property::Property::new(weak.clone(), Default::default(), |_, _| {})
    }
}

fn generate_out_property(global_type_name: &syn::Ident, setter_block: TokenStream) -> TokenStream {
    quote! {
        frand_property::Property::new(
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
