// ── API 错误码常量 (v2.3.0) ──
// 语言中立错误码, 前端可映射为 zh-CN/en-US 本地化文本
// 命名规范: ERR_CATEGORY_DESCRIPTION
// v3.5.0: 这些常量是公共API定义, 虽部分未被内部引用, 但作为错误码文档保留
#![allow(dead_code)]

// 通用
pub const ERR_BAD_REQUEST: &str = "BAD_REQUEST";
pub const ERR_INTERNAL: &str = "INTERNAL_ERROR";
pub const ERR_NOT_FOUND: &str = "NOT_FOUND";
pub const ERR_SERVICE_UNAVAILABLE: &str = "SERVICE_UNAVAILABLE";

// 策略图
pub const ERR_GRAPH_ID_EMPTY: &str = "GRAPH_ID_EMPTY";
pub const ERR_GRAPH_ID_TOO_LONG: &str = "GRAPH_ID_TOO_LONG";
pub const ERR_GRAPH_ID_INVALID_CHARS: &str = "GRAPH_ID_INVALID_CHARS";
pub const ERR_GRAPH_NOT_FOUND: &str = "GRAPH_NOT_FOUND";
pub const ERR_GRAPH_SAVE_FAILED: &str = "GRAPH_SAVE_FAILED";
pub const ERR_GRAPH_DELETE_FAILED: &str = "GRAPH_DELETE_FAILED";

// 编译
pub const ERR_STRATEGY_EMPTY_INTENT: &str = "STRATEGY_EMPTY_INTENT";
pub const ERR_COMPILE_CAPABILITY_GATED: &str = "COMPILE_CAPABILITY_GATED";
pub const ERR_COMPILE_FAILED: &str = "COMPILE_FAILED";
pub const ERR_QS_GENERATION_FAILED: &str = "QS_GENERATION_FAILED";
pub const ERR_QS_PARSE_FAILED: &str = "QS_PARSE_FAILED";
pub const ERR_QS_LOWER_FAILED: &str = "QS_LOWER_FAILED";
pub const ERR_COMPILE_BUSY: &str = "COMPILE_BUSY";

// 运行时
pub const ERR_RUN_IN_PROGRESS: &str = "RUN_IN_PROGRESS";
pub const ERR_RUN_NOT_FOUND: &str = "RUN_NOT_FOUND";
pub const ERR_RUNTIME_CAPABILITY: &str = "RUNTIME_CAPABILITY";
pub const ERR_RUNTIME_CONFIG_INVALID: &str = "RUNTIME_CONFIG_INVALID";

// 回测
pub const ERR_BACKTEST_NOT_FOUND: &str = "BACKTEST_NOT_FOUND";
pub const ERR_BACKTEST_COMPARE_TWO_IDS: &str = "BACKTEST_COMPARE_TWO_IDS";

// 认证
pub const ERR_AUTH_UNAUTHORIZED: &str = "AUTH_UNAUTHORIZED";
pub const ERR_AUTH_LOGIN_FAILED: &str = "AUTH_LOGIN_FAILED";
pub const ERR_AUTH_REGISTER_FAILED: &str = "AUTH_REGISTER_FAILED";
pub const ERR_AUTH_TOKEN_EXPIRED: &str = "AUTH_TOKEN_EXPIRED";
pub const ERR_AUTH_RATE_LIMITED: &str = "AUTH_RATE_LIMITED";

// 凭证
pub const ERR_CREDENTIAL_VAULT_UNAVAIL: &str = "CREDENTIAL_VAULT_UNAVAIL";
pub const ERR_CREDENTIAL_SAVE_FAILED: &str = "CREDENTIAL_SAVE_FAILED";
pub const ERR_CREDENTIAL_DELETE_FAILED: &str = "CREDENTIAL_DELETE_FAILED";
pub const ERR_CREDENTIAL_NOT_FOUND: &str = "CREDENTIAL_NOT_FOUND";

// 存储
pub const ERR_STORAGE_FULL: &str = "STORAGE_FULL";
pub const ERR_STORAGE_IO: &str = "STORAGE_IO";

// 插件
pub const ERR_PLUGIN_MANIFEST_INVALID: &str = "PLUGIN_MANIFEST_INVALID";
pub const ERR_PLUGIN_NOT_FOUND: &str = "PLUGIN_NOT_FOUND";
pub const ERR_PLUGIN_VALIDATION: &str = "PLUGIN_VALIDATION";

// 参数
pub const ERR_PARAM_INVALID: &str = "PARAM_INVALID";
pub const ERR_PARAM_PAGINATION: &str = "PARAM_PAGINATION";

