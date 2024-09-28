mod get;
mod increment;
mod reset;
mod set;

use get::SimpleCounter_get;
use increment::SimpleCounter_increment_and_get_ex;
use reset::SimpleCounter_reset;
use set::SimpleCounter_set;

#[servify_macro::service(
    impls = [
        SimpleCounter_increment_and_get_ex,
        SimpleCounter_get,
        SimpleCounter_set,
        SimpleCounter_reset,
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

    assert_eq!(counter_client.increment_and_get_ex().await, 1);
    assert_eq!(counter_client.increment_and_get_ex().await, 2);
    assert_eq!(counter_client.get().await, 2);
    counter_client.set(10).await;
    assert_eq!(counter_client.get().await, 10);
    counter_client.reset().await;
    assert_eq!(counter_client.get().await, 0);
}
