use crate::Diagnostic;
use qrpc_core_ir::v4::MachineMemoryField;

use super::{diag, parse_qs_type_ref, split_words};

pub(super) fn parse_memory(
    input: &str,
    line_number: usize,
) -> Result<MachineMemoryField, Diagnostic> {
    let Some((name, rest)) = input.split_once(':') else {
        return Err(diag(
            "QSV4113",
            "memory 语法必须是 `memory <name>: <type> [nullable]`",
            line_number,
        ));
    };
    let parts = split_words(rest);
    let Some(type_name) = parts.first() else {
        return Err(diag("QSV4114", "memory 必须声明类型", line_number));
    };
    let type_ref = parse_qs_type_ref(type_name).map_err(|message| {
        diag(
            "QSV4117",
            format!("memory 类型不在 v4 QS 类型系统中: {message}"),
            line_number,
        )
    })?;
    Ok(MachineMemoryField {
        name: name.trim().to_string(),
        type_name: (*type_name).to_string(),
        type_ref: Some(type_ref),
        default_value: None,
        nullable: parts.contains(&"nullable"),
    })
}
