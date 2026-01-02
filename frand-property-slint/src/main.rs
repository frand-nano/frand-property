slint::include_modules!();

mod adder;
mod screen;

use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use frand_property::System;
use crate::adder::AdderModel;
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

    window.run()?;
    Ok(())
}
