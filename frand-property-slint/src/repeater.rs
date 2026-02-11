use frand_property::slint_model;
use crate::{MainWindow, RepeaterGlobal};
use arraystring::ArrayString;
use arraystring::typenum::{U20, U41};

slint_model! {
    export to "components/repeater";
    pub RepeaterModel: RepeaterGlobal {
        in text: ArrayString<U20>,
        out repeated: ArrayString<U41>,
    }
}

impl RepeaterModel<MainWindow> {
    pub fn start(&self) {
        let mut text = self.text.clone();
        let repeated = self.repeated.clone();

        crate::spawn(async move {
            loop {
                let val = text.modified().await;
                
                let s = format!("{} {}", val, val);

                if let Ok(res) = ArrayString::<U41>::try_from_str(&s) {
                    repeated.send(res);
                }
            }
        });
    }
}
