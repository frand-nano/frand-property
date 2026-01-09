use tokio::task::JoinHandle;
pub use tokio_stream::StreamExt;
use tokio_stream::Stream;
use crate::{Sender};

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
                sender.send(value);
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