use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Ident, Token, Type, Visibility,
};

#[derive(PartialEq, Clone)]
pub enum Direction {
    In,
    Out,
}

pub struct SlintModel {
    pub vis: Visibility,
    pub model_name: Ident,
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
            Direction::In
        } else if input.peek(Ident) {
             let fork = input.fork();
             let ident: Ident = fork.parse()?;
             if ident == "out" {
                 input.parse::<Ident>()?;
                 Direction::Out
             } else {
                 return Err(syn::Error::new(ident.span(), "expected `in`, `out`"));
             }
        } else {
             return Err(input.error("expected `in`, `out`"));
        };

        let name = input.parse()?;
        let _colon_token = input.parse()?;
        let mut ty: Type = input.parse()?;
        if input.peek(token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            let len: syn::Expr = content.parse()?;
            ty = syn::parse_quote!([#ty; #len]);
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
