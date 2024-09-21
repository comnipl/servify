#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[allow(non_snake_case)]
    mod SomeStruct {
        use super::{SomeStruct_AddHello, SomeStruct_GetString};

        pub struct Server {
            pub a: String,
        }

        #[derive(Clone)]
        pub struct Client {
            tx: tokio::sync::mpsc::Sender<Message>,
        }

        pub enum Message {
            AddHello(
                SomeStruct_AddHello::Request,
                tokio::sync::oneshot::Sender<SomeStruct_AddHello::Response>,
            ),
            GetString(
                SomeStruct_GetString::Request,
                tokio::sync::oneshot::Sender<SomeStruct_GetString::Response>,
            ),
        }

        pub fn initiate_message_passing() -> (tokio::sync::mpsc::Receiver<Message>, Client) {
            let (tx, rx) = tokio::sync::mpsc::channel(64);
            let client = Client { tx };
            (rx, client)
        }

        impl Server {
            pub async fn listen(&mut self, mut rx: tokio::sync::mpsc::Receiver<Message>) {
                while let Some(msg) = rx.recv().await {
                    match msg {
                        Message::AddHello(req, tx) => {
                            let res = self.add_hello(req).await;
                            tx.send(res).unwrap();
                        }
                        Message::GetString(req, tx) => {
                            let res = self.get_string(req).await;
                            tx.send(res).unwrap();
                        }
                    }
                }
            }
        }

        impl SomeStruct_AddHello::SendAddHello for Client {
            async fn __internal_add_hello(
                &self,
                req: SomeStruct_AddHello::Request,
            ) -> SomeStruct_AddHello::Response {
                let (tx, rx) = tokio::sync::oneshot::channel();
                self.tx.send(Message::AddHello(req, tx)).await.unwrap();
                rx.await.unwrap()
            }
        }

        impl SomeStruct_GetString::SendGetString for Client {
            async fn __internal_get_string(&self) -> SomeStruct_GetString::Response {
                let (tx, rx) = tokio::sync::oneshot::channel();
                self.tx
                    .send(Message::GetString(SomeStruct_GetString::Request {}, tx))
                    .await
                    .unwrap();
                rx.await.unwrap()
            }
        }
    }

    #[allow(non_snake_case)]
    mod SomeStruct_AddHello {
        use super::SomeStruct;

        #[derive(Clone)]
        pub struct Request {
            n: usize,
        }
        pub type Response = String;

        impl SomeStruct::Server {
            pub async fn add_hello(&mut self, req: Request) -> Response {
                self.__internal_add_hello(req.n).await
            }

            async fn __internal_add_hello(&mut self, n: usize) -> Response {
                self.a.push_str(&"Hello".repeat(n));
                self.a.clone()
            }
        }

        pub trait SendAddHello {
            async fn __internal_add_hello(&self, req: Request) -> Response;

            async fn add_hello(&self, n: usize) -> Response {
                self.__internal_add_hello(Request { n }).await
            }
        }
    }

    #[allow(non_snake_case)]
    mod SomeStruct_GetString {
        use super::SomeStruct;

        #[derive(Clone)]
        pub struct Request {}
        pub type Response = String;

        impl SomeStruct::Server {
            pub async fn get_string(&mut self, _req: Request) -> Response {
                self.__internal_get_string().await
            }

            async fn __internal_get_string(&mut self) -> Response {
                self.a.clone()
            }
        }

        pub trait SendGetString {
            async fn __internal_get_string(&self) -> Response;

            async fn get_string(&self) -> Response {
                self.__internal_get_string().await
            }
        }
    }

    #[tokio::test]
    async fn test_manual_expanded() {
        use crate::tests::SomeStruct_AddHello::SendAddHello;
        use crate::tests::SomeStruct_GetString::SendGetString;

        let (rx, client) = SomeStruct::initiate_message_passing();

        tokio::spawn(async move {
            SomeStruct::Server {
                a: String::from("Servify, "),
            }
            .listen(rx)
            .await;
        });

        assert_eq!(client.get_string().await, "Servify, ");
        assert_eq!(client.add_hello(3).await, "Servify, HelloHelloHello");
        assert_eq!(client.get_string().await, "Servify, HelloHelloHello");
        assert_eq!(
            client.add_hello(2).await,
            "Servify, HelloHelloHelloHelloHello"
        );
        assert_eq!(
            client.get_string().await,
            "Servify, HelloHelloHelloHelloHello"
        );
    }
}
