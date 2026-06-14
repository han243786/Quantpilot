use crate::Diagnostic;
use qrpc_core_ir::v4::{StateGroup, TransitionConflictPolicy};

use super::parser_utilities_diagnostics::{diag, split_words};

pub(super) fn parse_state_group(input: &str, line_number: usize) -> Result<StateGroup, Diagnostic> {
    if input.contains('{') {
        return Err(diag(
            "QSV4111",
            "state_group 在 Phase 3 只能是扁平分组，不能打开嵌套块",
            line_number,
        ));
    }
    let parts = split_words(input);
    if parts.len() < 2 {
        return Err(diag(
            "QSV4112",
            "state_group 必须声明 group id 和至少一个 state",
            line_number,
        ));
    }
    Ok(StateGroup {
        group_id: parts[0].to_string(),
        state_ids: parts[1..]
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        conflict_policy: TransitionConflictPolicy::Error,
        timeout_ms: None,
    })
}
