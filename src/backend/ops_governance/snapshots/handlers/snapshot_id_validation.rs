pub(super) fn validate_snapshot_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("snapshot_id 不能为空".to_string());
    }
    if id.len() > 128 {
        return Err("snapshot_id 长度不能超过 128 字符".to_string());
    }
    if id.contains("..") || id.contains('/') || id.contains('\\') || id.contains('\0') {
        return Err("snapshot_id 不能包含路径分隔符".to_string());
    }
    if !id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err("snapshot_id 只能使用 ASCII 字母、数字、'_' 或 '-'".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn validate_snapshot_id_rejects_invalid() {
        let cases = ["..", "a/b", "a\\b", "\0x"];
        for case in &cases {
            assert!(
                super::validate_snapshot_id(case).is_err(),
                "ID '{}' 应被拒绝",
                case
            );
        }
        assert!(super::validate_snapshot_id("").is_err());
    }

    #[test]
    fn validate_snapshot_id_accepts_valid() {
        let cases = ["snap-123", "abc_def", "my-snapshot-001"];
        for case in &cases {
            assert!(
                super::validate_snapshot_id(case).is_ok(),
                "ID '{}' 应被接受",
                case
            );
        }
    }
}
