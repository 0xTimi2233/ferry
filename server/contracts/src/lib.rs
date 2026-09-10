//! 生成契约
//!
//! 管理面的入站请求、响应与视图以 `contracts/proto/` 为单一真源，本 crate 承载生成结果；
//! 协议面的入参出参由 `domain/canonical.rs` 承载，见 ADR 0001。
//! 切片引用这里的类型，改动管理面形状必须从 proto 改起；JSON 编解码由 pbjson 按 proto3 JSON
//! 映射生成，切片不得自行定义响应结构。

pub mod gateway {
    /// prost 与 pbjson 的产物，不受仓库静态检查约束
    #[allow(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/gateway.v1.rs"));
        include!(concat!(env!("OUT_DIR"), "/gateway.v1.serde.rs"));
    }
}
