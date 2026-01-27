use slint_build;
use frand_property_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    frand_property_build::scanner::generate_slint_files("src", "slint/global")?;
    slint_build::compile("slint/main.slint")?;
    Ok(())
}
