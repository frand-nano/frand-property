use async_trait::async_trait;
use tokio::sync::watch;

pub struct Transceiver<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

pub struct Sender<T> {
    sender: watch::Sender<T>,
}

pub struct Receiver<T> {
    receiver: watch::Receiver<T>,
}

impl<T: Clone> Transceiver<T> {
    pub fn sender(&self) -> &Sender<T> { &self.sender }
    pub fn receiver(&self) -> &Receiver<T> { &self.receiver }

    pub fn new(initial_value: T) -> Self {
        let channel = watch::channel(initial_value);

        Self {
            sender: Sender { sender: channel.0 },
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

#[async_trait]
impl<T: Copy + Send + Sync> ReceiverExt<T> for Transceiver<T> {
    async fn changed(&mut self) -> T {
        self.receiver.changed().await
    }
}