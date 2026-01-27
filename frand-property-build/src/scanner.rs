use crate::parser::SlintModel;
use crate::generator::generate_slint_file;
use std::path::Path;
use walkdir::WalkDir;
use syn::visit::Visit;
use syn::{ItemMacro, parse_file};
use std::fs;

struct SlintModelVisitor {
    models: Vec<SlintModel>,
}

impl<'ast> Visit<'ast> for SlintModelVisitor {
    fn visit_item_macro(&mut self, i: &'ast ItemMacro) {
        if let Some(segment) = i.mac.path.segments.last() {
            if segment.ident == "slint_model" {
                 if let Ok(model) = syn::parse2::<SlintModel>(i.mac.tokens.clone()) {
                     self.models.push(model);
                 }
            }
        }
    }
}

pub fn generate_slint_files(src_dir: impl AsRef<Path>, output_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let src_dir = src_dir.as_ref();
    let output_dir = output_dir.as_ref();

    for entry in WalkDir::new(src_dir) {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue, // Skip files we cannot read
            };
            
            // Parse full file
            if let Ok(file) = parse_file(&content) {
                let mut visitor = SlintModelVisitor { models: Vec::new() };
                visitor.visit_file(&file);

                for model in visitor.models {
                    generate_slint_file(&model, output_dir)?;
                }
            }
        }
    }
    Ok(())
}
