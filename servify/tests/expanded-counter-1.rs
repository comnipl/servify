use servify::processor::ServifyProcessor as _;
use tokio::task::JoinSet;

mod counter {
    use servify::processor::ServifyProcessor;
    use servify::serial::{ServifyMiddle, ServifyRequest};
    use std::{any::Any, future::Future, pin::Pin};

    pub(super) struct Processor {
        pub(super) count: u32,
    }

    impl ServifyProcessor for Processor {
        type Context = Context;
        type Kind = Kind;
        async fn serve(
            &mut self,
            middle: ServifyMiddle<Self::Kind, Self::Context>,
        ) -> Box<dyn Any + Send> {
            match middle.request.kind {
                Kind::IncrementAndGet => {
                    <Self as ImplIncrementAndGet>::process_any(
                        self,
                        middle.ctx,
                        middle.request.payload,
                    )
                    .await
                }
                Kind::Get => {
                    <Self as ImplGet>::process_any(self, middle.ctx, middle.request.payload).await
                }
            }
        }
    }

    // Dispatchable -
    //   このモジュールを、「あたかも構造体に属するメソッドを呼びだせるかのように」呼びだせるトレイト。
    //   このトレイトを実装すると、Counter<T> が勝手に各メソッドを実装し、呼びだされた時にsendメソッドを呼びだしてくれる。
    pub trait Dispatchable {
        fn send(
            &self,
            request: ServifyRequest<Kind>,
        ) -> Pin<Box<dyn '_ + Send + Future<Output = Box<dyn Any + Send>>>>;
    }

    #[derive(Clone)]
    pub struct Counter<T: Dispatchable>(T);

    impl<T: Dispatchable> Dispatchable for Counter<T> {
        fn send(
            &self,
            request: ServifyRequest<Kind>,
        ) -> Pin<Box<dyn '_ + Send + Future<Output = Box<dyn Any + Send>>>> {
            self.0.send(request)
        }
    }

    // Kind -
    //   このモジュールで扱うリクエストの種類を表す列挙型。
    pub enum Kind {
        IncrementAndGet,
        Get,
    }

    // Context -
    //  このモジュールで扱うリクエストが、呼ばれたクライアント(正確にはどのサーバーが受けとったか)によって
    //  付け加わるコンテキスト情報。
    //
    //  Directは、Processor構造体から直接呼び出された場合。
    pub(super) enum Context {
        Direct,
        MessagePassing,
    }

    // Impl(メソッド名) -
    //   実際に実装を記述している別の場所に対して、定数や関数を要求するトレイト。
    pub(super) trait ImplIncrementAndGet {
        async fn process_any(
            &mut self,
            ctx: Context,
            payload: Box<dyn Any + Send>,
        ) -> Box<dyn Any + Send>;
    }

    pub(super) trait ImplGet {
        async fn process_any(
            &mut self,
            ctx: Context,
            payload: Box<dyn Any + Send>,
        ) -> Box<dyn Any + Send>;
    }


    // Modules -
    //   メッセージパッシングやHTTPサーバーなど、異なるリクエストの種類に対して、それぞれの処理を記述するモジュール。
    pub mod message_passing {
        use std::{any::Any, future::Future, pin::Pin};

        use servify::{processor::ServifyAccess, serial::ServifyRequest};

        use super::{Context, Counter, Dispatchable, Kind};

        #[derive(Clone)]
        pub struct MessagePassing {
            access: ServifyAccess<super::Processor>,
        }

        pub fn initiate(access: ServifyAccess<super::Processor>) -> Counter<MessagePassing> {
            Counter(MessagePassing { access })
        }

        impl Dispatchable for MessagePassing {
            fn send(
                &self,
                request: ServifyRequest<Kind>,
            ) -> Pin<Box<dyn '_ + Send + Future<Output = Box<dyn Any + Send>>>> {
                Box::pin(async move {
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    self.access
                        .tx
                        .send((request.with_ctx(Context::MessagePassing), tx))
                        .await
                        .unwrap();
                    rx.await.unwrap()
                })
            }
        }
    }
}

