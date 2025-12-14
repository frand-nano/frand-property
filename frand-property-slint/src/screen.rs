use frand_property::*;
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
        let current_screen = self.current_screen.sender().clone();
        let mut confirm_start = self.confirm_start.receiver().clone();
        let mut cancel_pay = self.cancel_pay.receiver().clone();

        tokio::spawn(async move {
            loop {
                current_screen.send(ScreenVariant::Start);

                confirm_start.changed().await;

                current_screen.send(ScreenVariant::Pay);

                cancel_pay.changed().await;
            }
        });
    }
}