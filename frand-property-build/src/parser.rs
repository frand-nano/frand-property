use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Ident, Token, Type, Visibility,
};
use quote::quote;

pub fn parse_len_expr(input: ParseStream) -> syn::Result<proc_macro2::TokenStream> {
    if input.peek(syn::Lit) {
        let lit: syn::Lit = input.parse()?;
        Ok(quote! { #lit })
    } else if input.peek(Token![*]) {
        input.parse::<Token![*]>()?;
        let ident: Ident = input.parse()?;
        Ok(quote! { *#ident })
    } else if input.peek(Ident) {
        let ident: Ident = input.parse()?;
        Ok(quote! { #ident })
    } else {
        Err(input.error("Expected literal, identifier, or *identifier inside brackets"))
    }
}

#[derive(PartialEq, Clone)]
pub enum Direction {
    In,
    Out,
    Model,
}

// Model 구조체 정의
pub struct Model {
    pub vis: Visibility,
    pub model_name: Ident,
    pub len: Option<proc_macro2::TokenStream>,

    pub _brace_token: token::Brace,
    pub fields: Punctuated<ModelField, Token![,]>,
}

// Model 필드 정의
pub struct ModelField {
    pub vis: Visibility,
    pub is_model: bool,
    pub name: Ident,
    pub _colon_token: Token![:],
    pub ty: Type,
}

impl Parse for Model {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        
        let model_name: Ident = input.parse()?;
        
        let len = if input.peek(token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            Some(parse_len_expr(&content)?)
        } else {
            None
        };

        let content;
        Ok(Model {
            vis,
            model_name,
            len,

            _brace_token: syn::braced!(content in input),
            fields: content.parse_terminated(ModelField::parse, Token![,])?,
        })
    }
}

impl Parse for ModelField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        
        // 'model' 키워드 확인
        let is_model = if input.peek(Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            if ident == "model" {
                input.parse::<Ident>()?;
                true
            } else {
                false
            }
        } else {
            false
        };

        let name = input.parse()?;
        let _colon_token = input.parse()?;
        let mut ty: Type = input.parse()?;
        
        if input.peek(token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            if content.is_empty() {
                ty = syn::parse_quote!([#ty]);
            } else {
                let len_tokens = parse_len_expr(&content)?;
                ty = syn::parse_quote!([#ty; #len_tokens]);
            }
        }

        Ok(ModelField {
            vis,
            is_model,
            name,
            _colon_token,
            ty,
        })
    }
}

mod kw {
    syn::custom_keyword!(export);
    syn::custom_keyword!(to);
}

pub struct SlintModel {
    pub export_path: Option<String>,
    pub vis: Visibility,
    pub model_name: Ident,
    pub len: Option<proc_macro2::TokenStream>,

    pub _colon_token: Token![:],
    pub type_name: Ident,
    pub _brace_token: token::Brace,
    pub fields: Punctuated<SlintModelField, Token![,]>,
}

pub struct SlintModelField {
    pub vis: Visibility,
    pub direction: Direction,
    pub name: Ident,
    pub _colon_token: Token![:],
    pub ty: Type,
}

impl Parse for SlintModel {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let export_path = if input.peek(kw::export) {
            input.parse::<kw::export>()?;
            input.parse::<kw::to>()?;
            let path: syn::LitStr = input.parse()?;
            input.parse::<Token![;]>()?;
            Some(path.value())
        } else {
            None
        };

        let vis: Visibility = input.parse()?;
        
        let model_name: Ident = input.parse()?;
        
        let len = if input.peek(token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            Some(parse_len_expr(&content)?)
        } else {
            None
        };

        let content;
        Ok(SlintModel {
            export_path,
            vis,
            model_name,
            len,

            _colon_token: input.parse()?,
            type_name: input.parse()?,
            _brace_token: syn::braced!(content in input),
            fields: Punctuated::<SlintModelField, Token![,]>::parse_terminated(&content)?,
        })
    }
}

impl Parse for SlintModelField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        
        let direction = if input.peek(Token![in]) {
            input.parse::<Token![in]>()?;
            Direction::In
        } else if input.peek(Ident) {
             let fork = input.fork();
             let ident: Ident = fork.parse()?;
             if ident == "out" {
                 input.parse::<Ident>()?;
                 Direction::Out
             } else if ident == "model" {
                 input.parse::<Ident>()?;
                 Direction::Model
             } else {
                 return Err(syn::Error::new(ident.span(), "expected `in`, `out`, `model`"));
             }
        } else {
             return Err(input.error("expected `in`, `out`, `model`"));
        };

        let name = input.parse()?;
        let _colon_token = input.parse()?;
        let mut ty: Type = input.parse()?;
        if input.peek(token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            if content.is_empty() {
                ty = syn::parse_quote!([#ty]);
            } else {
                let len_tokens = parse_len_expr(&content)?;
                ty = syn::parse_quote!([#ty; #len_tokens]);
            }
        }

        Ok(SlintModelField {
            vis,
            direction,
            name,
            _colon_token,
            ty,
        })
    }
}
