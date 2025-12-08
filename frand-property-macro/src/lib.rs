use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, Ident, Token, Type, Visibility,
};

#[derive(PartialEq)]
enum Direction {
    In,
    Out,
    InOut,
}

struct SlintModel {
    vis: Visibility,
    model_name: Ident,
    _colon_token: Token![:],
    type_name: Ident,
    _brace_token: token::Brace,
    fields: Punctuated<SlintModelField, Token![,]>,
}

struct SlintModelField {
    vis: Visibility,
    direction: Direction,
    name: Ident,
    _colon_token: Token![:],
    ty: Type,
}

impl Parse for SlintModel {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(SlintModel {
            vis: input.parse()?,
            model_name: input.parse()?,
            _colon_token: input.parse()?,
            type_name: input.parse()?,
            _brace_token: syn::braced!(content in input),
            fields: content.parse_terminated(SlintModelField::parse, Token![,])?,
        })
    }
}

impl Parse for SlintModelField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        
        let direction = if input.peek(Token![in]) {
            input.parse::<Token![in]>()?;
            if input.peek(Token![-]) {
                input.parse::<Token![-]>()?;
                let out_kw: Ident = input.parse()?;
                if out_kw != "out" {
                    return Err(syn::Error::new(out_kw.span(), "expected `out` after `in-`"));
                }
                Direction::InOut
            } else {
                Direction::In
            }
        } else if input.peek(Ident) {
             let fork = input.fork();
             let ident: Ident = fork.parse()?;
             if ident == "out" {
                 input.parse::<Ident>()?;
                 Direction::Out
             } else {
                 return Err(syn::Error::new(ident.span(), "expected `in`, `out` or `in-out`"));
             }
        } else {
             return Err(input.error("expected `in`, `out` or `in-out`"));
        };

        Ok(SlintModelField {
            vis,
            direction,
            name: input.parse()?,
            _colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

#[proc_macro]
#[proc_macro_error]
pub fn slint_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SlintModel);

    let vis = input.vis;
    let model_name = input.model_name;
    let type_name = input.type_name;

    let field_defs = input.fields.iter().map(|f| {
        let f_vis = &f.vis;
        let f_name = &f.name;
        let f_ty = &f.ty;

        // 타입이 ()인지 확인
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
    });

    let field_inits = input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let direction = &f.direction;

        let is_unit = if let Type::Tuple(t) = f_ty {
            t.elems.is_empty()
        } else {
            false
        };

        // () 타입인데 Out 또는 InOut이면 에러
        if is_unit && (*direction == Direction::Out || *direction == Direction::InOut) {
            abort!(f_name, "`()` type cannot be used with `out` or `in-out` direction");
        }

        let setter = if *direction == Direction::Out || *direction == Direction::InOut {
             if is_unit {
                 // 위에서 걸러지지만 안전을 위해
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
            // In 방향이면 setter는 아무것도 안 함
            quote! { |_, _| {} }
        };

        quote! {
            let #f_name = Property::new(
                weak.clone(),
                Default::default(),
                #setter,
            );
        }
    });

    let sender_defs = input.fields.iter().map(|f| {
        let f_name = &f.name;
        let direction = &f.direction;
        let sender_name = format_ident!("{}_sender", f_name);
        
        // Out 방향이면 sender를 생성하지 않음 (Warning 해결)
        if *direction == Direction::Out {
            quote! {}
        } else {
            quote! {
                let #sender_name = #f_name.sender().clone();
            }
        }
    });

    let bindings = input.fields.iter().map(|f| {
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
    });

    let struct_init_fields = input.fields.iter().map(|f| {
        let f_name = &f.name;
        quote! {
            #f_name
        }
    });

    let expanded = quote! {
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
    };

    TokenStream::from(expanded)
}