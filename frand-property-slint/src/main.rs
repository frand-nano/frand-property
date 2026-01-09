
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    frand_property_slint::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
