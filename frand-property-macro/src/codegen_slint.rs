use quote::quote;
use syn::{Type, Expr, Lit};
use crate::parser::{Direction, SlintModel};

pub fn generate(input: &SlintModel) -> String {
    let global_name = input.type_name.to_string();
    let system_name = format!("{}System", global_name);

    let mut data_lines = Vec::new();
    let mut system_lines = Vec::new();

    for field in &input.fields {
        let name = field.name.to_string();
        let kebab_name = to_kebab_case(&name);
        
        let (slint_type, array_len) = if let Type::Array(arr) = &field.ty {
            let elem_type = rust_type_to_slint_type(&arr.elem);
            let len = if let Expr::Lit(lit) = &arr.len {
                if let Lit::Int(lit_int) = &lit.lit {
                    lit_int.base10_parse::<usize>().unwrap_or(0)
                } else { 0 }
            } else { 0 }; // Fallback for non-literal lengths
            (format!("[{}]", elem_type), Some(len))
        } else {
            (rust_type_to_slint_type(&field.ty), None)
        };
        
        // 배열의 기본값
        let init_val = if let Some(len) = array_len {
            let default_elem = if slint_type.contains("int") || slint_type.contains("float") { "0" } else { "\"\"" };
            let elems = vec![default_elem; len];
            format!(": [{},];", elems.join(", "))
        } else {
            ";".to_string()
        };

        match field.direction {
            Direction::Out => {
                data_lines.push(
                    format!("    in property <{slint_type}> {kebab_name}{init_val}")
                );
            }
            Direction::In => {
                if slint_type == "void" {
                     data_lines.push(
                        format!("    callback {kebab_name}();")
                    );
                } else {
                    data_lines.push(
                        format!("    in-out property <{slint_type}> {kebab_name}{init_val}")
                    );
                    
                    if let Some(_) = array_len {
                         data_lines.push(
                            format!("    callback changed-{kebab_name}(index: int, value: {inside_type});", inside_type=slint_type.trim_matches(|c| c == '[' || c == ']'))
                        );
                    } else {
                         data_lines.push(
                            format!("    callback changed-{kebab_name}({kebab_name}: {slint_type});")
                        );
                    }
                }
            }
            Direction::InOut => {
                data_lines.push(
                    format!("    in-out property <{slint_type}> {kebab_name}{init_val}")
                );

                if let Some(_) = array_len {
                     data_lines.push(
                        format!("    callback changed-{kebab_name}(index: int, value: {inside_type});", inside_type=slint_type.trim_matches(|c| c == '[' || c == ']'))
                    );
                } else {
                    data_lines.push(
                        format!("    callback changed-{kebab_name}({kebab_name}: {slint_type});")
                    );
                }
            }
        }

        let system_prop_name = format!("system-{}", kebab_name);

        match field.direction {
            Direction::Out => {

            }
            Direction::In => {
                if slint_type == "void" {

                } else {
                    if let Some(len) = array_len {
                         let inside_type = slint_type.trim_matches(|c| c == '[' || c == ']');
                         system_lines.push(
                             format!("    for value[i] in {global_name}.{kebab_name} : Rectangle {{")
                         );
                         system_lines.push(
                             format!("        property <{inside_type}> {system_prop_name}: value;")
                         );
                         system_lines.push(
                             format!("        changed {system_prop_name} => {{")
                         );
                         system_lines.push(
                             format!("            {global_name}.changed-{kebab_name}(i, {system_prop_name});")
                         );
                         system_lines.push(
                             format!("        }}")
                         );
                         system_lines.push(
                             format!("    }}")
                         );
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
            }
            Direction::InOut => {
                 if let Some(len) = array_len {
                         let inside_type = slint_type.trim_matches(|c| c == '[' || c == ']');
                         system_lines.push(
                             format!("    for value[i] in {global_name}.{kebab_name} : Rectangle {{")
                         );
                         system_lines.push(
                             format!("        property <{inside_type}> {system_prop_name}: value;")
                         );
                         system_lines.push(
                             format!("        changed {system_prop_name} => {{")
                         );
                         system_lines.push(
                             format!("            {global_name}.changed-{kebab_name}(i, {system_prop_name});")
                         );
                         system_lines.push(
                             format!("        }}")
                         );
                         system_lines.push(
                             format!("    }}")
                         );
                } else {
                    system_lines.push(
                        format!("    property <{slint_type}> {system_prop_name}: {global_name}.{kebab_name};")
                    );

                    system_lines.push(
                        format!("    changed {system_prop_name} => {{")
                    );

                    system_lines.push(
                        format!("        {global_name}.changed-{kebab_name}({system_prop_name});")
                    );

                    system_lines.push(
                        format!("    }}")
                    );
                }
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
        // 정수형
        "i8" | "i16" | "i32" | "i64" | "isize" => "int".to_string(),
        "u8" | "u16" | "u32" | "u64" | "usize" => "int".to_string(),
        // 부동 소수점
        "f32" | "f64" => "float".to_string(),
        // 불리언
        "bool" => "bool".to_string(),
        // 문자 및 문자열
        "char" | "String" => "string".to_string(),
        // 유닛 타입
        "()" => "void".to_string(),
        // 기본 폴백
        _ => type_str,
    }
}
