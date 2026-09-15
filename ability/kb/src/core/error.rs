use thiserror::Error;

#[derive(Debug, Error)]
pub enum KbError {
    #[error("非法标识: {0}")]
    InvalidId(String),

    #[error("知识库不存在: {0}")]
    KbNotFound(String),

    #[error("知识库已存在: {0}")]
    KbExists(String),

    #[error("作用域不存在: {0}")]
    ScopeNotFound(String),

    #[error("MIU 不存在: {0}")]
    MiuNotFound(String),

    #[error("能力不可用: {0}")]
    CapabilityUnavailable(String),

    #[error("存储失败: {0}")]
    Store(String),

    #[error("门控拒绝: {0}")]
    GateRejected(String),

    #[error("参数无效: {0}")]
    InvalidInput(String),
}
