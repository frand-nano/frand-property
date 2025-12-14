use crate::{AdderData, MainWindow};
use frand_property::slint_model;
use frand_property::*;
use slint::ComponentHandle;

slint_model! {
    pub AdderModel: AdderData {
        in x: i32,
        in y: i32,
        out sum: i32,
    }
}

pub fn run(window: &MainWindow) {
    let adder_model = AdderModel::<MainWindow>::new(window);

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
}
