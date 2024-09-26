#[servify_macro::service(
    impls = [
        counter_increment
    ]
)]
struct Counter {
    pub count: u32
}

#[servify_macro::export]
impl Counter {
    fn increment(&mut self, count: u32) -> u32 {
        self.count += count;
        self.count
    }
}

#[tokio::test]
async fn count_up() {
    let (rx, client) = Counter::initiate_message_passing(32);
    
    tokio::spawn(async move {
        Counter::Server { count: 3 }.listen(rx).await;
    });

    assert_eq!(client.increment(5).await, 8);
    assert_eq!(client.increment(3).await, 11);
}
