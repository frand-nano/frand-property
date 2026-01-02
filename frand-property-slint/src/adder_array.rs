use frand_property::*;
use crate::AdderArrayData;

const PROP_LEN: usize = 2;

slint_model! {
    pub AdderArrayModel: AdderArrayData {
        in x: i32[PROP_LEN],
        in y: i32[PROP_LEN],
        out sum: i32[PROP_LEN],
    }
}

impl<C: slint::ComponentHandle + 'static> System for AdderArrayModel<C> {
    fn start_system(&self) {
        let mut x_1 = self.x[0].clone();
        let mut y_1 = self.y[0].clone();
        let sum_1 = self.sum[0].clone();

        let mut x_2 = self.x[1].clone();
        let mut y_2 = self.y[1].clone();
        let sum_2 = self.sum[1].clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    x = x_1.changed() => {
                        sum_1.send(x + y_1.value());
                    }
                    y = y_1.changed() => {
                        sum_1.send(x_1.value() + y);
                    }
                    x = x_2.changed() => {
                        sum_2.send(x + y_2.value());
                    }
                    y = y_2.changed() => {
                        sum_2.send(x_2.value() + y);
                    }
                }
            }
        });
    }
}
