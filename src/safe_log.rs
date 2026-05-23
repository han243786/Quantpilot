use std::sync::RwLock;
use zeroize::Zeroizing;

// v2.1.0: 日志级别配置 (QUANTPILOT_LOG_LEVEL=error|warn|info|debug, 默认info)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

pub fn configured_log_level() -> LogLevel {
    static LEVEL: std::sync::OnceLock<LogLevel> = std::sync::OnceLock::new();
    *LEVEL.get_or_init(|| {
        let env = std::env::var("QUANTPILOT_LOG_LEVEL")
            .unwrap_or_default()
            .to_lowercase();
        match env.as_str() {
            "error" => LogLevel::Error,
            "warn" => LogLevel::Warn,
            "debug" => LogLevel::Debug,
            _ => LogLevel::Info,
        }
    })
}

// v2.1.3: RwLock 替代 Mutex — 读多写少(每行日志读取, 仅凭证变更时写入)
static EXTRA_PATTERNS: RwLock<Vec<Zeroizing<String>>> = RwLock::new(Vec::new());

/// 从 CredentialVault 注册凭证字段名到脱敏模块
pub fn register_credential_patterns(patterns: Vec<Zeroizing<String>>) {
    if let Ok(mut guard) = EXTRA_PATTERNS.write() {
        *guard = patterns;
    }
}

pub fn sanitize_secrets(input: &str) -> String {
    let builtin = [
        "api_key",
        "secret",
        "passphrase",
        "password",
        "apikey",
        "api_secret",
        // v2.0.1: JWT/令牌敏感字段
        "token",
        "jwt",
        "bearer",
        "authorization",
        "private_key",
        "access_key",
        "signing_key",
        "credential",
    ];

    let extra = EXTRA_PATTERNS.read().unwrap_or_else(|e| e.into_inner());
    let extra_strs: Vec<String> = extra.iter().map(|z| z.to_string()).collect();
    let all_patterns: Vec<String> = builtin
        .iter()
        .map(|s| s.to_string())
        .chain(extra_strs.into_iter())
        .collect();

    let mut result = input.to_string();
    // v2.5.0 NOTE: to_lowercase() 用于大小写不敏感匹配, 全 ASCII 模式 (api_key 等)
    // 不会影响 position 索引。如未来添加含 Unicode 的模式需改用 char_indices 遍历。
    for pattern in &all_patterns {
        let mut search_start = 0;
        loop {
            let lower = result.to_lowercase();
            if search_start >= lower.len() {
                break;
            }

            match lower[search_start..].find(pattern.as_str()) {
                Some(offset) => {
                    let pos = search_start + offset;
                    if let Some(sep_offset) = result[pos..].find([':', '=']) {
                        let after_sep = pos + sep_offset + 1;
                        let trimmed = result[after_sep..].trim_start();
                        let leading_ws = result[after_sep..].len() - trimmed.len();
                        let value_start = after_sep + leading_ws;
                        let rest = &result[value_start..];
                        let end = if pattern == "authorization" {
                            rest.find(|c: char| {
                                c == '\n' || c == ',' || c == '}' || c == ']' || c == ')'
                            })
                            .unwrap_or(rest.len())
                        } else if rest.starts_with('"') {
                            rest[1..].find('"').map(|i| i + 2).unwrap_or(rest.len())
                        // 无闭合引号 → 到字符串末尾
                        } else {
                            rest.find(|c: char| {
                                c == ','
                                    || c == ' '
                                    || c == '\n'
                                    || c == '}'
                                    || c == ']'
                                    || c == ')'
                            })
                            .unwrap_or(rest.len()) // 无终止符 → 到字符串末尾
                        };
                        let before = &result[..value_start];
                        let after = &result[value_start + end..];
                        result = format!("{}***{}", before, after);
                        search_start = value_start + 3;
                    } else {
                        search_start = pos + pattern.len();
                    }
                }
                None => break,
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_secrets_masks_api_key_in_json() {
        let input = r#"{"api_key": "abc123xyz", "side": "buy"}"#;
        let result = sanitize_secrets(input);
        assert!(!result.contains("abc123xyz"), "密钥值应被替换: {}", result);
        assert!(result.contains("***"), "应包含掩码: {}", result);
        assert!(result.contains("side"), "非敏感字段应保留");
    }

    #[test]
    fn sanitize_secrets_masks_secret_value() {
        let input = r#"secret="my-secret-value", quantity=1.0"#;
        let result = sanitize_secrets(input);
        assert!(!result.contains("my-secret-value"), "secret 值应被替换");
    }

    #[test]
    fn sanitize_secrets_masks_authorization_without_looping() {
        let input = "[auth] Authorization: Bearer generated-secret-token";
        let result = sanitize_secrets(input);
        assert_eq!(result, "[auth] Authorization: ***");
    }

    #[test]
    fn sanitize_secrets_preserves_safe_content() {
        let input = r#"{"instrument": "BTCUSDT", "quantity": 1.0}"#;
        let result = sanitize_secrets(input);
        assert_eq!(result, input, "无敏感字段时应原样保留");
    }
}
