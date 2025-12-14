slint::include_modules!();

use frand_property::*;

mod adder;

slint_model! {
    ScreenModel: ScreenData {
        out current_screen: ScreenVariant,
        in confirm_start: (),
        in cancel_pay: (),
    }
}

#[tokio::main]
async fn main() {
    let window = MainWindow::new().unwrap(); // TODO: 에러 처리
    let screen_model = ScreenModel::<MainWindow>::new(&window);
    
    adder::run(&window);

    tokio::spawn(async move {
        let mut confirm_start = screen_model.confirm_start.receiver().clone();
        let mut cancel_pay = screen_model.cancel_pay.receiver().clone();

        loop {
            screen_model.current_screen.send(ScreenVariant::Start);

            confirm_start.changed().await;

            screen_model.current_screen.send(ScreenVariant::Pay);

            cancel_pay.changed().await;
        }
    });

    window.run().unwrap(); // TODO: 에러 처리
}
