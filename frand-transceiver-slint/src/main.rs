slint::include_modules!();

use frand_transceiver::*;

/*
slint_model! {
    ScreenModel: ScreenData<MainWindow> {
        current_screen: ScreenVariant,
        confirm_start: (),
        cancel_pay: (),
    }
}
*/

struct ScreenModel {
    current_screen: Property<slint::Weak<MainWindow>, ScreenVariant>,
    confirm_start: Property<slint::Weak<MainWindow>, ()>,
    cancel_pay: Property<slint::Weak<MainWindow>, ()>,
}
impl ScreenModel {
    pub fn new(
        component: &MainWindow,
    ) -> Self
    where
        MainWindow: slint::ComponentHandle,
    {
        let weak = std::sync::Arc::new(component.as_weak());

        let current_screen = Property::new(
            weak.clone(),
            Default::default(),
            |c, v| {
                c.upgrade_in_event_loop(move |c| {
                    c.global::<ScreenData>().set_current_screen(v)
                }).unwrap()
            },
        );
        let confirm_start = Property::new(
            weak.clone(),
            (),
            |_, _| {},
        );
        let cancel_pay = Property::new(
            weak.clone(),
            (),
            |_, _| {},
        );

        let current_screen_sender = current_screen.sender().clone();
        let confirm_start_sender = confirm_start.sender().clone();
        let cancel_pay_sender = cancel_pay.sender().clone();

        component.global::<ScreenData>().on_changed_current_screen(move |v| current_screen_sender.send(v));
        component.global::<ScreenData>().on_confirm_start(move || confirm_start_sender.send(()));
        component.global::<ScreenData>().on_cancel_pay(move || cancel_pay_sender.send(()));

        Self {
            current_screen,
            confirm_start,
            cancel_pay,
        }
    }
}

#[tokio::main]
async fn main() {
    let window = MainWindow::new().unwrap(); // TODO: Error handling
    let screen_model = ScreenModel::new(&window);

    tokio::spawn(async move {
        let mut confirm_start = screen_model.confirm_start.receiver().clone();
        let mut cancel_pay = screen_model.cancel_pay.receiver().clone();

        loop {
            screen_model.current_screen.send(ScreenVariant::Start);

            confirm_start.changed().await;

            screen_model.current_screen.send(ScreenVariant::Pay);

            cancel_pay.changed().await;
        }
    });

    window.run().unwrap(); // TODO: Error handling
}
