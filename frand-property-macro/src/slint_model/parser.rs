use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Ident, Token, Type, Visibility,
};
use crate::common::parse_len_expr;

#[derive(PartialEq, Clone)]
pub enum Direction {
    In,
    Out,
    Model,
}

pub struct SlintModel {
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
            vis,
            model_name,
            len,

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
            let len_tokens = parse_len_expr(&content)?;
            ty = syn::parse_quote!([#ty; #len_tokens]);
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
