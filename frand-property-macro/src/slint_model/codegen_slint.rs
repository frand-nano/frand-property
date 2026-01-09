use quote::quote;
use syn::Type;
use super::parser;
use parser::{SlintModel};

pub fn generate(input: &SlintModel) -> String {
    let global_name = format!("{}Global", input.type_name);
    let struct_name = input.type_name.to_string();

    let mut struct_fields = Vec::new();
    let mut global_fields = Vec::new();
    let mut data_prop_emitted = false;

    let mut component_fields = Vec::new();
    
    // 기본 인덱스 프로퍼티 추가
    component_fields.push("    in-out property <int> global-index: 0;".to_string());

    for field in &input.fields {
        let kebab_name = field.name.to_string().replace("_", "-");
        let slint_type = rust_type_to_slint_type(&field.ty);

        if is_unit_ty(&field.ty) {
             // 유닛 타입 -> 콜백
             // Global: callback name(int); 
             global_fields.push(format!("    callback {kebab_name}(int);"));
             
             // Component: callback name; name => { Global.name(global-index); }
             component_fields.push(format!("    callback global-{kebab_name};"));
             component_fields.push(format!("    global-{kebab_name} => {{ {global_name}.{kebab_name}(global-index); }}"));
        } else {
             struct_fields.push(
                format!("    {kebab_name}: {slint_type},")
            );
            
            if !data_prop_emitted {
                 global_fields.push(format!("    in-out property <[{struct_name}]> data: [{{}}];"));
                 data_prop_emitted = true;
            }

            match field.direction {
                parser::Direction::In => {
                    // Rust -> Slint (Slint 로직에 의해 변경되어 Rust 가 읽음)
                    // "in x: i32" -> "property <int> global-x;"
                    
                    component_fields.push(format!("    in-out property <{slint_type}> global-{kebab_name};"));
                    component_fields.push(format!("    changed global-{kebab_name} => {{ {global_name}.data[global-index].{kebab_name} = self.global-{kebab_name}; }}"));
                }
                parser::Direction::Out => {
                    // Rust -> Slint (Rust 가 쓰고 Slint 가 읽음)
                    // "out sum: i32" -> "property <int> global-sum: Global.data[index].sum;"
                    component_fields.push(format!("    out property <{slint_type}> global-{kebab_name}: {global_name}.data[global-index].{kebab_name};"));
                }
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
    
    // 생성된 Slint 코드:
    // 1. 데이터 구조체 (Struct)
    // 2. 전역 싱글톤 (Global)
    // 3. 컴포넌트 (Component)

    let component_name = format!("{}Component", struct_name);
    let mut component_fields = Vec::new();
    
    // 기본 인덱스 프로퍼티 추가
    component_fields.push("    in-out property <int> global-index: 0;".to_string());

    for field in &input.fields {
        let kebab_name = field.name.to_string().replace("_", "-");
        let slint_type = rust_type_to_slint_type(&field.ty);

        if is_unit_ty(&field.ty) {
             // 유닛 타입 -> 콜백
             // Component: callback name; name => { Global.name(global-index); }
             component_fields.push(format!("    callback global-{kebab_name};"));
             component_fields.push(format!("    global-{kebab_name} => {{ {global_name}.{kebab_name}(global-index); }}"));
        } else {
            match field.direction {
                parser::Direction::In => {
                    // Rust -> Slint (Slint 로직에 의해 변경되어 Rust 가 읽음)
                    component_fields.push(format!("    in-out property <{slint_type}> global-{kebab_name}: {global_name}.data[global-index].{kebab_name};"));
                    component_fields.push(format!("    changed global-{kebab_name} => {{ {global_name}.data[global-index].{kebab_name} = self.global-{kebab_name}; }}"));
                }
                parser::Direction::Out => {
                    // Rust -> Slint (Rust 가 쓰고 Slint 가 읽음)
                    component_fields.push(format!("    out property <{slint_type}> global-{kebab_name}: {global_name}.data[global-index].{kebab_name};"));
                }
            }
        }
    }
    
    // export 제외 (사용자 요청)
    let component_def = format!(
        "component {} inherits Rectangle {{\n{}\n}}",
        component_name,
        component_fields.join("\n")
    );

    format!(
        " 생성된 Slint 코드:\n```slint\n{struct_def}\n\n{global_def}\n\n{component_def}\n```"
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

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_generate_adder() {
        let input: SlintModel = parse_quote! {
            pub AdderModel: AdderData {
                in x: i32,
                out sum: i32,
                in click: (),
            }
        };

        let output = generate(&input);
        
        println!("{}", output);

        // Data 구조체 정의 확인
        assert!(output.contains("export struct AdderData {"));
        assert!(output.contains("x: int,"));
        assert!(output.contains("sum: int,"));

        // Global 싱글톤 정의 확인
        assert!(output.contains("export global AdderDataGlobal {"));
        assert!(output.contains("in-out property <[AdderData]> data: [{}];"));
        assert!(output.contains("callback click(int);"));

        // Component 정의 확인
        assert!(output.contains("component AdderDataComponent inherits Rectangle {"));
        assert!(output.contains("in-out property <int> global-index: 0;"));
        
        // 'in' 프로퍼티 확인
        assert!(output.contains("in-out property <int> global-x: AdderDataGlobal.data[global-index].x;"));
        assert!(output.contains("changed global-x => { AdderDataGlobal.data[global-index].x = self.global-x; }"));

        // 'out' 프로퍼티 확인
        assert!(output.contains("out property <int> global-sum: AdderDataGlobal.data[global-index].sum;"));

        // 유닛 타입 콜백 확인
        assert!(output.contains("callback global-click;"));
        assert!(output.contains("global-click => { AdderDataGlobal.click(global-index); }"));
    }
}
