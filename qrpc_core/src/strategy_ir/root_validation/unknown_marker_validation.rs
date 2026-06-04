use super::{KnownOrUnknown, StrategyIr};

pub(super) fn validate_unknowns(ir: &StrategyIr, errors: &mut Vec<String>) {
    for (index, item) in ir.unknowns.iter().enumerate() {
        if item.path.trim().is_empty() {
            errors.push(format!("unknowns[{index}].path 是必需的"));
        }
        if item.reason.trim().is_empty() {
            errors.push(format!("unknowns[{index}].reason 是必需的"));
        }
    }
}

pub(super) fn validate_unknownable<T>(
    value: &KnownOrUnknown<T>,
    path: &str,
    errors: &mut Vec<String>,
) {
    if let KnownOrUnknown::Unknown(marker) = value {
        if marker != "unknown" {
            errors.push(format!("{path} 未知标记必须为 \"unknown\""));
        }
    }
}

pub(super) fn validate_unknownable_opt<T>(
    value: Option<&KnownOrUnknown<T>>,
    path: &str,
    errors: &mut Vec<String>,
) {
    if let Some(value) = value {
        validate_unknownable(value, path, errors);
    }
}
