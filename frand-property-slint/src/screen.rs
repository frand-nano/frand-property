use frand_property::slint_model;
use crate::{MainWindow, ScreenGlobal};
use crate::ScreenVariant;

slint_model! {
    export to "screen/screen";
    pub ScreenModel: ScreenGlobal {
        out current_screen: ScreenVariant,
        in confirm_start: (),
        in cancel_pay: (),
    }
}

impl ScreenModel<MainWindow> {
    pub fn start(&self) {
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