#!/usr/bin/env -S cargo +nightly -Zscript run --release --manifest-path
---
[package]
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.40.0", features = ["macros", "rt", "process", "fs", "io-util"] }
futures = "0.3.30"
serde_json = "1.0.128"
toml_edit = "0.22.22"
anyhow = "1.0.89"
---

use std::sync::LazyLock;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use serde_json::Value;
use futures::{StreamExt as _};
use toml_edit::{DocumentMut, value};

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

async fn read_file(path: &PathBuf) -> Result<String, anyhow::Error> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

impl Package {

    async fn get_pnpm_version(&self) -> Result<String, anyhow::Error> {
        let contents = read_file(&self.pnpm_file).await?;
        let package_json: Value = serde_json::from_str(&contents)?;
        package_json
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("pnpm version not found in {}", self.name))
    }
    async fn update_version(&self) -> Result<(), anyhow::Error> {

        let pnpm_version = self.get_pnpm_version().await?;

        let mut cargo_toml = read_file(&self.cargo_file).await?.parse::<DocumentMut>()?;
        let cargo_version = cargo_toml["package"]["version"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("cargo version not found in {}", self.name))?;

        if pnpm_version != cargo_version {
            cargo_toml["package"]["version"] = value(pnpm_version.clone());
            let new_cargo_toml = cargo_toml.to_string();
            tokio::fs::write(&self.cargo_file, new_cargo_toml).await?;
            println!("[{}] Updated {} --> {}", self.name, cargo_version, pnpm_version);
        } else {
            println!("[{}] already up to date", self.name);
        }

        Ok(())
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
                package.update_version().await
            }))
            .buffer_unordered(4)
            .collect::<Vec<_>>()
            .await;

    } else {
        println!("Changeset version failed with exit code: {}", status.code().unwrap_or(-1));
    }

}
