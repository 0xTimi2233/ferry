//! 完成订阅授权

/// 入站数据
#[derive(Debug, Clone)]
pub struct CompleteSubscriptionAuthorization {
    pub state: String,
    pub code: String,
}
