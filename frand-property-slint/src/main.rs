slint::include_modules!();

mod adder;
mod screen;

use frand_property::System;
use crate::adder::AdderModel;
use crate::screen::ScreenModel;

#[tokio::main]
async fn main() {
    let window = MainWindow::new().unwrap(); // TODO: 에러 처리
    
    let screen_model = ScreenModel::<MainWindow>::new(&window);
    screen_model.start_system();
    
    let adder_model = AdderModel::<MainWindow>::new(&window);
    adder_model.start_system();

    window.run().unwrap(); // TODO: 에러 처리
}
