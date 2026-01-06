use frand_property::*;
use crate::ScreenDataGlobal;
use crate::ScreenData;
use crate::ScreenVariant;

slint_model! {
    pub ScreenModel: ScreenData {
        out current_screen: ScreenVariant,
        in confirm_start: (),
        in cancel_pay: (),
    }
}

impl<C: slint::ComponentHandle + 'static> System for ScreenModel<C> {
    fn start_system(&self) {
        let current_screen = self.current_screen.clone();
        let mut confirm_start = self.confirm_start.clone();
        let mut cancel_pay = self.cancel_pay.clone();

        crate::spawn(async move {
            loop {
                current_screen.send(ScreenVariant::Start);

                confirm_start.notified().await;

                current_screen.send(ScreenVariant::Pay);

                cancel_pay.notified().await;
            }
        });
    }
}