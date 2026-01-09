use std::fmt;
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Debug, Clone)]
pub struct Property<T, C = ()> {
    sender: Sender<T, C>,
    receiver: Receiver<T>,
}

pub struct Sender<T, C = ()> {
    component: C,
    sender: watch::Sender<T>,
    receiver: watch::Receiver<T>,
    set: Arc<dyn Fn(&C, T) + Send + Sync>,
}

impl<T, C> Clone for Sender<T, C> where C: Clone {
    fn clone(&self) -> Self {
        Self {
            component: self.component.clone(),
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
            set: self.set.clone(),
        }
    }
}

impl<T: fmt::Debug, C> fmt::Debug for Sender<T, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sender")
            .field("component", &"C")
            .field("sender", &self.sender)
            .field("receiver", &self.receiver)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct Receiver<T> {
    #[allow(dead_code)] sender: watch::Sender<T>,
    receiver: watch::Receiver<T>,
}

impl<T, C> Property<T, C> {
    pub fn sender(&self) -> &Sender<T, C> { &self.sender }
    pub fn receiver(&self) -> &Receiver<T> { &self.receiver }
    pub fn receiver_mut(&mut self) -> &mut Receiver<T> { &mut self.receiver }

    pub fn new(
        component: C,
        initial_value: T,
        set: impl Fn(&C, T) + 'static + Send + Sync,
    ) -> Self where T: Copy + Send + Sync {
        let channel = watch::channel(initial_value);

        Self {
            sender: Sender {
                component,
                sender: channel.0.clone(),
                receiver: channel.1.clone(),
                set: Arc::new(set),
            },
            receiver: Receiver {
                receiver: channel.1,
                sender: channel.0,
            },
        }
    }
}

impl<T> Receiver<T> {
    pub fn value(&self) -> T where T: Copy {
        *self.receiver.borrow()
    }

    pub fn has_changed(&self) -> bool {
        self.receiver.has_changed().unwrap_or(false)
    }

    pub async fn changed(&mut self) -> T where T: Copy + PartialEq {
        let last_value = self.value();

        *self.receiver
            .wait_for(|value| *value != last_value).await
            .unwrap_or_else(|_|
                // self 가 sender 와 receiver 를 모두 소유하기 때문에 sender 는 언제나 존재합니다.
                unreachable!("Sender is already dropped.")
            )
    }

    pub async fn notified(&mut self) -> T where T: Copy {
        self.receiver
            .changed().await
            .unwrap_or_else(|_|
                // self 가 sender 와 receiver 를 모두 소유하기 때문에 sender 는 언제나 존재합니다.
                unreachable!("Sender is already dropped.")
            );

        self.value()
    }
}

impl<T, C> Sender<T, C> {
    pub fn send(&self, value: T) where T: Copy + PartialEq {
        let current_value = *self.receiver.borrow();

        if value == current_value { return; }

        (self.set)(&self.component, value);

        self.sender.send(value)
            .unwrap_or_else(|_|
                // self 가 sender 와 receiver 를 모두 소유하기 때문에 receiver 는 언제나 존재합니다.
                unreachable!("Receiver is already dropped.")
            );
    }

    pub fn notify(&self) where T: Copy {
        let value = *self.receiver.borrow();

        (self.set)(&self.component, value);

        self.sender.send(value)
            .unwrap_or_else(|_|
                // self 가 sender 와 receiver 를 모두 소유하기 때문에 receiver 는 언제나 존재합니다.
                unreachable!("Receiver is already dropped.")
            );
    }
}