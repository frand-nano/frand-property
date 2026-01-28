use std::future::Future;
use frand_property::SlintSingleton;
use crate::adder::AdderModel;
use crate::adders::AddersModel;
use crate::screen::ScreenModel;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod screen;
mod adder;
mod adders;
mod repeater;

slint::include_modules!();

pub const MODEL_LEN: usize = 2;

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    tokio::spawn(future);

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(future);
}

pub fn run() -> Result<(), slint::PlatformError> {
    init_logging();

    let window = MainWindow::new()?;
    window.init_singleton();

    let screen_model = ScreenModel::<MainWindow>::new(&window);
    screen_model.start();

    let adder_model = AdderModel::<MainWindow>::new(&window);
    adder_model.start();

    let adder_vec_models = AddersModel::<MainWindow>::new(&window);
    for model in adder_vec_models.iter() {
        model.start();
    }

    let repeater_model = repeater::RepeaterModel::<MainWindow>::new(&window);
    repeater_model.start();

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
pub fn wasm_main() -> Result<(), JsValue> {
    run().map_err(|e| JsValue::from_str(&e.to_string()))
}
