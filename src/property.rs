use std::fmt;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::WatchStream;
use crate::ReceiverGroup;
use crate::stream::PropertyStreamExt;

#[derive(Debug, Clone)]
pub struct Property<T, C = ()> {
    sender: Sender<T, C>,
    receiver: Receiver<T>,
}

impl<T: Copy + Send + Sync> From<T> for Property<T> {
    fn from(value: T) -> Self {
        Self::new((), value, |_, _| {})
    }
}

impl<T: Default + Copy + Send + Sync> Default for Property<T> {
    fn default() -> Self {
        Self::from(T::default())
    }
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

impl<T: Debug, C> Debug for Sender<T, C> {
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

    pub fn has_notified(&self) -> bool {
        self.receiver.has_changed().unwrap_or(false)
    }

    pub fn mark_notified(&mut self) {
        self.receiver.mark_changed();
    }

    pub fn mark_unnotified(&mut self) {
        self.receiver.mark_unchanged();
    }

    pub fn clone_notified(&self) -> Self {
        let mut result = Self {
            receiver: self.receiver.clone(),
            sender: self.sender.clone(),
        };

        result.mark_notified();

        result
    }

    pub fn clone_unnotified(&self) -> Self {
        let mut result = Self {
            receiver: self.receiver.clone(),
            sender: self.sender.clone(),
        };

        result.mark_unnotified();

        result
    }

    pub async fn modified(&mut self) -> T where T: Copy + PartialEq {
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
    
    pub fn stream(&self) -> WatchStream<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        WatchStream::new(self.receiver.clone())
    }

    pub fn spawn<F, Fut>(&self, handler: F) -> JoinHandle<()>
    where
        T: Clone + Send + Sync + 'static,
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.stream().spawn(handler)
    }

    pub fn spawn_bind<C>(&self, sender: Sender<T, C>) -> JoinHandle<()>
    where
        T: Copy + PartialEq + Send + Sync + 'static,
        C: Send + Sync + Clone + 'static,
    {
        self.stream().spawn_bind(sender)
    }
    
    pub fn from_spawn_bind<G: ReceiverGroup<Item = T>>(group: G) -> Self 
    where T: Default + Copy + PartialEq + Send + Sync + 'static {
        let property = Property::<T>::default();

        group.spawn_bind(property.sender);
        
        property.receiver
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

        self.notify_with(value);
    }

    pub fn notify_with(&self, value: T) where T: Copy {
        (self.set)(&self.component, value);

        self.sender.send(value)
            .unwrap_or_else(|_|
                // self 가 sender 와 receiver 를 모두 소유하기 때문에 receiver 는 언제나 존재합니다.
                unreachable!("Receiver is already dropped.")
            );
    }
}

pub trait PropertyList<T, C> {
    fn into_senders(self) -> Vec<Sender<T, C>>;
    fn into_receivers(self) -> Vec<Receiver<T>>;
}

impl<'a, T, C, I> PropertyList<T, C> for I
where
    I: IntoIterator<Item = &'a Property<T, C>>,
    T: 'a + Clone,
    C: 'a + Clone,
{
    fn into_senders(self) -> Vec<Sender<T, C>> {
        self.into_iter().map(|p| p.sender.clone()).collect()
    }

    fn into_receivers(self) -> Vec<Receiver<T>> {
        self.into_iter().map(|p| p.receiver.clone()).collect()
    }
}

