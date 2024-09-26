#[servify_macro::service(
    impls = [
        counter_increment_and_get,
        counter_get_value,
    ]
)]
struct Counter {
    pub count: u32
}

#[servify_macro::export]
impl Counter {
    fn increment_and_get(&mut self, count: u32) -> u32 {
        self.count += count;
        self.count
    }
    fn get_value(&self) -> u32 {
        self.count
    }
}

#[tokio::test]
async fn count_up() {
    let (rx, client) = Counter::initiate_message_passing(32);
    
    tokio::spawn(async move {
        Counter::Server { count: 3 }.listen(rx).await;
    });
    
    assert_eq!(client.get_value().await, 3);
    assert_eq!(client.increment_and_get(5).await, 8);
    assert_eq!(client.get_value().await, 8);
    assert_eq!(client.increment_and_get(3).await, 11);
    assert_eq!(client.get_value().await, 11);
}
