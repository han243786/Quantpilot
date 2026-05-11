use std::sync::Mutex;

static EXTRA_PATTERNS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// 从 CredentialVault 注册凭证字段名到脱敏模块
pub fn register_credential_patterns(patterns: Vec<String>) {
    if let Ok(mut guard) = EXTRA_PATTERNS.lock() {
        *guard = patterns;
    }
}

pub fn sanitize_secrets(input: &str) -> String {
    let builtin = [
        "api_key", "secret", "passphrase", "password", "apikey", "api_secret",
    ];

    let extra = EXTRA_PATTERNS.lock().unwrap_or_else(|e| e.into_inner());
    let all_patterns: Vec<String> = builtin
        .iter()
        .map(|s| s.to_string())
        .chain(extra.iter().cloned())
        .collect();

    let mut result = input.to_string();
    for pattern in &all_patterns {
        loop {
            let lower = result.to_lowercase();
            match lower.find(pattern.as_str()) {
                Some(pos) => {
                    if let Some(sep_offset) = result[pos..].find(|c: char| c == ':' || c == '=') {
                        let after_sep = pos + sep_offset + 1;
                        let trimmed = result[after_sep..].trim_start();
                        let leading_ws = result[after_sep..].len() - trimmed.len();
                        let value_start = after_sep + leading_ws;
                        let rest = &result[value_start..];
                        let end = if rest.starts_with('"') {
                            rest[1..].find('"').map(|i| i + 2)
                        } else {
                            rest.find(|c: char| {
                                c == ',' || c == ' ' || c == '\n' || c == '}' || c == ']' || c == ')'
                            })
                        };
                        if let Some(end) = end {
                            let before = &result[..value_start];
                            let after = &result[value_start + end..];
                            result = format!("{}***{}", before, after);
                        } else {
                            break;
                        }
                    } else {
                        break;
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
    fn sanitize_secrets_preserves_safe_content() {
        let input = r#"{"instrument": "BTCUSDT", "quantity": 1.0}"#;
        let result = sanitize_secrets(input);
        assert_eq!(result, input, "无敏感字段时应原样保留");
    }
}
