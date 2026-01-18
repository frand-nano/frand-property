
use async_trait::async_trait;
use tokio::task::JoinHandle;
use crate::{Receiver, Sender};

#[async_trait]
pub trait ReceiverGroup: Clone + Send + 'static {
    type Item: Copy + Send + Sync + 'static;

    fn value(&self) -> Self::Item;

    async fn notified(&mut self);

    /// 현재 그룹의 변경 사항을 지정된 `Sender`로 바인딩합니다.
    /// 값이 변경될 때마다 `Sender`로 새로운 값을 보냅니다.
    fn spawn_bind<C>(&self, sender: &Sender<Self::Item, C>) -> JoinHandle<()>
    where
        Self::Item: PartialEq,
        C: Send + Sync + Clone + 'static,
    {
        let mut group = self.clone();
        let sender_owned = sender.clone();

        tokio::spawn(async move {
            loop {
                group.notified().await;
                sender_owned.send(group.value());
            }
        })
    }
}

#[async_trait]
impl<T: Copy + Send + Sync + 'static> ReceiverGroup for Receiver<T> {
    type Item = T;

    fn value(&self) -> Self::Item {
        self.value()
    }

    async fn notified(&mut self) {
        self.notified().await;
    }
}

macro_rules! impl_tuple_merge {
    ($($T:ident),+) => {
        #[async_trait]
        impl<$($T),+> ReceiverGroup for ($($T),+)
        where
            $($T: ReceiverGroup),+
        {
            type Item = ($($T::Item),+);

            #[allow(non_snake_case)]
            fn value(&self) -> Self::Item {
                let ($($T),+) = self;
                ($($T.value()),+)
            }

            async fn notified(&mut self) {
                #[allow(non_snake_case)]
                let ($($T),+) = self;
                tokio::select! {
                    $( _ = $T.notified() => {} ),+
                }
            }
        }
    }
}

impl_tuple_merge!(A, B);
impl_tuple_merge!(A, B, C);
impl_tuple_merge!(A, B, C, D);
impl_tuple_merge!(A, B, C, D, E);
impl_tuple_merge!(A, B, C, D, E, F);
impl_tuple_merge!(A, B, C, D, E, F, G);
impl_tuple_merge!(A, B, C, D, E, F, G, H);
impl_tuple_merge!(A, B, C, D, E, F, G, H, I);
impl_tuple_merge!(A, B, C, D, E, F, G, H, I, J);
impl_tuple_merge!(A, B, C, D, E, F, G, H, I, J, K);
