use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Ident, Token, Type, Visibility,
};

// Model 구조체 정의
pub struct Model {
    pub vis: Visibility,
    pub model_name: Ident,

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
        let content;
        Ok(Model {
            vis: input.parse()?,
            model_name: input.parse()?,

            _brace_token: syn::braced!(content in input),
            fields: content.parse_terminated(ModelField::parse, Token![,])?,
        })
    }
}

impl Parse for ModelField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        
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

        // in/out 키워드 없음, 바로 식별자 파싱
        let name = input.parse()?;
        let _colon_token = input.parse()?;
        let mut ty: Type = input.parse()?;
        
        // 타입 뒤에 배열 크기가 오는지 확인 (예: i32[PROP_LEN])
        if input.peek(token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            let len: syn::Expr = content.parse()?;
            ty = syn::parse_quote!([#ty; #len]);
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
