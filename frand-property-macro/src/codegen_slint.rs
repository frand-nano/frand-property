use quote::quote;
use syn::Type;
use crate::parser::{Direction, SlintModel};

pub fn generate(input: &SlintModel) -> String {
    let global_name = input.type_name.to_string();
    let system_name = format!("{}System", global_name);

    let mut data_lines = Vec::new();
    let mut system_lines = Vec::new();

    for field in &input.fields {
        let name = field.name.to_string();
        let kebab_name = to_kebab_case(&name);
        let slint_type = rust_type_to_slint_type(&field.ty);
        
        match field.direction {
            Direction::Out => {
                data_lines.push(
                    format!("    in property <{slint_type}> {kebab_name};")
                );
            }
            Direction::In => {
                if slint_type == "void" {
                     data_lines.push(
                        format!("    callback {kebab_name}();")
                    );
                } else {
                    data_lines.push(
                        format!("    in-out property <{slint_type}> {kebab_name};")
                    );

                    data_lines.push(
                        format!("    callback changed-{kebab_name}({kebab_name}: {slint_type});")
                    );
                }
            }
            Direction::InOut => {
                data_lines.push(
                    format!("    in-out property <{slint_type}> {kebab_name};")
                );

                data_lines.push(
                    format!("    callback changed-{kebab_name}({kebab_name}: {slint_type});")
                );
            }
        }

        let system_prop_name = format!("system-{}", kebab_name);

        match field.direction {
            Direction::Out => {

            }
            Direction::In => {
                if slint_type == "void" {

                } else {
                    system_lines.push(
                        format!("    property <{slint_type}> {system_prop_name}: {global_name}.{kebab_name};")
                    );

                    system_lines.push(
                        format!("    changed {system_prop_name} => {{")
                    );

                    system_lines.push(
                        format!("        {global_name}.changed-{kebab_name}({system_prop_name})")
                    );

                    system_lines.push(
                        format!("    }}")
                    );
                }
            }
            Direction::InOut => {
                system_lines.push(
                    format!("    property <{slint_type}> {system_prop_name}: {global_name}.{kebab_name};")
                );

                system_lines.push(
                    format!("    changed {system_prop_name} => {{")
                );

                system_lines.push(
                    format!("        {global_name}.changed-{kebab_name}({system_prop_name})")
                );

                system_lines.push(
                    format!("    }}")
                );
            }
        }
    }

    let data_doc = format!(
        "export global {global_name} {{\n{}\n}}",
        data_lines.join("\n")
    );

    if system_lines.is_empty() {
        format!(" 생성된 Slint 코드:\n```slint\n{data_doc}\n```")
    } else {
        let system_doc = format!(
            "export component {} {{\n{}\n}}",
            system_name,
            system_lines.join("\n")
        );

        format!(" 생성된 Slint 코드:\n```slint\n{data_doc}\n\n{system_doc}\n```")
    }
}

fn to_kebab_case(s: &str) -> String {
    s.replace('_', "-")
}

fn rust_type_to_slint_type(ty: &Type) -> String {
    let type_str = quote!(#ty).to_string();
    // 매칭을 위해 타입 문자열에서 공백 제거
    let type_str_clean = type_str.replace(" ", "");
    
    
    match type_str_clean.as_str() {
        // Integers
        "i8" | "i16" | "i32" | "i64" | "isize" => "int".to_string(),
        "u8" | "u16" | "u32" | "u64" | "usize" => "int".to_string(),
        // Floating point
        "f32" | "f64" => "float".to_string(),
        // Boolean
        "bool" => "bool".to_string(),
        // Characters and strings
        "char" | "String" => "string".to_string(),
        // Unit type
        "()" => "void".to_string(),
        // Default fallback
        _ => type_str,
    }
}
