// v0.5.2 S0-1: 将 include!() 中 src/ 文件引用的 crate:: 模块暴露到集成测试 crate root。
// 每个集成测试文件在 mod common; 之后 include! 本文件。
pub use common::backend::credential_vault;
pub use common::backend::storage_lifecycle;
pub use common::backend::safe_log;
pub use common::backend::app_runtime_helpers;
