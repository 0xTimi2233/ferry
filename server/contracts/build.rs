//! 从 proto 契约生成 Rust 类型与 JSON 编解码
//!
//! 生成物输出到 `OUT_DIR`，不入库，因此不存在生成物漂移。Rust 侧不依赖 `protoc` 二进制。
//! JSON 编解码由 pbjson 按 proto3 JSON 映射生成，字段命名与枚举取值口径因此与契约一致，
//! 切片不得自行定义响应结构。时间等已知类型由 `pbjson-types` 提供带同样映射的 serde 实现，
//! 契约因此照常用 `google.protobuf` 的已知类型，不必退化成字符串。

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use protox::prost::Message;

/// 生成的模块骨架文件名，`lib.rs` 以 `include!` 引入
const MODULES: &str = "packages.rs";

/// 已知类型的 serde 实现由 `pbjson-types` 提供
const WELL_KNOWN_PREFIX: &str = ".google.protobuf";
const WELL_KNOWN_RUST_PATH: &str = "::pbjson_types";

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contracts/proto");
    let third_party = root.join("third_party");

    let mut names = Vec::new();
    for package in packages(&root)? {
        for path in proto_files(&root.join(package))? {
            names.push(relative(&root, &path)?);
        }
    }
    names.sort();
    assert!(!names.is_empty(), "contracts/proto 下没有 proto 文件");

    println!("cargo:rerun-if-changed={}", root.display());

    let descriptors = protox::compile(&names, [&root, &third_party])?;

    // 包名取 proto 的声明而非目录名：两者漂移时，按目录名选会静默少生成产物
    let packages: BTreeSet<String> = descriptors
        .file
        .iter()
        .filter(|file| names.iter().any(|name| name.as_str() == file.name()))
        .map(|file| {
            let package = file.package();
            assert!(!package.is_empty(), "{} 未声明 package", file.name());
            package.to_string()
        })
        .collect();
    assert!(!packages.is_empty(), "契约里没有可生成的 package");
    let packages: Vec<String> = packages.into_iter().collect();

    let encoded = descriptors.encode_to_vec();

    let mut config = prost_build::Config::new();
    config.compile_well_known_types();
    config.extern_path(WELL_KNOWN_PREFIX, WELL_KNOWN_RUST_PATH);
    config.compile_fds(descriptors)?;

    let selectors: Vec<String> = packages
        .iter()
        .map(|package| format!(".{package}"))
        .collect();
    let mut builder = pbjson_build::Builder::new();
    builder.register_descriptors(&encoded)?;
    builder.extern_path(WELL_KNOWN_PREFIX, WELL_KNOWN_RUST_PATH);
    builder.build(&selectors)?;

    let out = PathBuf::from(std::env::var("OUT_DIR")?).join(MODULES);
    std::fs::write(out, modules(&packages))?;
    Ok(())
}

/// 按包名嵌套的模块骨架：包 `gateway.v1` 生成 `pub mod gateway { pub mod v1 { …… } }`
#[derive(Default)]
struct Node {
    /// 该节点自身是契约包时记下包名，用于引入其生成物
    package: Option<String>,
    children: BTreeMap<String, Node>,
}

fn modules(packages: &[String]) -> String {
    let mut tree = Node::default();
    for package in packages {
        let mut node = &mut tree;
        for segment in package.split('.') {
            node = node.children.entry(segment.to_owned()).or_default();
        }
        node.package = Some(package.clone());
    }

    let mut out = String::from("// 由 build.rs 生成：按契约包名嵌套模块并引入生成物\n");
    render(&tree, 0, &mut out);
    out
}

fn render(node: &Node, depth: usize, out: &mut String) {
    let pad = indent(depth);
    let inner = indent(depth + 1);
    for (name, child) in &node.children {
        if child.package.is_some() {
            let _ = writeln!(
                out,
                "{pad}#[allow(warnings, clippy::all, clippy::pedantic, clippy::nursery)]"
            );
        }
        let _ = writeln!(out, "{pad}pub mod {name} {{");
        if let Some(package) = &child.package {
            let _ = writeln!(
                out,
                "{inner}include!(concat!(env!(\"OUT_DIR\"), \"/{package}.rs\"));"
            );
            let _ = writeln!(
                out,
                "{inner}include!(concat!(env!(\"OUT_DIR\"), \"/{package}.serde.rs\"));"
            );
        }
        render(child, depth + 1, out);
        let _ = writeln!(out, "{pad}}}");
    }
}

fn indent(depth: usize) -> String {
    "    ".repeat(depth)
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
