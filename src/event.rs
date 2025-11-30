use crate::Transceiver;

pub struct Event<C> {
    component: C,
    transceiver: Transceiver<Option<()>>,
}

impl<'a, C> Event<C> {
    pub fn new(
        component: C,
    ) -> Self {
        Self {
            component,
            transceiver: Transceiver::new(None),
        }
    }
}