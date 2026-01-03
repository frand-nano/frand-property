use quote::quote;
use syn::Type;
use crate::parser::{SlintModel};

pub fn generate(input: &SlintModel) -> String {
    let global_name = format!("{}Global", input.type_name);
    let struct_name = input.type_name.to_string();

    let mut struct_fields = Vec::new();
    let mut global_fields = Vec::new();
    let mut data_prop_emitted = false;

    for field in &input.fields {
        let kebab_name = field.name.to_string().replace("_", "-");
        let slint_type = rust_type_to_slint_type(&field.ty);

        if is_unit_ty(&field.ty) {
             global_fields.push(format!("    callback {kebab_name}();"));
        } else {
             struct_fields.push(
                format!("    {kebab_name}: {slint_type},")
            );
            
            if !data_prop_emitted {
                 global_fields.push(format!("    in-out property <[{struct_name}]> data;"));
                 data_prop_emitted = true;
            }
        }
    }

    let struct_def = format!(
        "export struct {} {{\n{}\n}}",
        struct_name,
        struct_fields.join("\n")
    );

    let global_def = format!(
        "export global {} {{\n{}\n}}",
        global_name,
        global_fields.join("\n")
    );

    format!(" 생성된 Slint 코드:\n```slint\n{struct_def}\n\n{global_def}\n```")
}

fn rust_type_to_slint_type(ty: &Type) -> String {
    if let Type::Array(type_array) = ty {
        let inner_type = rust_type_to_slint_type(&type_array.elem);
        return format!("[{}]", inner_type);
    }

    let type_str = quote!(#ty).to_string();

    // 매칭을 위해 타입 문자열에서 공백 제거
    let type_str_clean = type_str.replace(" ", "");

    match type_str_clean.as_str() {
        // 정수형
        "i8" | "i16" | "i32" | "i64" | "isize" => "int".to_string(),
        "u8" | "u16" | "u32" | "u64" | "usize" => "int".to_string(),
        // 부동 소수점
        "f32" | "f64" => "float".to_string(),
        // 불리언
        "bool" => "bool".to_string(),
        // 문자
        "char" => "string".to_string(),
        // 문자열
        s if (s.starts_with("ArrayString<") || s.starts_with("ArrayString::<")) && s.ends_with(">") => "string".to_string(),
        // 유닛 타입
        "()" => "void".to_string(),
        // 기본 폴백
        _ => type_str,
    }
}

fn is_unit_ty(ty: &Type) -> bool {
    if let Type::Tuple(t) = ty {
        t.elems.is_empty()
    } else {
        false
    }
}
