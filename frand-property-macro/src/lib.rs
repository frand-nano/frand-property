use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, Ident, Token, Type, Visibility,
};

struct ModelField {
    vis: Visibility,
    name: Ident,
    _colon_token: Token![:],
    ty: Type,
}

impl Parse for ModelField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ModelField {
            vis: input.parse()?,
            name: input.parse()?,
            _colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

struct SlintModelInput {
    vis: Visibility,
    model_name: Ident,
    _colon_token: Token![:],
    type_name: Ident,
    _lt_token: Token![<],
    component_type: Type,
    _gt_token: Token![>],
    _brace_token: token::Brace,
    fields: Punctuated<ModelField, Token![,]>,
}

impl Parse for SlintModelInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(SlintModelInput {
            vis: input.parse()?,
            model_name: input.parse()?,
            _colon_token: input.parse()?,
            type_name: input.parse()?,
            _lt_token: input.parse()?,
            component_type: input.parse()?,
            _gt_token: input.parse()?,
            _brace_token: syn::braced!(content in input),
            fields: content.parse_terminated(ModelField::parse, Token![,])?,
        })
    }
}

#[proc_macro]
pub fn slint_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SlintModelInput);

    let vis = input.vis;
    let model_name = input.model_name;
    let type_name = input.type_name;
    let component_type = input.component_type;

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
                #f_vis #f_name: Property<slint::Weak<#component_type>, ()>
            }
        } else {
            quote! {
                #f_vis #f_name: Property<slint::Weak<#component_type>, #f_ty>
            }
        }
    });

    let field_inits = input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        
         let is_unit = if let Type::Tuple(t) = f_ty {
            t.elems.is_empty()
        } else {
            false
        };

        if is_unit {
            quote! {
                let #f_name = Property::new(
                    weak.clone(),
                    (),
                    |_, _| {},
                );
            }
        } else {
            let set_ident = format_ident!("set_{}", f_name);
            quote! {
                let #f_name = Property::new(
                    weak.clone(),
                    Default::default(),
                    |c, v| {
                        c.upgrade_in_event_loop(move |c| {
                            c.global::<#type_name>().#set_ident(v)
                        }).unwrap() // TODO: 에러 처리
                    },
                );
            }
        }
    });

    let sender_defs = input.fields.iter().map(|f| {
        let f_name = &f.name;
        let sender_name = format_ident!("{}_sender", f_name);
        quote! {
            let #sender_name = #f_name.sender().clone();
        }
    });

    let bindings = input.fields.iter().map(|f| {
        let f_name = &f.name;
        let f_ty = &f.ty;
        let sender_name = format_ident!("{}_sender", f_name);

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
    });

    let struct_init_fields = input.fields.iter().map(|f| {
        let f_name = &f.name;
        quote! {
            #f_name
        }
    });

    let expanded = quote! {
        #vis struct #model_name {
            #(#field_defs),*
        }

        impl #model_name {
            pub fn new(
                component: &#component_type,
            ) -> Self
            where #component_type: slint::ComponentHandle {
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
