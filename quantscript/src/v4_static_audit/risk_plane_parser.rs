use crate::Diagnostic;
use qrpc_core_ir::v4::MachineGraphRiskPlane;

use super::parser_utilities_diagnostics::{diag, split_csv_words, split_words};

pub(super) fn parse_risk_plane(
    input: &str,
    line_number: usize,
) -> Result<MachineGraphRiskPlane, Diagnostic> {
    let parts = split_words(input);
    if parts.is_empty() {
        return Err(diag(
            "QSV4121",
            "risk_plane 必须声明至少一个 machine id",
            line_number,
        ));
    }
    let mut machine_ids = Vec::new();
    let mut min_priority = 9_000;
    let mut cursor = 0;
    while cursor < parts.len() {
        if parts[cursor] == "priority" {
            cursor += 1;
            let Some(value) = parts.get(cursor) else {
                return Err(diag("QSV4122", "risk_plane priority 缺少数值", line_number));
            };
            min_priority = value
                .parse::<i32>()
                .map_err(|_| diag("QSV4123", "risk_plane priority 必须是整数", line_number))?;
            cursor += 1;
        } else {
            machine_ids.extend(split_csv_words(parts[cursor]));
            cursor += 1;
        }
    }
    Ok(MachineGraphRiskPlane {
        required: true,
        machine_ids,
        min_priority,
    })
}
