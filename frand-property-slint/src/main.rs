slint::include_modules!();

mod adder;
mod screen;

use frand_property::System;
use crate::adder::AdderModel;
use crate::screen::ScreenModel;

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    let window = MainWindow::new()?;

    let screen_model = ScreenModel::<MainWindow>::new(&window);
    screen_model.start_system();

    let adder_model = AdderModel::<MainWindow>::new(&window);
    adder_model.start_system();

    window.run()?;
    Ok(())
}
