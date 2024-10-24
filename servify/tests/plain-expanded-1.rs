

mod counter {
    use std::{any::Any, future::Future, pin::Pin};

    use servify::processor::ServifyProcessor;

    // これ切り出せるくね！？！？
    pub trait Dispatchable {
        fn send(&self, kind: Kind, payload: Box<dyn Any>) -> Pin<Box<dyn '_ + Future<Output = Box<dyn Any>>>>;
    }

    pub struct Counter<T: Dispatchable>(T);

    impl<T: Dispatchable> Dispatchable for Counter<T> {
        fn send(&self, kind: Kind, payload: Box<dyn Any>) -> Pin<Box<dyn '_ + Future<Output = Box<dyn Any>>>> {
            self.0.send(kind, payload)
        }
    }

    pub enum Kind {
        IncrementAndGet,
    }

    pub(super) trait ImplIncrementAndGet {
        fn process_any(&mut self, ctx: Context, payload: Box<dyn Any>) -> Box<dyn Any>;
    }

    pub(super) struct Processor {
        pub(super) count: u32,
    }

    pub(super) struct MessagePassing {
        tx: tokio::sync::mpsc::Sender<(Kind, Context, Box<dyn Any>, tokio::sync::oneshot::Sender<Box<dyn Any>>)>,
    }

    impl Dispatchable for MessagePassing {
        fn send(&self, kind: Kind, payload: Box<dyn Any>) -> Pin<Box<dyn '_ + Future<Output = Box<dyn Any>>>> {
            Box::pin(async move {
                let (tx, rx) = tokio::sync::oneshot::channel();
                self.tx.send((kind, Context::MessagePassing, payload, tx)).await.unwrap();
                rx.await.unwrap()
            })
        }
    }

    impl ServifyProcessor for Processor {
        type Context = Context;
    }

    pub(super) enum Context {
        Direct,
        MessagePassing,
    }


}


mod counter_increment_and_get {
    use std::any::Any;

    use super::counter;
    use servify::processor::ServifyProcessor;

    struct Request {
        amount: u32,
    }

    trait Process {
        fn process_internal(&mut self, ctx: <counter::Processor as ServifyProcessor>::Context, amount: u32) -> u32;
    }

    impl Process for counter::Processor {
        fn process_internal(&mut self, ctx: <Self as ServifyProcessor>::Context, amount: u32) -> u32 {
            self.count += amount;
            self.count
        }
    }

    impl counter::Processor {
        pub fn increment_and_get(&mut self, amount: u32) -> u32 {
            <Self as Process>::process_internal(self, counter::Context::Direct, amount)
        }
    }

    impl counter::ImplIncrementAndGet for counter::Processor {
        fn process_any(&mut self, ctx: counter::Context, payload: Box<dyn Any>) -> Box<dyn Any> {
            let req = *payload.downcast::<Request>().unwrap();
            Box::new(<Self as Process>::process_internal(self, ctx, req.amount))
        }
    }

    impl<T: counter::Dispatchable> counter::Counter<T> {
        pub async fn increment_and_get(&self, amount: u32) -> u32 {
            let req = Request { amount };
            *<Self as counter::Dispatchable>::send(self, counter::Kind::IncrementAndGet, Box::new(req)).await.downcast::<u32>().unwrap()
        }
    }
}