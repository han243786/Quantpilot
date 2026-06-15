use anyhow::{bail, Result};

use super::OKX_ORDER_PATH;

pub fn okx_order_lookup_path(
    inst_id: &str,
    ord_id: Option<&str>,
    cl_ord_id: Option<&str>,
) -> Result<String> {
    validate_order_lookup(inst_id, ord_id, cl_ord_id)?;
    let mut path = format!("{}?instId={}", OKX_ORDER_PATH, inst_id.trim());
    if let Some(ord_id) = clean_optional_token(ord_id) {
        path.push_str("&ordId=");
        path.push_str(&ord_id);
    }
    if let Some(cl_ord_id) = clean_optional_token(cl_ord_id) {
        path.push_str("&clOrdId=");
        path.push_str(&cl_ord_id);
    }
    Ok(path)
}

pub(super) fn validate_order_lookup(
    inst_id: &str,
    ord_id: Option<&str>,
    cl_ord_id: Option<&str>,
) -> Result<()> {
    let inst_id = inst_id.trim();
    if inst_id.is_empty() || !inst_id.chars().all(valid_okx_inst_char) {
        bail!("OKX instId 不能为空，且只能包含 ASCII 字母、数字和连字符");
    }
    if clean_optional_token(ord_id).is_none() && clean_optional_token(cl_ord_id).is_none() {
        bail!("OKX 查单/撤单必须提供 ordId 或 clOrdId");
    }
    for token in [ord_id, cl_ord_id].into_iter().flatten() {
        let token = token.trim();
        if token.is_empty() || !token.chars().all(valid_okx_id_char) {
            bail!("OKX ordId/clOrdId 只能包含 ASCII 字母、数字、连字符和下划线");
        }
    }
    Ok(())
}

pub(super) fn clean_optional_token(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn valid_okx_inst_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-'
}

fn valid_okx_id_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
}
