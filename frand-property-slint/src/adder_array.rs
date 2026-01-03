use frand_property::*;
use crate::AdderArrayDataGlobal;
use crate::AdderArrayData;

const MODEL_LEN: usize = 2;
//const PROP_LEN: usize = 2;

slint_model! {
    pub AdderArrayModel[MODEL_LEN]: AdderArrayData {
        in x: i32,
        in y: i32,
        out sum: i32,
    }
}

impl<C: slint::ComponentHandle + 'static> System for AdderArrayModel<C> {
    fn start_system(&self) {
        let mut x = self.x.clone();
        let mut y = self.y.clone();
        let sum = self.sum.clone();

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
