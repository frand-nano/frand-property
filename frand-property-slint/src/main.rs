slint::include_modules!();

mod screen;
mod adder;
mod adder_array;
mod repeater;

use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use frand_property::System;
use crate::adder::AdderModel;
use crate::adder_array::AdderArrayModel;
use crate::screen::ScreenModel;

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    TermLogger::init(log::LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)
        .unwrap_or_else(|err| log::error!("{err}"));

    let window = MainWindow::new()?;

    let screen_model = ScreenModel::<MainWindow>::new(&window);
    screen_model.start_system();

    let adder_model = AdderModel::<MainWindow>::new(&window);
    adder_model.start_system();

    let adder_array_models = AdderArrayModel::<MainWindow>::new(&window);
    for model in adder_array_models {
        model.start_system();
    }

    let repeater_model = crate::repeater::RepeaterModel::<MainWindow>::new(&window);
    repeater_model.start_system();

    window.run()?;
    Ok(())
}
