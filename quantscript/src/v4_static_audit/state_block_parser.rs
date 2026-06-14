use crate::Diagnostic;
use qrpc_core_ir::v4::{
    MachineCachePolicy, MachineRecoveryPolicy, MachineSilencePolicy, MachineState,
    MachineTemplateKind, V4MachineContract, V4_MACHINE_CONTRACT_VERSION,
};
use std::collections::BTreeMap;

use super::{diag, parse_machine_block, split_words, ParsedMachine, PreparedLine};

pub(super) fn parse_state(input: &str, line_number: usize) -> Result<MachineState, Diagnostic> {
    let parts = split_words(input);
    let Some(state_id) = parts.first() else {
        return Err(diag("QSV4110", "state 必须声明 state id", line_number));
    };
    if parts.contains(&"{") {
        return Err(diag(
            "QSV4110",
            "state 块必须使用 `state <id> [initial] [terminal] {` 并单独声明子 machine",
            line_number,
        ));
    }
    Ok(MachineState {
        state_id: (*state_id).to_string(),
        group_id: None,
        initial: parts.contains(&"initial"),
        terminal: parts.contains(&"terminal"),
        child_machine: None,
    })
}

pub(super) fn parse_state_block(
    lines: &[PreparedLine],
    start_index: usize,
    machine_depth: u32,
    parent_template: MachineTemplateKind,
) -> Result<(MachineState, usize), (Vec<Diagnostic>, usize)> {
    let header = &lines[start_index];
    let parts = split_words(&header.text);
    if parts.len() < 3 || parts[0] != "state" || parts.last() != Some(&"{") {
        return Err((
            vec![diag(
                "QSV4110",
                "state 块语法必须是 `state <id> [initial] [terminal] {`",
                header.number,
            )],
            start_index + 1,
        ));
    }

    let state_id = parts[1].to_string();
    if machine_depth >= 2 {
        return Err((
            vec![diag(
                "QSV4118",
                "嵌套状态机深度上限为 2，三级及以上仍为 reserved",
                header.number,
            )],
            skip_block(lines, start_index + 1),
        ));
    }

    let mut diagnostics = Vec::new();
    let mut child_machine = None;
    let mut index = start_index + 1;
    while index < lines.len() {
        let line = &lines[index];
        if line.text == "}" {
            index += 1;
            break;
        }
        if line.text.starts_with("machine ") {
            if child_machine.is_some() {
                diagnostics.push(diag(
                    "QSV4120",
                    "每个 state 块最多声明一个子 machine",
                    line.number,
                ));
                let (_, next_index) = parse_machine_block(
                    lines,
                    index,
                    machine_depth + 1,
                    Some(parent_template.clone()),
                )
                .unwrap_or_else(|(_, next)| {
                    (
                        ParsedMachine {
                            machine: empty_child_machine(&state_id, parent_template.clone()),
                        },
                        next,
                    )
                });
                index = next_index;
                continue;
            }
            match parse_machine_block(
                lines,
                index,
                machine_depth + 1,
                Some(parent_template.clone()),
            ) {
                Ok((machine, next_index)) => {
                    child_machine = Some(Box::new(machine.machine));
                    index = next_index;
                }
                Err((errors, next_index)) => {
                    diagnostics.extend(errors);
                    index = next_index;
                }
            }
            continue;
        }

        diagnostics.push(diag(
            "QSV4121",
            "state 块内只允许声明一个子 machine",
            line.number,
        ));
        index += 1;
    }

    if index >= lines.len() && lines.last().map(|line| line.text.as_str()) != Some("}") {
        diagnostics.push(diag("QSV4122", "state 块缺少结束 `}`", header.number));
    }

    let state = MachineState {
        state_id,
        group_id: None,
        initial: parts.contains(&"initial"),
        terminal: parts.contains(&"terminal"),
        child_machine,
    };

    if diagnostics.is_empty() {
        Ok((state, index))
    } else {
        Err((diagnostics, index))
    }
}

fn skip_block(lines: &[PreparedLine], start_index: usize) -> usize {
    let mut depth = 1usize;
    let mut index = start_index;
    while index < lines.len() {
        let text = lines[index].text.as_str();
        if text.ends_with('{') {
            depth += 1;
        }
        if text == "}" {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return index + 1;
            }
        }
        index += 1;
    }
    index
}

fn empty_child_machine(state_id: &str, template: MachineTemplateKind) -> V4MachineContract {
    V4MachineContract {
        schema_version: V4_MACHINE_CONTRACT_VERSION.to_string(),
        machine_id: format!("{state_id}.invalid_child"),
        template,
        states: Vec::new(),
        state_groups: Vec::new(),
        transitions: Vec::new(),
        memory: Vec::new(),
        cache_policy: MachineCachePolicy::ReturnLastThenRecover,
        silence_policy: MachineSilencePolicy::SoftDormantAfter { ttl_ms: 60_000 },
        recovery_policy: MachineRecoveryPolicy::AsyncRecover,
        priority: 0,
        metadata: BTreeMap::new(),
    }
}
