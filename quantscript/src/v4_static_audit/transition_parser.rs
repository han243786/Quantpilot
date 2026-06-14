use crate::Diagnostic;
use qrpc_core_ir::v4::{MachineActionSpec, MachineEventSelector, MachineTransition};

use super::{diag, split_csv_words, split_words};

pub(super) fn parse_transition(
    input: &str,
    machine_id: &str,
    transition_index: usize,
    line_number: usize,
) -> Result<MachineTransition, Diagnostic> {
    let parts = split_words(input);
    if parts.len() < 6 || parts[1] != "from" || parts[3] != "to" {
        return Err(diag(
            "QSV4115",
            "transition 语法必须是 `on <event> from <state> to <state> [emit ...] [write ...]`",
            line_number,
        ));
    }
    let event_type = parts[0].to_string();
    let mut emits = Vec::new();
    let mut memory_writes = Vec::new();
    let mut cursor = 5;
    while cursor < parts.len() {
        match parts[cursor] {
            "emit" => {
                cursor += 1;
                while cursor < parts.len() && parts[cursor] != "write" {
                    emits.extend(split_csv_words(parts[cursor]));
                    cursor += 1;
                }
            }
            "write" => {
                cursor += 1;
                while cursor < parts.len() && parts[cursor] != "emit" {
                    memory_writes.extend(split_csv_words(parts[cursor]));
                    cursor += 1;
                }
            }
            other => {
                return Err(diag(
                    "QSV4116",
                    format!("transition 不支持的修饰符: {other}"),
                    line_number,
                ));
            }
        }
    }

    Ok(MachineTransition {
        transition_id: format!("{machine_id}.t{transition_index}"),
        from_state: parts[2].to_string(),
        to_state: parts[4].to_string(),
        event: MachineEventSelector {
            event_type,
            source: None,
            freshness: None,
        },
        guard: None,
        priority: 0,
        action: Some(MachineActionSpec {
            emits,
            memory_writes,
            diagnostics: Vec::new(),
        }),
    })
}
