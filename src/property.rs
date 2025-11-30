use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::watch;

pub struct Property<C, T: Copy + Send + Sync + PartialEq> {
    sender: Sender<C, T>,
    receiver: Receiver<T>,
}

#[derive(Clone)]
pub struct Sender<C, T> {
    component: Arc<C>,
    sender: watch::Sender<T>,
    set: fn(&C, T),
}

#[derive(Clone)]
pub struct Receiver<T> {
    receiver: watch::Receiver<T>,
}

impl<C, T: Copy + Send + Sync + PartialEq> Property<C, T> {
    pub fn sender(&self) -> &Sender<C, T> { &self.sender }
    pub fn receiver(&self) -> &Receiver<T> { &self.receiver }

    pub fn new(
        component: Arc<C>,
        initial_value: T,
        set: fn(&C, T),
    ) -> Self {
        let channel = watch::channel(initial_value);

        Self {
            sender: Sender {
                component,
                sender: channel.0,
                set,
            },
            receiver: Receiver { receiver: channel.1},
        }
    }
}

#[async_trait]
pub trait ReceiverExt<T: Copy + Send + Sync> {
    async fn changed(&mut self) -> T;
}

#[async_trait]
impl<T: Copy + Send + Sync> ReceiverExt<T> for Receiver<T> {
    async fn changed(&mut self) -> T {
        self.receiver.changed().await.unwrap(); // TODO: Error handling
        
        *self.receiver.borrow()
    }
}

pub trait SenderExt<T: Copy + Send + Sync> {
    fn send(&self, value: T);
}

impl<C, T: Copy + Send + Sync + PartialEq> SenderExt<T> for Sender<C, T> {
    fn send(&self, value: T) {
        (self.set)(&self.component, value);
        self.sender.send(value).unwrap(); // TODO: Error handling
    }
}

impl<C, T: Copy + Send + Sync + PartialEq> SenderExt<T> for Property<C, T> {
    fn send(&self, value: T) {
        self.sender.send(value)
    }
}