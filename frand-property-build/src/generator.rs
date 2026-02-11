use crate::parser::{self, SlintModel};
use std::fs;
use std::path::{Path};
use quote::quote;
use syn::Type;
use heck::ToSnakeCase;

pub fn generate_slint_doc(input: &SlintModel) -> String {
    let (struct_name, struct_body, global_name, global_body, component_name, component_body) = generate_code_components(input);

    let struct_def = format!("export struct {} {{\n{}\n}}", struct_name, struct_body);
    let global_def = format!("export global {} {{\n{}\n}}", global_name, global_body);
    let component_def = format!("export component {} inherits Rectangle {{\n{}\n}}", component_name, component_body);

    format!(
        " 생성된 Slint 코드:\n```slint\n{struct_def}\n\n{global_def}\n\n{component_def}\n```"
    )
}

pub fn generate_slint_file(input: &SlintModel, output_dir: &Path) -> anyhow::Result<()> {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    let (struct_name, struct_body, global_name, global_body, component_name, component_body) = generate_code_components(input);

    let global_name_str = input.type_name.to_string();
    let name_for_file = if let Some(stripped) = global_name_str.strip_suffix("Global") {
        stripped
    } else {
        &global_name_str
    };

    let file_stem = name_for_file.to_snake_case();
    let file_name = format!("{}.slint", file_stem);
    let target_path = output_dir.join(&file_name);

    let original_content = if target_path.exists() {
        fs::read_to_string(&target_path)?
    } else {
        String::new()
    };
    let mut content = original_content.clone();

    let struct_header_pattern = format!("export struct {}", struct_name);
    content = replace_or_append_block(content, &struct_header_pattern, &struct_body, || {
        format!("export struct {} {{\n{}\n}}", struct_name, struct_body)
    });

    let global_header_pattern = format!("export global {}", global_name);
    content = replace_or_append_block(content, &global_header_pattern, &global_body, || {
        format!("export global {} {{\n{}\n}}", global_name, global_body)
    });

    let component_header_pattern = format!("export component {}", component_name);
    content = replace_or_append_block(content, &component_header_pattern, &component_body, || {
        format!("export component {} inherits Rectangle {{\n{}\n}}", component_name, component_body)
    });

    if content != original_content {
        fs::write(&target_path, content)?;
    }
    
    // Update index.slint
    update_index_slint(output_dir, &global_name, &file_name)?;

    Ok(())
}

fn update_index_slint(output_dir: &Path, global_name: &str, file_name: &str) -> anyhow::Result<()> {
    let index_path = output_dir.join("index.slint");
    let export_stmt = format!("export {{ {} }} from \"{}\";", global_name, file_name);
    
    let mut index_content = if index_path.exists() {
        fs::read_to_string(&index_path)?
    } else {
        String::new()
    };
    
    if !index_content.contains(&export_stmt) {
        if !index_content.is_empty() && !index_content.ends_with('\n') {
            index_content.push('\n');
        }
        index_content.push_str(&export_stmt);
        index_content.push('\n');
        
        fs::write(&index_path, index_content)?;
    }
    Ok(())
}

pub fn generate_code_components(input: &SlintModel) -> (String, String, String, String, String, String) {
    let global_name = input.type_name.to_string();
    let struct_name = format!("{}Data", input.type_name);

    let global_data = format!("    in-out property <[{struct_name}]> data: [{{}}];");
    
    let mut struct_fields = Vec::new();
    let mut global_fields = vec![global_data];

    let mut component_fields = Vec::new();

    // 기본 길이 프로퍼티 추가
    component_fields.push(format!("    in-out property <int> global-data-length: {global_name}.data.length;"));
    
    // 기본 인덱스 프로퍼티 추가
    component_fields.push("    in-out property <int> global-data-index: 0;".to_string());

    for field in &input.fields {
        let kebab_name = field.name.to_string().replace("_", "-");
        let slint_type = rust_type_to_slint_type(&field.ty);

        if is_unit_ty(&field.ty) {
             // 유닛 타입 -> 콜백
             // Global: callback name(int); 
             global_fields.push(format!("    callback {kebab_name}(int);"));
             
             // Component: callback name; name => { Global.name(global-data-index); }
             component_fields.push(format!("    callback global-{kebab_name};"));
             component_fields.push(format!("    global-{kebab_name} => {{ {global_name}.{kebab_name}(global-data-index); }}"));
        } else {
             struct_fields.push(
                format!("    {kebab_name}: {slint_type},")
            );

            match field.direction {
                parser::Direction::In => {
                    // Rust -> Slint (Slint 로직에 의해 변경되어 Rust 가 읽음)
                    // "in x: i32" -> "property <int> global-x;"
                    
                    component_fields.push(format!("    in-out property <{slint_type}> global-{kebab_name}: {global_name}.data[global-data-index].{kebab_name};"));
                    component_fields.push(format!("    changed global-{kebab_name} => {{ {global_name}.data[global-data-index].{kebab_name} = self.global-{kebab_name}; }}"));
                }
                parser::Direction::Out => {
                    // Rust -> Slint (Rust 가 쓰고 Slint 가 읽음)
                    // "out sum: i32" -> "property <int> global-sum: Global.data[index].sum;"
                    component_fields.push(format!("    out property <{slint_type}> global-{kebab_name}: {global_name}.data[global-data-index].{kebab_name};"));
                }
                parser::Direction::Model => {}
            }
        }
    }
    
    let component_name = format!("{}Component", input.type_name);
    
    (
        struct_name,
        struct_fields.join("\n"),
        global_name,
        global_fields.join("\n"),
        component_name,
        component_fields.join("\n")
    )
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
        "String" => "string".to_string(),
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

fn replace_or_append_block<F>(mut content: String, header_pattern: &str, new_body: &str, default_block_gen: F) -> String
where F: Fn() -> String
{
    match find_block_range(&content, header_pattern) {
        Some((start, end)) => {
            let mut new_content = String::with_capacity(content.len() + new_body.len());
            new_content.push_str(&content[..start+1]);
            new_content.push('\n');
            new_content.push_str(new_body);
            new_content.push('\n');
            new_content.push_str(&content[end..]);
            new_content
        },
        None => {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(&default_block_gen());
            content.push('\n');
            content
        }
    }
}

fn find_block_range(content: &str, header_pattern: &str) -> Option<(usize, usize)> {
    let header_start = content.find(header_pattern)?;
    let rest = &content[header_start..];
    let brace_offset = rest.find('{')?;
    let start_index = header_start + brace_offset;

    let mut balance = 0;

    for (i, c) in content[start_index..].char_indices() {
        if c == '{' {
            balance += 1;
        } else if c == '}' {
            balance -= 1;
            if balance == 0 {
                return Some((start_index, start_index + i));
            }
        }
    }

    None
}


