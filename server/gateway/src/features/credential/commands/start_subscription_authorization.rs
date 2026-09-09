//! 发起订阅授权

/// 入站数据
#[derive(Debug, Clone)]
pub struct StartSubscriptionAuthorization {
    pub provider: String,
}

/// 出站数据
#[derive(Debug, Clone)]
pub struct AuthorizationUrl {
    pub url: String,
}
