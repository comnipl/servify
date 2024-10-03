#!/usr/bin/env -S cargo +nightly -Zscript run --release --manifest-path
---
[package]
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.40.0", features = ["macros", "rt", "process", "fs", "io-util"] }
futures = "0.3.30"
serde_json = "1.0.128"
toml = "0.8.19"
---

use std::sync::LazyLock;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use serde_json::Value;
use futures::{StreamExt as _};

static PACKAGES: LazyLock<Vec<Package>> = LazyLock::new(|| {
    vec![
        Package {
            name: "servify".to_string(),
            pnpm_file: PathBuf::from("servify/package.json"),
            cargo_file: PathBuf::from("servify/Cargo.toml"),
        },
        Package {
            name: "servify_macro".to_string(),
            pnpm_file: PathBuf::from("servify_macro/package.json"),
            cargo_file: PathBuf::from("servify_macro/Cargo.toml"),
        },
    ]
});

struct Package {
    name: String,
    pnpm_file: PathBuf,
    cargo_file: PathBuf,
}

impl Package {

    async fn get_pnpm_version(&self) -> Option<String> {
        let mut file = File::open(&self.pnpm_file).await.ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await.ok()?;

        let package_json: Value = serde_json::from_str(&contents).ok()?;
        package_json.get("version").and_then(|v| v.as_str()).map(String::from)
    }

    async fn get_cargo_version(&self) -> Option<String> {
        let mut file = File::open(&self.cargo_file).await.ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await.ok()?;

        let cargo_toml: Value = toml::from_str(&contents).ok()?;
        cargo_toml.get("package").and_then(|p| p.get("version")).and_then(|v| v.as_str()).map(String::from)
    }

    async fn update_version(&self) {

        println!(
            "Version ({}): pnpm = {}, cargo = {}",
            self.name,
            self.get_pnpm_version().await.unwrap_or("N/A".to_string()),
            self.get_cargo_version().await.unwrap_or("N/A".to_string()),
        );
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let status = Command::new("pnpm")
            .arg("changeset")
            .arg("version")
            .status()
            .await
            .expect("Failed to execute pnpm changeset version");

    if status.success() {
        println!("Changeset version executed successfully with exit code 0");

        futures::stream::iter((*PACKAGES).iter())
            .map(|package| tokio::spawn(async move {
                package.update_version().await;
            }))
            .buffer_unordered(4)
            .collect::<Vec<_>>()
            .await;

    } else {
        println!("Changeset version failed with exit code: {}", status.code().unwrap_or(-1));
    }

}
