use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::watch;

pub struct Property<C, T> {
    sender: Sender<C, T>,
    receiver: Receiver<T>,
}

pub struct Sender<C, T> {
    component: Arc<C>,
    sender: watch::Sender<T>,
    receiver: watch::Receiver<T>,
    set: fn(&C, T),
}

impl<C, T> Clone for Sender<C, T> {
    fn clone(&self) -> Self {
        Self {
            component: self.component.clone(),
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
            set: self.set,
        }
    }
}

#[derive(Clone)]
pub struct Receiver<T> {
    #[allow(dead_code)] sender: watch::Sender<T>,
    receiver: watch::Receiver<T>,
}

impl<C, T> Property<C, T> {
    pub fn sender(&self) -> &Sender<C, T> { &self.sender }
    pub fn receiver(&self) -> &Receiver<T> { &self.receiver }

    pub fn new(
        component: Arc<C>,
        initial_value: T,
        set: fn(&C, T),
    ) -> Self where T: Copy + Send + Sync {
        let channel = watch::channel(initial_value);

        Self {
            sender: Sender {
                component,
                sender: channel.0.clone(),
                receiver: channel.1.clone(),
                set,
            },
            receiver: Receiver {
                receiver: channel.1,
                sender: channel.0,
            },
        }
    }
}

#[async_trait]
pub trait ReceiverExt<T: Copy + Send + Sync> {
    fn value(&self) -> T;
    async fn changed(&mut self) -> T where T: PartialEq;
    async fn notified(&mut self) -> T;
}

#[async_trait]
impl<T: Copy + Send + Sync> ReceiverExt<T> for Receiver<T> {
    fn value(&self) -> T {
        *self.receiver.borrow()
    }

    async fn changed(&mut self) -> T where T: PartialEq {
        let last_value = self.value();

        *self.receiver
            .wait_for(|value| *value != last_value).await
            .unwrap_or_else(|_|
                // self 가 sender 와 receiver 를 모두 소유하기 때문에 sender 는 언제나 존재합니다.
                unreachable!("Sender is already dropped.")
            )
    }

    async fn notified(&mut self) -> T {
        self.receiver
            .changed().await
            .unwrap_or_else(|_|
                // self 가 sender 와 receiver 를 모두 소유하기 때문에 sender 는 언제나 존재합니다.
                unreachable!("Sender is already dropped.")
            );

        self.value()
    }
}

pub trait SenderExt<T: Copy + Send + Sync> {
    fn send(&self, value: T) where T: PartialEq;
    fn notify(&self);
}

impl<C, T: Copy + Send + Sync> SenderExt<T> for Sender<C, T> {
    fn send(&self, value: T) where T: PartialEq {
        let current_value = *self.receiver.borrow();

        if value == current_value { return; }

        (self.set)(&self.component, value);

        self.sender.send(value)
            .unwrap_or_else(|_|
                // self 가 sender 와 receiver 를 모두 소유하기 때문에 receiver 는 언제나 존재합니다.
                unreachable!("Receiver is already dropped.")
            );
    }

    fn notify(&self) {
        let value = *self.receiver.borrow();

        (self.set)(&self.component, value);

        self.sender.send(value)
            .unwrap_or_else(|_|
                // self 가 sender 와 receiver 를 모두 소유하기 때문에 receiver 는 언제나 존재합니다.
                unreachable!("Receiver is already dropped.")
            );
    }
}

impl<C, T: Copy + Send + Sync> SenderExt<T> for Property<C, T> {
    fn send(&self, value: T) where T: PartialEq {
        self.sender.send(value)
    }

    fn notify(&self) {
        self.sender.notify()
    }
}