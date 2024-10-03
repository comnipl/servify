#!/usr/bin/env -S cargo +nightly -Zscript run --release --manifest-path
---
[package]
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.40.0", features = ["macros", "rt"] }
---

use std::process::Command;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Hello, world!");

    let status = Command::new("pnpm")
        .arg("changeset")
        .arg("version")
        .status()
        .expect("Failed to execute pnpm changeset version");

    if status.success() {
        println!("Command executed successfully with exit code 0");
    } else {
        println!("Command failed with exit code: {}", status.code().unwrap_or(-1));
    }

}
