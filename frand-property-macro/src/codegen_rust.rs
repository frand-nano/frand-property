use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;
use crate::parser::{Direction, SlintModel};
use proc_macro_error::abort;

pub fn generate(input: &SlintModel, doc_comment: TokenStream) -> TokenStream {
    let vis = &input.vis;
    let model_name = &input.model_name;
    let type_name = &input.type_name;
    let struct_data_name = type_name; // Use raw name as Struct Name
    let global_type_name = format_ident!("{}Global", type_name); // Append Global for Global Type

    let field_defs = generate_field_defs(input);
    let struct_init_fields = generate_struct_init_fields(input);
    
    let body_logic = generate_body_logic(input, &global_type_name, struct_data_name);

    quote! {
        #doc_comment
        #vis struct #model_name<C: slint::ComponentHandle> {
            #(#field_defs),*
        }

        impl<C: slint::ComponentHandle + 'static> #model_name<C> {
            pub fn new(component: &C) -> Self where for<'a> #global_type_name<'a>: slint::Global<'a, C> {
                use slint::Model as _;
                let weak = std::sync::Arc::new(component.as_weak());

                #body_logic

                Self {
                    #(#struct_init_fields),*
                }
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

    // 2. 데이터 필드 로직 준비
    //    - Diffing을 위해 Sender를 캡처하는 IN 속성 채널(Sender/Receiver) 인스턴스화
    //    - 기본 데이터 생성 준비
    let mut initial_field_assigns = Vec::new();
    let mut receiver_extracts = Vec::new();
    
    // 부모 NotifyModel Diffing 로직 (스칼라)
    let mut parent_diffing_checks = Vec::new();
    // 중첩 NotifyModel 설정 로직 (배열)
    let mut array_setups = Vec::new();
    // Out 속성 로직
    let mut out_prop_setups = Vec::new();

    for f in &data_fields {
        let f_name = &f.name;
        // 배열인지 확인
        if let Type::Array(arr) = &f.ty {
             // 배열 로직
             let len = &arr.len;
             let f_prop_vec = format_ident!("{}_props", f_name);
             let f_senders = format_ident!("{}_senders", f_name);
             let f_senders_clone = format_ident!("{}_senders_clone", f_name); // 수정: 식별자 정의
             
             if f.direction == Direction::In {
                 // 중첩 NotifyModel
                 let inner_array_ident = format_ident!("{}_inner_array", f_name);
                 let array_model_ident = format_ident!("{}_array_model", f_name);

                 array_setups.push(quote! {
                     let #f_prop_vec: Vec<_> = (0..#len).map(|_| {
                         frand_property::Property::new(weak.clone(), Default::default(), |_, _| {})
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
                 // Out 배열: Setter가 전역 모델->행->배열->행을 업데이트
                 out_prop_setups.push(quote! {
                     let #f_name: Vec<_> = (0..#len).map(|idx| {
                         frand_property::Property::new(
                             weak.clone(),
                             Default::default(),
                             move |c, v| {
                                 c.upgrade_in_event_loop(move |c| {
                                     let global = c.global::<#global_type_name>();
                                     let model = global.get_data();
                                     if let Some(data) = model.row_data(0) {
                                         // ModelRc인 data.#f_name에 접근
                                         data.#f_name.set_row_data(idx, v);
                                     }
                                 }).unwrap();
                             }
                         ).sender().clone()
                     }).collect();
                 });
                 
                 // VecModel로 구조체 초기화 (Out이므로 알림 불필요, 엄격한 Out)
                 initial_field_assigns.push(quote! {
                     #f_name: slint::ModelRc::new(std::rc::Rc::new(slint::VecModel::from(vec![Default::default(); #len])))
                 });
             }
             
        } else {
            // 스칼라 로직
            let f_prop = format_ident!("{}_prop", f_name);
            
            if f.direction == Direction::In {
                array_setups.push(quote! {
                    let #f_prop = frand_property::Property::new(
                        weak.clone(),
                        Default::default(),
                        |_, _| {}
                    );
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
                // 스칼라 Out
                out_prop_setups.push(quote! {
                     let #f_name = frand_property::Property::new(
                         weak.clone(),
                         Default::default(),
                         move |c, v| {
                             c.upgrade_in_event_loop(move |c| {
                                 let global = c.global::<#global_type_name>();
                                 let model = global.get_data();
                                 if let Some(mut data) = model.row_data(0) {
                                      data.#f_name = v;
                                      model.set_row_data(0, data);
                                 }
                             }).unwrap();
                         }
                     ).sender().clone();
                });
                initial_field_assigns.push(quote! { #f_name: Default::default() });
            }
        }
    }

    if !data_fields.is_empty() {
        setup_lines.push(quote! {
            // 1. 중첩 배열 및 IN 스칼라 채널 설정
            #(#array_setups)*
            
            // 2. 데이터 구조체 초기화
            let initial_data = #struct_data_name {
                #(#initial_field_assigns),*
            };
            
            // 3. 부모 NotifyModel 설정 (스칼라용)
            // RefCell에 초기 데이터를 캡처하여 상태 추적
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
            
            // 4. OUT 속성 설정
            #(#out_prop_setups)*
        });
    }
    
    setup_lines.push(quote! { #(#receiver_extracts)* });

    // 시그널 필드 (유닛)
    for f in &signal_fields {
         let f_name = &f.name;
         let f_prop = format_ident!("{}_prop", f_name);
         
         if f.direction == Direction::Out {
              // 원본 코드에서 abort로 정의됨
              abort!(f_name, "`()` type cannot be used with `out` direction");
         } else {
             // IN 유닛: Slint 콜백 -> Rust Receiver
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
