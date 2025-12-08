use quote::quote;
use syn::Type;
use crate::parser::{Direction, SlintModel};

pub fn generate(input: &SlintModel) -> String {
    let global_name = input.type_name.to_string();
    let system_name = format!("{}System", global_name);

    let mut screen_data_lines = Vec::new();
    let mut screen_system_lines = Vec::new();

    for field in &input.fields {
        let name = field.name.to_string();
        let kebab_name = to_kebab_case(&name);
        let slint_type = rust_type_to_slint_type(&field.ty);
        
        match field.direction {
            Direction::Out => {
                screen_data_lines.push(format!("    in property <{}> {};", slint_type, kebab_name));
                screen_data_lines.push(format!("    callback changed-{}({}: {});", kebab_name, kebab_name, slint_type));
                
                screen_system_lines.push(format!("    property <{}> {}: {}.{};", slint_type, kebab_name, global_name, kebab_name));
                screen_system_lines.push(format!("    changed {} => {{", kebab_name));
                screen_system_lines.push(format!("        {}.changed-{}({})", global_name, kebab_name, kebab_name));
                screen_system_lines.push(format!("    }}"));
            }
            Direction::In => {
                if slint_type == "void" {
                    screen_data_lines.push(format!("    callback {}();", kebab_name));
                } else {
                    screen_data_lines.push(format!("    callback {}({});", kebab_name, slint_type));
                }
            }
            Direction::InOut => {
                screen_data_lines.push(format!("    property <{}> {};", slint_type, kebab_name));
                screen_data_lines.push(format!("    callback changed-{}({}: {});", kebab_name, kebab_name, slint_type));
                
                screen_system_lines.push(format!("    property <{}> {}: {}.{};", slint_type, kebab_name, global_name, kebab_name));
                screen_system_lines.push(format!("    changed {} => {{", kebab_name));
                screen_system_lines.push(format!("        {}.changed-{}({})", global_name, kebab_name, kebab_name));
                screen_system_lines.push(format!("    }}"));
            }
        }
    }

    let screen_data_doc = format!(
        "export global {} {{\n{}\n}}",
        global_name,
        screen_data_lines.join("\n")
    );

    let screen_system_doc = format!(
        "component {} {{\n{}\n}}",
        system_name,
        screen_system_lines.join("\n")
    );

    format!(
        " 생성된 Slint 코드:\n```slint\n{}\n\n{}\n```",
        screen_data_doc,
        screen_system_doc
    )
}

fn to_kebab_case(s: &str) -> String {
    s.replace('_', "-")
}

fn rust_type_to_slint_type(ty: &Type) -> String {
    let type_str = quote!(#ty).to_string();
    // 매칭을 위해 타입 문자열에서 공백 제거
    let type_str_clean = type_str.replace(" ", "");
    match type_str_clean.as_str() {
        "i32" => "int".to_string(),
        "f32" | "f64" => "float".to_string(),
        "bool" => "bool".to_string(),
        "String" => "string".to_string(),
        "()" => "void".to_string(),
        _ => type_str,
    }
}
