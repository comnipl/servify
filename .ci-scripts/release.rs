#!/usr/bin/env -S cargo +nightly -Zscript run --release --manifest-path
---
[package]
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.40.0", features = ["macros", "rt", "process", "fs", "io-util"] }
futures = "0.3.30"
clap = { version = "4.5.19", features = ["derive"] }
toml_edit = "0.22.22"
anyhow = "1.0.89"
petgraph = "0.6.5"
---

use toml_edit::{DocumentMut, Item, Value, Formatted, InlineTable};
use tokio::fs::File;
use std::path::PathBuf;
use std::sync::LazyLock;
use futures::{StreamExt as _};
use tokio::io::{AsyncReadExt as _};
use tokio::process::Command;
use petgraph::{
    algo::{toposort, DfsSpace},
    graph::DiGraph,
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(long)]
    dry_run: bool,
}


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

#[derive(Clone, Debug)]
struct Package {
    name: String,
    cargo_file: PathBuf,
}

#[derive(Clone, Debug)]
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
    async fn solve_path_deps(&mut self, packages: &Vec<PackageParsed>) -> Result<Vec<usize>, anyhow::Error> {
        let mut deps_ids = vec![];
        let Some(deps) = self.document["dependencies"].as_table_mut() else {
            return Ok(deps_ids);
        };
        deps.iter_mut().for_each(|(key, v)| {
            if !v.as_inline_table().map(|t| t.contains_key("path")).unwrap_or(false) {
                return;
            }
            let deps_pkg_name = key.get();
            let deps_pkg = packages.iter().position(|package| package.package.name == deps_pkg_name)
                .ok_or_else(|| anyhow::anyhow!("crate {} is specified with path, but it is not included in packages.", deps_pkg_name))
                .unwrap();
            deps_ids.push(deps_pkg);

            let deps_pkg_version = packages[deps_pkg].version.clone();
            *v = Item::Value(Value::InlineTable(v.as_inline_table().unwrap().iter().map(|(key, v)| {
                if key == "path" {
                    println!("[{}] {}: {}", self.package.name, deps_pkg_name, deps_pkg_version);
                    ("version", Value::String(Formatted::new(deps_pkg_version.clone())))
                } else {
                    (key, v.clone())
                }
            }).collect::<InlineTable>()));
        });
        tokio::fs::write(&self.package.cargo_file, self.document.to_string()).await?;
        Ok(deps_ids)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {

    let args = Args::parse();

    let packages: Vec<PackageParsed> = futures::stream::iter((*PACKAGES).iter())
        .map(|package| tokio::spawn(async move {
            package.clone().parse().await
        }))
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Result<Vec<_>, _>, _>>()
        .unwrap()
        .unwrap();

    let mut deps = packages.clone();

    let mut crates_dig: DiGraph<(), ()> = DiGraph::new();
    let nodes = deps.iter().map(|_| crates_dig.add_node(())).collect::<Vec<_>>();
        
    for (idx, p) in deps.iter_mut().enumerate() {
        let r = p.solve_path_deps(&packages).await.unwrap();
        for d in r {
            crates_dig.add_edge(nodes[d], nodes[idx], ());
        }
    }

    let tagged = Command::new("pnpm")
            .arg("changeset")
            .arg("tag")
            .output()
            .await
            .expect("Failed to execute pnpm changeset tag");
            
    if !tagged.status.success() {
        panic!("Changeset version failed with exit code: {}", tagged.status.code().unwrap_or(-1));
    }

    let output = String::from_utf8(tagged.stdout).unwrap();
    println!("{}", output);

    let publishes = output.lines().map(|s| s.split("tag:")
            .nth(1)
            .unwrap_or("")
            .trim()
            .split('@')
            .next()
            .unwrap_or("")
        ).collect::<Vec<_>>();
    
    let topo = {
        let mut space = DfsSpace::new(&crates_dig);
        toposort(&crates_dig, Some(&mut space))
    }.unwrap().into_iter().map(|node_index| deps[nodes.iter().position(|&n| n == node_index).unwrap()].clone()).collect::<Vec<_>>();

    for publishing in topo {
        let name = publishing.package.name.clone();
        if !publishes.contains(&name.as_str()) {
            println!("[{}] not tagged, Skipping...", name);
            continue;
        }

        println!("[{}] Publishing...", name);
        let mut command = Command::new("cargo");
        command.arg("publish").arg("--allow-dirty");

        if args.dry_run {
            command.arg("--dry-run");
        }
        
        let status = command.current_dir(publishing.package.cargo_file.parent().unwrap())
            .status()
            .await
            .expect("Failed to execute cargo publish");
        
        if !status.success() {
            panic!("[{}] Failed to publish with exit code: {}", name, status.code().unwrap_or(-1));
        }
    }
}
