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
                use slint::Model as _;
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
        let direction = &f.direction;

        if let Type::Array(arr) = f_ty {
            let elem_ty = &arr.elem;
            if *direction == Direction::Out {
                quote! {
                    #f_vis #f_name: Vec<Sender<slint::Weak<C>, #elem_ty>>
                }
            } else {
                 quote! {
                    #f_vis #f_name: Vec<Receiver<#elem_ty>>
                }
            }
        } else {
             let is_unit = if let Type::Tuple(t) = f_ty {
                t.elems.is_empty()
            } else {
                false
            };

            if is_unit {
                if *direction == Direction::Out {
                     quote! {
                        #f_vis #f_name: Sender<slint::Weak<C>, ()>
                    }
                } else {
                     quote! {
                        #f_vis #f_name: Receiver<()>
                    }
                }
            } else {
                if *direction == Direction::Out {
                    quote! {
                        #f_vis #f_name: Sender<slint::Weak<C>, #f_ty>
                    }
                } else {
                    quote! {
                        #f_vis #f_name: Receiver<#f_ty>
                    }
                }
            }
        }
    }).collect()
}

fn generate_field_inits(input: &SlintModel, type_name: &syn::Ident) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_prop_ident = format_ident!("{}_prop", f_name);
        let f_ty = &f.ty;
        let direction = &f.direction;

        if let Type::Array(arr) = f_ty {
            let len = &arr.len;
            let set_global_ident = format_ident!("set_{}", f_name);

            if *direction == Direction::Out {
                let get_ident = format_ident!("get_{}", f_name);
                let setter = quote! {
                    move |c, v| {
                        c.upgrade_in_event_loop(move |c| {
                            c.global::<#type_name>().#get_ident().set_row_data(_index, v);
                        }).unwrap()
                    }
                };

                quote! {
                    let #f_name = (0..#len).map(|_index| {
                        Property::new(
                            weak.clone(),
                            Default::default(),
                            #setter
                        ).sender().clone()
                    }).collect::<Vec<_>>();

                    let senders = #f_name.clone();
                    // Property::new로 생성된 sender들은 각각의 set_row_data를 호출하도록 설정됨.
                    let inner = std::rc::Rc::new(slint::VecModel::from(vec![Default::default(); #len]));
                    // NotifyModel의 on_change는 Slint -> Rust 알림용인데 Out이므로 필요 없음.
                    let model = frand_property::NotifyModel::new(inner, |_, _| {});

                    component.global::<#type_name>()
                        .#set_global_ident(
                            slint::ModelRc::new(std::rc::Rc::new(model))
                        );

                }
            } else {
                // In: Slint -> Rust.
                // Slint에서 변경되면 Rust에 알림.
                // Property::new로 생성하여 Receiver는 보관, Sender는 NotifyModel에 전달.
                quote! {
                    let #f_prop_ident = (0..#len).map(|_index| {
                        Property::new(
                            weak.clone(),
                            Default::default(),
                            move |_, _| {} // In 방향이므로 Rust -> Slint 세터 필요 없음 (NotifyModel이 처리할수도 있으나 보통 역방향은 X)
                        )
                    }).collect::<Vec<_>>();
                    
                    let #f_name = #f_prop_ident.iter().map(|p| p.receiver().clone()).collect::<Vec<_>>();
                    let senders = #f_prop_ident.iter().map(|p| p.sender().clone()).collect::<Vec<_>>();

                    let inner = std::rc::Rc::new(slint::VecModel::from(vec![Default::default(); #len]));
                    let model = frand_property::NotifyModel::new(inner, move |i, v| {
                        if let Some(sender) = senders.get(i) {
                            sender.send(v);
                        }
                    });

                    component.global::<#type_name>()
                        .#set_global_ident(
                            slint::ModelRc::new(std::rc::Rc::new(model))
                        );
                }
            }
        } else {
            let is_unit = if let Type::Tuple(t) = f_ty {
                t.elems.is_empty()
            } else {
                false
            };

            if is_unit && *direction == Direction::Out {
                abort!(f_name, "`()` type cannot be used with `out` direction");
            }

            if *direction == Direction::Out {
                 let set_ident = format_ident!("set_{}", f_name);
                 let setter = if is_unit {
                      quote! { |_, _| {} }
                 } else {
                     quote! {
                         |c, v| {
                             c.upgrade_in_event_loop(move |c| {
                                 c.global::<#type_name>().#set_ident(v)
                             }).unwrap() // TODO: 에러 처리
                         }
                     }
                 };
                  
                quote! {
                    let #f_name = Property::new(
                        weak.clone(),
                        Default::default(),
                        #setter,
                    ).sender().clone();
                }
            } else {
                // In Property
                quote! {
                    let #f_prop_ident = Property::new(
                        weak.clone(),
                        Default::default(),
                        |_, _| {},
                    );
                    let #f_name = #f_prop_ident.receiver().clone();
                }
            }
        }
    }).collect()
}

fn generate_sender_defs(input: &SlintModel) -> Vec<TokenStream> {
    input.fields.iter().map(|f| {
        let f_name = &f.name;
        let direction = &f.direction;
        let sender_name = format_ident!("{}_sender", f_name);
        let f_prop_ident = format_ident!("{}_prop", f_name);
        let f_ty = &f.ty;
        
        if let Type::Array(_) = f_ty {
            quote! {}
        } else {
            if *direction == Direction::Out {
                // Out 필드는 Sender가 이미 #f_name 에 있음. 
                // bindings에서 필요하다면 #f_name을 씀.
                // Property 구조가 아니므로 sender() 메서드 호출 불가.
                // 하지만 Out 이므로 Slint 이벤트 바인딩(In 동작)이 필요 없음.
                quote! {}
            } else {
                // In 필드: Property로 생성했으므로 sender 추출 가능
                quote! {
                    let #sender_name = #f_prop_ident.sender().clone();
                }
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

        if *direction == Direction::In {
             if let Type::Array(_) = f_ty {
                 quote! {}
             } else {
                let is_unit = if let Type::Tuple(t) = f_ty {
                    t.elems.is_empty()
                } else {
                    false
                };

                if is_unit {
                    let on_ident = format_ident!("on_{}", f_name);

                    quote! {
                        component.global::<#type_name>().#on_ident(move || #sender_name.notify());
                    }
                } else {
                    let on_changed_ident = format_ident!("on_changed_{}", f_name);

                    quote! {
                        component.global::<#type_name>().#on_changed_ident(move |v| #sender_name.send(v));
                    }
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
