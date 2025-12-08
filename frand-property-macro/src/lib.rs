use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;
use crate::parser::SlintModel;

mod parser;
mod codegen_rust;
mod codegen_slint;

#[proc_macro]
#[proc_macro_error]
pub fn slint_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SlintModel);

    // 1. Slint 문서 생성
    let slint_doc = codegen_slint::generate(&input);

    // 2. Rust 코드 생성 (문서 포함)
    let expanded = codegen_rust::generate(&input, quote::quote! {
        #[doc = #slint_doc]
    });

    TokenStream::from(expanded)
}