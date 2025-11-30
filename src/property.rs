use crate::Transceiver;

pub struct Property<C, T> {
    component: C,
    transceiver: Transceiver<T>,
    set: fn(&C, T),
}

impl<C, T: Clone> Property<C, T> {
    pub fn new(
        component: C,
        initial_value: T,
        set: fn(&C, T),
    ) -> Self {
        Self {
            component,
            transceiver: Transceiver::new(initial_value),
            set,
        }
    }

    pub fn set(&self, value: T) {
        (self.set)(&self.component, value)
    }
}