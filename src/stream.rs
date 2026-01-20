use std::borrow::Borrow;
use std::future::Future;
use tokio::task::JoinHandle;
pub use tokio_stream::StreamExt;
use tokio_stream::Stream;
use crate::{Receiver, Sender};

pub trait PropertyStreamExt: Stream {
    fn drive<F, Fut>(self, mut handler: F) -> impl Future<Output = ()> + Send + 'static
    where
        Self: Sized + Unpin + Send + 'static,
        Self::Item: Send,
        F: FnMut(Self::Item) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut stream = self;
        async move {
            while let Some(value) = stream.next().await {
                handler(value).await;
            }
        }
    }

    fn bind<T, C>(self, sender: Sender<T, C>) -> impl Future<Output = ()> + Send + 'static
    where
        Self: Sized + Stream<Item = T> + Unpin + Send + 'static,
        Self::Item: Send,
        T: Copy + PartialEq + Send + Sync + 'static,
        C: Send + Sync + Clone + 'static,
    {
        self.drive(move |value| {
            let sender = sender.clone();
            async move {
                sender.notify_with(value);
            }
        })
    }

    fn spawn<F, Fut>(self, handler: F) -> JoinHandle<()>
    where
        Self: Sized + Send + Unpin + 'static,
        Self::Item: Send,
        F: FnMut(Self::Item) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(self.drive(handler))
    }

    fn spawn_bind<T, C>(self, sender: Sender<T, C>) -> JoinHandle<()>
    where
        Self: Sized + Stream<Item = T> + Unpin + Send + 'static,
        T: Copy + PartialEq + Send + Sync + 'static,
        C: Send + Sync + Clone + 'static,
    {
        tokio::spawn(self.bind(sender))
    }
}

impl<S: Stream> PropertyStreamExt for S {}

pub trait PropertyIteratorExt: Iterator {
    fn spawn_bind<T, C>(self, senders: impl IntoIterator<Item = Sender<T, C>>) -> Vec<JoinHandle<()>>
    where
        Self: Sized,
        Self::Item: Borrow<Receiver<T>>,
        T: Copy + PartialEq + Send + Sync + 'static,
        C: Send + Sync + Clone + 'static,
    {
        self.zip(senders)
            .map(|(receiver, sender)|
                receiver.borrow().spawn_bind(sender)
            )
            .collect()
    }
}

impl<I> PropertyIteratorExt for I where I: Iterator {}