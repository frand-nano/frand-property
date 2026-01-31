use proc_macro2::{TokenStream};
use quote::quote;
use syn::{Type};

pub fn resolve_type(ty: &Type) -> TokenStream {
    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            if seg.ident == "ArrayString" {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(Type::Path(type_path))) = args.args.first() {
                         if let Some(inner_seg) = type_path.path.segments.last() {
                             let ident_str = inner_seg.ident.to_string();
                             if ident_str.starts_with('U') {
                                 if ident_str[1..].parse::<u32>().is_ok() {
                                     let n = &inner_seg.ident;
                                     return quote! { frand_property::arraystring::ArrayString<frand_property::arraystring::typenum::#n> };
                                 }
                             }
                         }
                    }
                }
            }
        }
    }
    quote! { #ty }
}

pub fn is_array_string_type(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            return seg.ident == "ArrayString";
        }
    }
    false
}

pub fn is_std_string_type(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            return seg.ident == "String";
        }
    }
    false
}

pub fn is_special_string_type(ty: &Type) -> bool {
    is_array_string_type(ty) || is_std_string_type(ty)
}

pub fn is_unit_ty(ty: &Type) -> bool {
    if let Type::Tuple(t) = ty {
        t.elems.is_empty()
    } else {
        false
    }
}

pub fn generate_vec_init_tokens(len: impl quote::ToTokens, elem_ty: impl quote::ToTokens) -> TokenStream {
    quote! {
        {
            let mut v: std::vec::Vec<#elem_ty> = std::vec::Vec::with_capacity(#len);
            for _ in 0..#len {
                v.push(<#elem_ty as Default>::default());
            }
            v
        }
    }
}


