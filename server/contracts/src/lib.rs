//! 生成契约
//!
//! 管理面的入站请求、响应与视图以 `contracts/proto/` 为单一真源，本 crate 承载生成结果；
//! 协议面的入参出参由 `domain/canonical.rs` 承载，见 ADR 0001。
//! 切片引用这里的类型，改动管理面形状必须从 proto 改起；JSON 编解码由 pbjson 按 proto3 JSON
//! 映射生成，切片不得自行定义响应结构。
//!
//! 模块路径与契约包名一一对应，包名由 `build.rs` 扫描契约得到，本文件不写死包名，
//! 新增契约包时无需改动这里。

include!(concat!(env!("OUT_DIR"), "/packages.rs"));

#[cfg(test)]
mod tests {
    use super::gateway::v1::{HealthState, HealthStatus};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    /// 时间字段按 proto3 JSON 映射编成 RFC 3339 字符串，证明已知类型与 pbjson 可以共存
    #[test]
    fn should_encode_timestamp_field_as_rfc3339() -> TestResult {
        let status = HealthStatus {
            state: HealthState::Cooling as i32,
            reason: "429 速率限制".to_owned(),
            recover_at: Some(pbjson_types::Timestamp {
                seconds: 1_760_000_000,
                nanos: 0,
            }),
        };

        let json = serde_json::to_value(&status)?;

        // pbjson-types 以 RFC 3339 渲染 UTC 时刻，偏移写作 +00:00，与 Z 等价
        assert_eq!(
            json.get("recoverAt"),
            Some(&serde_json::json!("2025-10-09T08:53:20+00:00"))
        );
        assert_eq!(
            json.get("state"),
            Some(&serde_json::json!("HEALTH_STATE_COOLING"))
        );

        let parsed: HealthStatus = serde_json::from_value(json)?;
        assert_eq!(parsed, status);
        Ok(())
    }
}
