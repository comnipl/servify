#!/usr/bin/env -S cargo +nightly -Zscript run --release --manifest-path
---
[package]
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.40.0", features = ["macros", "rt", "process", "fs", "io-util"] }
futures = "0.3.30"
toml_edit = "0.22.22"
anyhow = "1.0.89"
---

use toml_edit::{DocumentMut, Item, Value, Formatted};
use tokio::fs::File;
use std::path::PathBuf;
use std::sync::LazyLock;
use futures::{StreamExt as _};
use tokio::io::{AsyncReadExt as _};

static PACKAGES: LazyLock<Vec<Package>> = LazyLock::new(|| {
    vec![
        Package {
            name: "servify".to_string(),
            cargo_file: PathBuf::from("servify/Cargo.toml"),
        },
        Package {
            name: "servify_macro".to_string(),
            cargo_file: PathBuf::from("servify_macro/Cargo.toml"),
        },
    ]
});

#[derive(Clone)]
struct Package {
    name: String,
    cargo_file: PathBuf,
}

#[derive(Clone)]
struct PackageParsed {
    package: Package,
    document: DocumentMut,
    version: String,
}

impl Package {
    async fn parse(self) -> Result<PackageParsed, anyhow::Error> {
        let contents = read_file(&self.cargo_file).await?;
        let document = contents.parse::<DocumentMut>()?;
        let version = document["package"]["version"].as_str().map(String::from).ok_or(anyhow::anyhow!("version not found"))?;
        Ok(PackageParsed { package: self, document, version })
    }
}

async fn read_file(path: &PathBuf) -> Result<String, anyhow::Error> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

impl PackageParsed {
    async fn solve_path_deps(&self, packages: &Vec<PackageParsed>) -> Result<Vec<usize>, anyhow::Error> {
        let deps_ids = vec![];
        let Some(deps) = self.document["dependencies"].as_table_mut() else {
            return Ok(deps_ids);
        };
        deps.iter_mut().for_each(|(key, v)| {
            if !v.as_table().map(|t| t.contains_key("path")).unwrap_or(false) {
                return;
            }
            let deps_pkg_name = key.get();
            let deps_pkg = packages.iter().position(|package| package.package.name == deps_pkg_name)
                .ok_or_else(|| anyhow::anyhow!("crate {} is specified with path, but it is not included in packages.", deps_pkg_name))
                .unwrap();
            deps_ids.push(deps_pkg);

            let deps_pkg_version = packages[deps_pkg].version.clone();
            v.as_table_mut().unwrap().iter_mut().map(|(key, v)| {
                if key == "path" {
                    println!("[{}] {}: {}", self.package.name, deps_pkg_name, deps_pkg_version);
                    *key = String::from("version").into();
                    *v = Item::Value(Value::String(Formatted::new(deps_pkg_version)));
                }
            });
        });
        tokio::fs::write(&self.package.cargo_file, self.document.to_string()).await?;
        Ok(deps_ids)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let packages = futures::stream::iter((*PACKAGES).iter())
        .map(|package| tokio::spawn(async move {
            package.clone().parse().await
        }))
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await
        .unwrap()
        .into_iter()
        .collect::<Result<Vec<PackageParsed>, _>>()
        .unwrap();

    let deps_solves = futures::stream::iter(packages.iter())
        .map(|package| tokio::spawn(async move {
            package.solve_path_deps(&packages).await
        }))
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await
        .unwrap()
        .iter()
        .collect::<Result<Vec<Vec<usize>>, _>>()
        .unwrap();
}
