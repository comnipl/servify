use tokio::sync::oneshot;

use crate::serial::ServifyMiddle;
use std::any::Any;

pub trait ServifyProcessor: Sized {
    type Kind;
    type Context;

    fn serve(
        &mut self,
        middle: ServifyMiddle<Self::Kind, Self::Context>,
    ) -> impl std::future::Future<Output = Box<dyn Any + Send>>;

    fn launch(self, buffer_size: usize) -> (ServifyService<Self>, ServifyAccess<Self>) {
        let (tx, rx) = tokio::sync::mpsc::channel(buffer_size);
        (
            ServifyService {
                processor: self,
                rx,
            },
            ServifyAccess { tx },
        )
    }
}

pub struct ServifyService<T: ServifyProcessor + Sized> {
    processor: T,
    rx: tokio::sync::mpsc::Receiver<(
        ServifyMiddle<T::Kind, T::Context>,
        oneshot::Sender<Box<dyn Any + Send>>,
    )>,
}

impl<T: ServifyProcessor + Sized> ServifyService<T> {
    pub async fn listen(mut self) {
        while let Some((middle, tx)) = self.rx.recv().await {
            let response = self.processor.serve(middle).await;
            tx.send(response).unwrap();
        }
    }
}

pub struct ServifyAccess<T: ServifyProcessor + Sized> {
    pub tx: tokio::sync::mpsc::Sender<(
        ServifyMiddle<T::Kind, T::Context>,
        oneshot::Sender<Box<dyn Any + Send>>,
    )>,
}
