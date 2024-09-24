use pretty_assertions::assert_eq;

#[allow(non_snake_case)]
#[allow(unexpected_cfgs)]
mod SomeStruct {

    use super::some_other::some_struct_increment;

    pub struct Server {
        pub count: u32,
    }

    #[derive(Clone)]
    pub struct Client {
        tx: tokio::sync::mpsc::Sender<Message>,
    }

    pub enum Message {
        Increment(
            some_struct_increment::Request,
            tokio::sync::oneshot::Sender<some_struct_increment::Response>,
        ),
    }

    pub fn initiate_message_passing() -> (::tokio::sync::mpsc::Receiver<Message>, Client) {
        let (tx, rx) = ::tokio::sync::mpsc::channel(64);
        let client = Client { tx };
        (rx, client)
    }

    impl Server {
        pub async fn listen(&mut self, mut rx: ::tokio::sync::mpsc::Receiver<Message>) {
            while let Some(msg) = rx.recv().await {
                match msg {
                    Message::Increment(req, tx) => {
                        let res = self.increment(req).await;
                        tx.send(res).unwrap();
                    }
                }
            }
        }
    }

    #[doc(hidden)]
    pub async fn __internal_increment(
        client: &Client,
        req: some_struct_increment::Request,
    ) -> some_struct_increment::Response {
        let (tx, rx) = ::tokio::sync::oneshot::channel();
        client.tx.send(Message::Increment(req, tx)).await.unwrap();
        rx.await.unwrap()
    }
}

mod some_other {
    use crate::tests::expanded_1::SomeStruct;

    #[allow(non_camel_case_types)]
    pub type __increment_response = u32;
    #[allow(non_camel_case_types)]
    #[derive(Clone)]
    pub struct __increment_request {
        count: u32,
    }

    impl SomeStruct::Server {
        pub async fn increment(&mut self, req: __increment_request) -> __increment_response {
            self.__internal_increment(req.count).await
        }

        async fn __internal_increment(&mut self, count: u32) -> __increment_response {
            self.count += count;
            self.count
        }
    }

    impl SomeStruct::Client {
        pub async fn increment(&self, count: u32) -> __increment_response {
            SomeStruct::__internal_increment(self, __increment_request { count }).await
        }
    }

    pub mod some_struct_increment {
        pub use super::{__increment_request as Request, __increment_response as Response};
    }
}

#[tokio::test]
async fn test_manual_expanded() {
    let (rx, client) = SomeStruct::initiate_message_passing();

    tokio::spawn(async move {
        SomeStruct::Server { count: 3 }.listen(rx).await;
    });

    assert_eq!(client.increment(5).await, 8);
    assert_eq!(client.increment(3).await, 11);
}
