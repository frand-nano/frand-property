slint::include_modules!();

mod screen;
mod adder;
mod adder_array;
mod repeater;


use frand_property::System;
use crate::adder::AdderModel;
use crate::adder_array::AdderArrayModel;
use crate::screen::ScreenModel;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub const MODEL_LEN: usize = 2;

pub fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    tokio::spawn(future);

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(future);
}

pub async fn run() -> Result<(), slint::PlatformError> {
    init_logging();

    let window = MainWindow::new()?;

    let screen_model = ScreenModel::<MainWindow>::new(&window);
    screen_model.start_system();

    let adder_model = AdderModel::<MainWindow>::new(&window);
    adder_model.start_system();

    let adder_array_models = AdderArrayModel::<MainWindow>::new_array::<MODEL_LEN>(&window);
    for model in adder_array_models {
        model.start_system();
    }

    let repeater_model = crate::repeater::RepeaterModel::<MainWindow>::new(&window);
    repeater_model.start_system();

    window.run()?;
    Ok(())
}

fn init_logging() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
        TermLogger::init(log::LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
            .unwrap_or_else(|err| log::error!("{err}"));
    }
    #[cfg(target_arch = "wasm32")]
    {
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn wasm_main() -> Result<(), JsValue> {
    run().await.map_err(|e| JsValue::from_str(&e.to_string()))
}
