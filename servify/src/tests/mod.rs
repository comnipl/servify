mod expanded_1;



#[servify_macro::export]
impl A {
    fn increment(&mut self, count: u32) -> u32 {
        self.a += count;
        self.a
    }
}

#[servify_macro::service(
    impls = (
        a_increment
    )
)]
struct B {

}

mod A {
    use super::a_increment;

    pub struct Server {
        pub a: u32,
    }
    #[derive(Clone)]
    pub struct Client {
        tx: tokio::sync::mpsc::Sender<Message>,
    }

    pub enum Message {
        Increment(
            a_increment::Request,
            tokio::sync::oneshot::Sender<a_increment::Response>,
        ),
    }

    #[doc(hidden)]
    pub async fn __internal_increment(
        client: &Client,
        req: a_increment::Request,
    ) -> a_increment::Response {
        let (tx, rx) = ::tokio::sync::oneshot::channel();
        client.tx.send(Message::Increment(req, tx)).await.unwrap();
        rx.await.unwrap()
    }

}