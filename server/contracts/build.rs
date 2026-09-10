//! 从 proto 契约生成 Rust 类型
//!
//! 生成物输出到 `OUT_DIR`，不入库，因此不存在生成物漂移。Rust 侧不依赖 `protoc` 二进制。

use std::error::Error;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contracts/proto");
    let third_party = root.join("third_party");
    let gateway = root.join("gateway").join("v1");

    let mut names = Vec::new();
    for entry in std::fs::read_dir(&gateway)? {
        let path = entry?.path();
        if is_proto(&path) {
            names.push(relative(&root, &path)?);
        }
    }
    names.sort();
    assert!(
        !names.is_empty(),
        "contracts/proto/gateway/v1 下没有 proto 文件"
    );

    println!("cargo:rerun-if-changed={}", gateway.display());
    println!("cargo:rerun-if-changed={}", third_party.display());

    let descriptors = protox::compile(&names, [&root, &third_party])?;

    let mut config = prost_build::Config::new();
    config.compile_fds(descriptors)?;
    Ok(())
}

fn is_proto(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "proto")
}

fn relative(root: &Path, path: &Path) -> Result<String, Box<dyn Error>> {
    let relative = path.strip_prefix(root)?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}
