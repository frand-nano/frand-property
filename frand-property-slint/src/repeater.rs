use frand_property::*;
use crate::RepeaterData;
use crate::RepeaterDataGlobal;
use arraystring::ArrayString;
use arraystring::typenum::U41;

slint_model! {
    pub RepeaterModel: RepeaterData {
        in text: ArrayString<U20>,
        out repeated: ArrayString<U41>,
    }
}

impl<C: slint::ComponentHandle + 'static> RepeaterModel<C> {
    pub fn start(&self) {
        let mut text = self.text.clone();
        let repeated = self.repeated.clone();

        crate::spawn(async move {
            loop {
                let val = text.changed().await;
                
                let s = format!("{} {}", val, val);
                
                if let Ok(res) = ArrayString::<U41>::try_from_str(&s) {
                    repeated.send(res);
                }
            }
        });
    }
}
