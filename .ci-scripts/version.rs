#!/usr/bin/env -S cargo +nightly -Zscript run --release --manifest-path
---
[package]
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.40.0", features = ["macros", "rt"] }
---

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Hello, world!");
}
