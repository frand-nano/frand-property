slint::include_modules!();

use frand_transceiver::*;
use std::time::Duration;
use tokio::time::sleep;

slint_model! {
    ScreenModel: ScreenData<MainWindow> {
        current_screen: ScreenVariant,
    }
}

#[tokio::main]
async fn main() {
    let window = MainWindow::new().unwrap(); // TODO: Error handling
    let screen_model = ScreenModel::new(&window.as_weak());

    tokio::spawn(async move {
        screen_model.current_screen.set(ScreenVariant::Start);

        sleep(Duration::from_secs(1)).await;

        screen_model.current_screen.set(ScreenVariant::Pay);
    });

    window.run().unwrap(); // TODO: Error handling
}
