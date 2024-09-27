mod get;
mod increment;
mod reset;
mod set;

use get::simple_counter_get;
use increment::simple_counter_increment_and_get;
use reset::simple_counter_reset;
use set::simple_counter_set;

#[servify_macro::service(
    impls = [
        simple_counter_increment_and_get,
        simple_counter_get,
        simple_counter_reset,
        simple_counter_set
    ]
)]
struct SimpleCounter {
    pub counter: u32,
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
