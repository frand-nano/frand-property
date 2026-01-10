use frand_property::*;
use crate::{AdderGlobal, MainWindow};

slint_model! {
    pub AdderModel: AdderGlobal {
        in x: i32,
        in y: i32,
        out sum: i32,
    }
}

impl AdderModel<MainWindow> {
    pub fn start(&self) {
        let mut x = self.x.clone();
        let mut y = self.y.clone();
        let sum = self.sum.clone();

        crate::spawn(async move {
            loop {
                tokio::select! {
                    x = x.changed() => {
                        sum.send(x + y.value());
                    }
                    y = y.changed() => {
                        sum.send(x.value() + y);
                    }
                }
            }
        });
    }
}
