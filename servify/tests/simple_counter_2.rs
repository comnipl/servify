#[servify_macro::service(
    impls = [
        SimpleCounter_increment_and_get,
        SimpleCounter_get,
        SimpleCounter_set,
        SimpleCounter_reset,
    ]
)]
struct SimpleCounter {
    pub counter: u32,
}

#[servify_macro::export]
impl SimpleCounter {
    fn increment_and_get(&mut self) -> u32 {
        self.counter += 1;
        self.counter
    }

    fn get(&self) -> u32 {
        self.counter
    }

    fn set(&mut self, value: u32) {
        self.counter = value;
    }

    fn reset(&mut self) {
        self.counter = 0;
    }
}

#[tokio::test]
async fn main() {
    let (counter_rx, counter_client) = SimpleCounter::initiate_message_passing(32);

    tokio::spawn(async move {
        SimpleCounter::Server { counter: 0 }
            .listen(counter_rx)
            .await;
    });

    assert_eq!(counter_client.increment_and_get().await, 1);
    assert_eq!(counter_client.increment_and_get().await, 2);
    assert_eq!(counter_client.get().await, 2);
    counter_client.set(10).await;
    assert_eq!(counter_client.get().await, 10);
    counter_client.reset().await;
    assert_eq!(counter_client.get().await, 0);
}
