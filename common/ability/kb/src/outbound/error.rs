use thiserror::Error;

#[derive(Debug, Error)]
pub enum OutboundError {
    #[error("订阅不存在: {0}")]
    NotFound(String),

    #[error("投递失败: {0}")]
    Deliver(String),

    #[error("参数无效: {0}")]
    Invalid(String),
}
