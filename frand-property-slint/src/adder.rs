use frand_property::*;
use crate::AdderData;

slint_model! {
    pub AdderModel: AdderData {
        in x: i32,
        in y: i32,
        out sum: i32,
    }
}

impl<C: slint::ComponentHandle + 'static> System for AdderModel<C> {
    fn start_system(&self) {
        let mut x = self.x.receiver().clone();
        let mut y = self.y.receiver().clone();
        let sum = self.sum.sender().clone();

        tokio::spawn(async move {
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
