pub trait Model: Clone {
    type Sender;
    type Receiver;

    fn clone_sender(&self) -> Self::Sender;
    fn clone_receiver(&self) -> Self::Receiver;
}

pub trait ModelList {
    type Sender;
    type Receiver;

    fn clone_senders(&self) -> Vec<Self::Sender>;
    fn clone_receivers(&self) -> Vec<Self::Receiver>;
}

impl<T: Model> ModelList for [T] {
    type Sender = T::Sender;
    type Receiver = T::Receiver;

    fn clone_senders(&self) -> Vec<Self::Sender> {
        self.iter().map(|m| m.clone_sender()).collect()
    }

    fn clone_receivers(&self) -> Vec<Self::Receiver> {
        self.iter().map(|m| m.clone_receiver()).collect()
    }
}

impl<T: Model> Model for std::sync::Arc<T> {
    type Sender = T::Sender;
    type Receiver = T::Receiver;

    fn clone_sender(&self) -> Self::Sender {
        (**self).clone_sender()
    }

    fn clone_receiver(&self) -> Self::Receiver {
        (**self).clone_receiver()
    }
}
