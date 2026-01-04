use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

mod slint_model;
mod model;
mod common;

#[proc_macro]
#[proc_macro_error]
pub fn slint_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as slint_model::parser::SlintModel);

    // 1. Slint 문서 생성
    let slint_doc = slint_model::codegen_slint::generate(&input);

    // 2. Rust 코드 생성 (문서 포함)
    let expanded = slint_model::codegen_rust::generate(&input, quote::quote! {
        #[doc = #slint_doc]
    });

    TokenStream::from(expanded)
}

#[proc_macro]
#[proc_macro_error]
pub fn model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as model::parser::Model);
    let expanded = model::codegen_rust::generate(&input);
    TokenStream::from(expanded)
}