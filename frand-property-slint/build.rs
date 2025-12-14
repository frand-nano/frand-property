use slint_build;
    
fn main() -> Result<(), slint_build::CompileError> {
    slint_build::compile("slint/main.slint")
}