mod counter_increment_and_get {
    use std::any::Any;

    use super::counter;
    use servify::{processor::ServifyProcessor, serial::ServifyRequest};

    struct Request {
        amount: u32,
    }

    trait Process {
        async fn process_internal(
            &mut self,
            ctx: <counter::Processor as ServifyProcessor>::Context,
            amount: u32,
        ) -> u32;
    }

    impl Process for counter::Processor {
        async fn process_internal(
            &mut self,
            ctx: <Self as ServifyProcessor>::Context,
            amount: u32,
        ) -> u32 {
            match ctx {
                counter::Context::MessagePassing => {}
                _ => unimplemented!(),
            }
            self.count += amount;
            self.get().await
        }
    }

    impl counter::Processor {
        #[allow(unused)]
        #[inline(always)]
        pub async fn increment_and_get(&mut self, amount: u32) -> u32 {
            <Self as Process>::process_internal(self, counter::Context::Direct, amount).await
        }
    }

    impl counter::ImplIncrementAndGet for counter::Processor {
        #[inline(always)]
        async fn process_any(
            &mut self,
            ctx: counter::Context,
            payload: Box<dyn Any + Send>,
        ) -> Box<dyn Any + Send> {
            let req = *payload.downcast::<Request>().unwrap();
            Box::new(<Self as Process>::process_internal(self, ctx, req.amount).await)
        }
    }

    impl<T: counter::Dispatchable> counter::Counter<T> {
        #[inline(always)]
        pub async fn increment_and_get(&self, amount: u32) -> u32 {
            let req = Request { amount };
            let request = ServifyRequest {
                kind: counter::Kind::IncrementAndGet,
                payload: Box::new(req),
            };
            *<Self as counter::Dispatchable>::send(self, request)
                .await
                .downcast::<u32>()
                .unwrap()
        }
    }
}

mod get {
    use std::any::Any;

    use super::counter;
    use servify::serial::ServifyRequest;

    struct Request {}

    trait Process {
        async fn process_internal(&mut self) -> u32;
    }

    impl Process for counter::Processor {
        async fn process_internal(&mut self) -> u32 {
            self.count
        }
    }

    impl counter::Processor {
        #[allow(unused)]
        #[inline(always)]
        pub async fn get(&mut self) -> u32 {
            <Self as Process>::process_internal(self).await
        }
    }

    impl counter::ImplGet for counter::Processor {
        #[inline(always)]
        async fn process_any(
            &mut self,
            _ctx: counter::Context,
            payload: Box<dyn Any + Send>,
        ) -> Box<dyn Any + Send> {
            let _req = *payload.downcast::<Request>().unwrap();
            Box::new(<Self as Process>::process_internal(self).await)
        }
    }

    impl<T: counter::Dispatchable> counter::Counter<T> {
        #[inline(always)]
        pub async fn get(&self) -> u32 {
            let req = Request {};
            let request = ServifyRequest {
                kind: counter::Kind::Get,
                payload: Box::new(req),
            };
            *<Self as counter::Dispatchable>::send(self, request)
                .await
                .downcast::<u32>()
                .unwrap()
        }
    }
}
#[tokio::test]
async fn main() {
    let (counter_server, counter_access) = counter::Processor { count: 0 }.channel(32);
    tokio::spawn(async move {
        counter_server.listen().await;
    });

    let counter = counter::message_passing::initiate(counter_access);

    let mut set = JoinSet::new();

    for _ in 0..10 {
        let counter = counter.clone();
        set.spawn(async move {
            for _ in 0..1000 {
                counter.increment_and_get(1).await;
            }
        });
    }
    set.join_all().await;

    assert_eq!(counter.get().await, 10000);
}