// 端口/配置
pub const ERR_PORT_IN_USE: &str = "PORT_IN_USE";
pub const ERR_PORT_RESERVED: &str = "PORT_RESERVED";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_error_codes_are_non_empty() {
        let codes = [
            ERR_BAD_REQUEST, ERR_INTERNAL, ERR_NOT_FOUND, ERR_SERVICE_UNAVAILABLE,
            ERR_GRAPH_ID_EMPTY, ERR_GRAPH_ID_TOO_LONG, ERR_GRAPH_ID_INVALID_CHARS,
            ERR_GRAPH_NOT_FOUND, ERR_GRAPH_SAVE_FAILED, ERR_GRAPH_DELETE_FAILED,
            ERR_STRATEGY_EMPTY_INTENT, ERR_COMPILE_CAPABILITY_GATED, ERR_COMPILE_FAILED,
            ERR_QS_GENERATION_FAILED, ERR_QS_PARSE_FAILED, ERR_QS_LOWER_FAILED, ERR_COMPILE_BUSY,
            ERR_RUN_IN_PROGRESS, ERR_RUN_NOT_FOUND, ERR_RUNTIME_CAPABILITY, ERR_RUNTIME_CONFIG_INVALID,
            ERR_BACKTEST_NOT_FOUND, ERR_BACKTEST_COMPARE_TWO_IDS,
            ERR_AUTH_UNAUTHORIZED, ERR_AUTH_LOGIN_FAILED, ERR_AUTH_REGISTER_FAILED,
            ERR_AUTH_TOKEN_EXPIRED, ERR_AUTH_RATE_LIMITED,
            ERR_CREDENTIAL_VAULT_UNAVAIL, ERR_CREDENTIAL_SAVE_FAILED,
            ERR_CREDENTIAL_DELETE_FAILED, ERR_CREDENTIAL_NOT_FOUND,
            ERR_STORAGE_FULL, ERR_STORAGE_IO,
            ERR_PLUGIN_MANIFEST_INVALID, ERR_PLUGIN_NOT_FOUND, ERR_PLUGIN_VALIDATION,
            ERR_PARAM_INVALID, ERR_PARAM_PAGINATION,
            ERR_PORT_IN_USE, ERR_PORT_RESERVED,
        ];
        for code in &codes {
            assert!(!code.is_empty(), "错误码不能为空");
            assert!(!code.contains(' '), "错误码不能含空格: {code}");
            assert_eq!(code.to_uppercase(), *code, "错误码必须全大写: {code}");
        }
    }

    #[test]
    fn error_codes_have_valid_format() {
        // 所有错误码值为大写字母和下划线组成
        let codes = [ERR_BAD_REQUEST, ERR_COMPILE_FAILED, ERR_AUTH_UNAUTHORIZED];
        for code in &codes {
            assert!(code.chars().all(|c| c.is_ascii_uppercase() || c == '_'),
                "错误码格式无效: {code}");
        }
    }

    #[test]
    fn no_duplicate_error_codes() {
        use std::collections::BTreeSet;
        let codes = [
            ERR_BAD_REQUEST, ERR_INTERNAL, ERR_NOT_FOUND, ERR_SERVICE_UNAVAILABLE,
            ERR_GRAPH_ID_EMPTY, ERR_GRAPH_ID_TOO_LONG, ERR_GRAPH_ID_INVALID_CHARS,
            ERR_GRAPH_NOT_FOUND, ERR_GRAPH_SAVE_FAILED, ERR_GRAPH_DELETE_FAILED,
            ERR_STRATEGY_EMPTY_INTENT, ERR_COMPILE_CAPABILITY_GATED, ERR_COMPILE_FAILED,
            ERR_QS_GENERATION_FAILED, ERR_QS_PARSE_FAILED, ERR_QS_LOWER_FAILED, ERR_COMPILE_BUSY,
            ERR_RUN_IN_PROGRESS, ERR_RUN_NOT_FOUND, ERR_RUNTIME_CAPABILITY, ERR_RUNTIME_CONFIG_INVALID,
            ERR_BACKTEST_NOT_FOUND, ERR_BACKTEST_COMPARE_TWO_IDS,
            ERR_AUTH_UNAUTHORIZED, ERR_AUTH_LOGIN_FAILED, ERR_AUTH_REGISTER_FAILED,
            ERR_AUTH_TOKEN_EXPIRED, ERR_AUTH_RATE_LIMITED,
            ERR_CREDENTIAL_VAULT_UNAVAIL, ERR_CREDENTIAL_SAVE_FAILED,
            ERR_CREDENTIAL_DELETE_FAILED, ERR_CREDENTIAL_NOT_FOUND,
            ERR_STORAGE_FULL, ERR_STORAGE_IO,
            ERR_PLUGIN_MANIFEST_INVALID, ERR_PLUGIN_NOT_FOUND, ERR_PLUGIN_VALIDATION,
            ERR_PARAM_INVALID, ERR_PARAM_PAGINATION,
            ERR_PORT_IN_USE, ERR_PORT_RESERVED,
        ];
        let unique: BTreeSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), codes.len(), "存在重复的错误码定义");
    }
}
