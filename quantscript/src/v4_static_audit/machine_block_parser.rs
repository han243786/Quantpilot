use crate::Diagnostic;
use qrpc_core_ir::v4::{
    MachineCachePolicy, MachineRecoveryPolicy, MachineSilencePolicy, MachineTemplateKind,
    V4MachineContract, V4_MACHINE_CONTRACT_VERSION,
};
use std::collections::BTreeMap;

use super::{
    diag, parse_machine_template, parse_memory, parse_state, parse_state_block, parse_state_group,
    parse_transition, split_words, PreparedLine,
};

pub(super) struct ParsedMachine {
    pub(super) machine: V4MachineContract,
}

pub(super) fn parse_machine_block(
    lines: &[PreparedLine],
    start_index: usize,
    machine_depth: u32,
    parent_template: Option<MachineTemplateKind>,
) -> Result<(ParsedMachine, usize), (Vec<Diagnostic>, usize)> {
    let header = &lines[start_index];
    let parts = split_words(&header.text);
    if parts.len() < 4 || parts[0] != "machine" || parts.last() != Some(&"{") {
        return Err((
            vec![diag(
                "QSV4100",
                "machine 语法必须是 `machine <id> <template> [priority N] {`",
                header.number,
            )],
            start_index + 1,
        ));
    }

    let machine_id = parts[1].to_string();
    let template = match parse_machine_template(parts[2]) {
        Ok(template) => template,
        Err(message) => {
            return Err((
                vec![diag("QSV4101", message, header.number)],
                start_index + 1,
            ));
        }
    };
    if let Some(expected_template) = parent_template {
        if template != expected_template {
            return Err((
                vec![diag(
                    "QSV4119",
                    "子 machine template 必须与父 machine template 一致",
                    header.number,
                )],
                start_index + 1,
            ));
        }
    }
    let mut priority = 0;
    if parts.len() > 4 {
        if parts.len() != 6 || parts[3] != "priority" {
            return Err((
                vec![diag(
                    "QSV4102",
                    "machine header 只允许追加 `priority <number>`",
                    header.number,
                )],
                start_index + 1,
            ));
        }
        priority = match parts[4].parse::<i32>() {
            Ok(value) => value,
            Err(_) => {
                return Err((
                    vec![diag(
                        "QSV4103",
                        "machine priority 必须是整数",
                        header.number,
                    )],
                    start_index + 1,
                ));
            }
        };
    }

    let mut diagnostics = Vec::new();
    let mut states = Vec::new();
    let mut state_groups = Vec::new();
    let mut transitions = Vec::new();
    let mut memory = Vec::new();
    let mut index = start_index + 1;
    while index < lines.len() {
        let line = &lines[index];
        if line.text == "}" {
            index += 1;
            break;
        }
        if line.text.starts_with("machine ") {
            diagnostics.push(diag(
                "QSV4104",
                "嵌套 machine 必须声明在 state 块内部",
                line.number,
            ));
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("state ") {
            if line.text.ends_with('{') {
                match parse_state_block(lines, index, machine_depth, template.clone()) {
                    Ok((state, next_index)) => {
                        states.push(state);
                        index = next_index;
                    }
                    Err((errors, next_index)) => {
                        diagnostics.extend(errors);
                        index = next_index;
                    }
                }
                continue;
            } else {
                match parse_state(rest, line.number) {
                    Ok(state) => states.push(state),
                    Err(error) => diagnostics.push(error),
                }
                index += 1;
                continue;
            }
        }
        if let Some(rest) = line
            .text
            .strip_prefix("group ")
            .or_else(|| line.text.strip_prefix("state_group "))
        {
            match parse_state_group(rest, line.number) {
                Ok(group) => state_groups.push(group),
                Err(error) => diagnostics.push(error),
            }
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("memory ") {
            match parse_memory(rest, line.number) {
                Ok(field) => memory.push(field),
                Err(error) => diagnostics.push(error),
            }
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("on ") {
            match parse_transition(rest, &machine_id, transitions.len(), line.number) {
                Ok(transition) => transitions.push(transition),
                Err(error) => diagnostics.push(error),
            }
            index += 1;
            continue;
        }

        diagnostics.push(diag(
            "QSV4105",
            format!("machine `{machine_id}` 中不支持的语句: {}", line.text),
            line.number,
        ));
        index += 1;
    }

    let machine = V4MachineContract {
        schema_version: V4_MACHINE_CONTRACT_VERSION.to_string(),
        machine_id,
        template,
        states,
        state_groups,
        transitions,
        memory,
        cache_policy: MachineCachePolicy::ReturnLastThenRecover,
        silence_policy: MachineSilencePolicy::SoftDormantAfter { ttl_ms: 60_000 },
        recovery_policy: MachineRecoveryPolicy::AsyncRecover,
        priority,
        metadata: BTreeMap::new(),
    };

    if diagnostics.is_empty() {
        Ok((ParsedMachine { machine }, index))
    } else {
        Err((diagnostics, index))
    }
}
