//! 选定凭证

/// 入站数据
#[derive(Debug, Clone)]
pub struct SelectCredential {
    pub alias: String,
    pub session_id: Option<String>,
}

/// 出站数据
#[derive(Debug, Clone)]
pub struct SelectedCredential {
    pub credential_id: String,
    pub affinity_hit: bool,
}
