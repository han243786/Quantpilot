// ── 统一错误类型 (v2.2.0) ──
// 替代 anyhow::Error, 提供类型化错误以便调用者 match 不同错误变体

use std::fmt;

/// QuantPilot 核心错误类型
#[derive(Debug)]
pub enum QuantPilotError {
    /// 验证失败 (输入不合法)
    Validation(String),
    /// 运行时错误 (执行过程中异常)
    Runtime(String),
    /// IO 错误
    Io(std::io::Error),
    /// 序列化/反序列化错误
    Serialization(String),
    /// 插件相关错误
    Plugin(String),
    /// 能力合约相关错误
    Capability(String),
    /// 内部错误 (不应暴露给用户)
    Internal(String),
}

impl fmt::Display for QuantPilotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(msg) => write!(f, "验证失败: {}", msg),
            Self::Runtime(msg) => write!(f, "运行时错误: {}", msg),
            Self::Io(e) => write!(f, "IO错误: {}", e),
            Self::Serialization(msg) => write!(f, "序列化错误: {}", msg),
            Self::Plugin(msg) => write!(f, "插件错误: {}", msg),
            Self::Capability(msg) => write!(f, "能力错误: {}", msg),
            Self::Internal(msg) => write!(f, "内部错误: {}", msg),
        }
    }
}

impl std::error::Error for QuantPilotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for QuantPilotError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
