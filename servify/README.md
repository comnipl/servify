# Servify

A macro for effortlessly enabling message passing, inter-process communication, HTTP/TCP server functionality, and more with a unified implementation in struct methods.

```rs
#[servify::service(
    impls = [
        Counter_increment_and_get,
        Counter_get_value,
    ]
)]
struct Counter {
    pub count: u32,
}

#[servify::export]
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
}

```

## License

Licensed under either of

 - Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE))

 - MIT license
   ([LICENSE-MIT](LICENSE-MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
