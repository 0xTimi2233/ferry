//! 从 proto 契约生成 Rust 类型与 JSON 编解码
//!
//! 生成物输出到 `OUT_DIR`，不入库，因此不存在生成物漂移。Rust 侧不依赖 `protoc` 二进制。
//! JSON 编解码由 pbjson 按 proto3 JSON 映射生成，字段命名与枚举取值口径因此与契约一致，
//! 切片不得自行定义响应结构。

use std::error::Error;
use std::path::{Path, PathBuf};

use protox::prost::Message;

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contracts/proto");
    let third_party = root.join("third_party");
    let packages = packages(&root)?;

    let mut names = Vec::new();
    for package in &packages {
        for path in proto_files(&root.join(package))? {
            names.push(relative(&root, &path)?);
        }
    }
    names.sort();
    assert!(!names.is_empty(), "contracts/proto 下没有 proto 文件");

    println!("cargo:rerun-if-changed={}", root.display());

    let descriptors = protox::compile(&names, [&root, &third_party])?;
    let encoded = descriptors.encode_to_vec();

    let mut config = prost_build::Config::new();
    config.compile_fds(descriptors)?;

    let mut builder = pbjson_build::Builder::new();
    builder.register_descriptors(&encoded)?;
    builder.build(&[".gateway.v1"])?;
    Ok(())
}

/// 枚举 contracts/proto 下形如 `<context>/v1` 的目录
fn packages(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut found = Vec::new();
    for context in std::fs::read_dir(root)? {
        let context = context?.path();
        let v1 = context.join("v1");
        if v1.is_dir() {
            let name = context
                .file_name()
                .ok_or("契约目录名不可为空")?
                .to_string_lossy()
                .to_string();
            found.push(PathBuf::from(name).join("v1"));
        }
    }
    found.sort();
    assert!(!found.is_empty(), "contracts/proto 下没有 v1 契约目录");
    Ok(found)
}

fn proto_files(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "proto") {
            found.push(path);
        }
    }
    found.sort();
    assert!(!found.is_empty(), "{} 下没有 proto 文件", dir.display());
    Ok(found)
}

fn relative(root: &Path, path: &Path) -> Result<String, Box<dyn Error>> {
    let relative = path.strip_prefix(root)?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}
