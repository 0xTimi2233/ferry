//! 生成契约
//!
//! 入站数据类型与对外响应形状的单一真源是 `contracts/proto/`，本 crate 只承载生成结果。
//! 切片引用这里的类型，改动形状必须从 proto 改起。

pub mod gateway {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/gateway.v1.rs"));
    }
}
