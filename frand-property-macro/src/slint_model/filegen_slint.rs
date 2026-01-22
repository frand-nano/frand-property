use super::*;
use std::path::{PathBuf};
use std::fs;
use std::env;
use crate::slint_model::parser::SlintModel;

pub fn generate_file(input: &SlintModel) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    let mut target_dir = PathBuf::from(&manifest_dir);
    target_dir.push("slint");
    target_dir.push("global");

    if let Err(e) = fs::create_dir_all(&target_dir) {
        eprintln!("Cargo:warning=Failed to create directory {:?}: {}", target_dir, e);
        return;
    }

    let (struct_name, struct_body, global_name, global_body, component_name, component_body) = codegen_slint::generate_code_components(input);

    let global_name_str = input.type_name.to_string();
    let name_for_file = if let Some(stripped) = global_name_str.strip_suffix("Global") {
        stripped
    } else {
        &global_name_str
    };

    let file_stem = camel_to_snake(name_for_file);
    let file_name = format!("{}.slint", file_stem);
    let target_path = target_dir.join(&file_name);

    let mut content = if target_path.exists() {
        match fs::read_to_string(&target_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Cargo:warning=Failed to read existing file {:?}: {}", target_path, e);
                return;
            }
        }
    } else {
        String::new()
    };

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

    if let Err(e) = fs::write(&target_path, content) {
        eprintln!("Cargo:warning=Failed to write file {:?}: {}", target_path, e);
    }
    
    // Update index.slint
    let index_path = target_dir.join("index.slint");
    let export_stmt = format!("export {{ {} }} from \"{}\";", global_name, file_name);
    
    let mut index_content = if index_path.exists() {
        match fs::read_to_string(&index_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Cargo:warning=Failed to read index file {:?}: {}", index_path, e);
                return;
            }
        }
    } else {
        String::new()
    };
    
    if !index_content.contains(&export_stmt) {
        if !index_content.is_empty() && !index_content.ends_with('\n') {
            index_content.push('\n');
        }
        index_content.push_str(&export_stmt);
        index_content.push('\n');
        
        if let Err(e) = fs::write(&index_path, index_content) {
             eprintln!("Cargo:warning=Failed to write index file {:?}: {}", index_path, e);
        }
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

fn camel_to_snake(s: &str) -> String {
    let mut snake = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                snake.push('_');
            }
            snake.push(c.to_ascii_lowercase());
        } else {
            snake.push(c);
        }
    }
    snake
}