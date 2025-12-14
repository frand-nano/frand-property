slint::include_modules!();

use frand_property::*;

slint_model! {
    ScreenModel: ScreenData {
        out current_screen: ScreenVariant,
        in confirm_start: (),
        in cancel_pay: (),
    }
}

slint_model! {
    AdderModel: AdderData {
        in x: i32,
        in y: i32,
        out sum: i32,
    }
}

#[tokio::main]
async fn main() {
    let window = MainWindow::new().unwrap(); // TODO: 에러 처리
    let screen_model = ScreenModel::<MainWindow>::new(&window);
    let adder_model = AdderModel::<MainWindow>::new(&window);

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

    tokio::spawn(async move {
        let mut x_rx = adder_model.x.receiver().clone();
        let mut y_rx = adder_model.y.receiver().clone();

        let mut x = 0;
        let mut y = 0;

        loop {
            tokio::select! {
                new_x = x_rx.changed() => {
                    x = new_x;
                }
                new_y = y_rx.changed() => {
                    y = new_y;
                }
            }
            adder_model.sum.send(x + y);
        }
    });

    window.run().unwrap(); // TODO: 에러 처리
}
