use crate::{Diagnostic, DiagnosticSeverity, Span};
use qrpc_core_ir::v4::{
    ExecutionCapabilityKind, MachineActionSpec, MachineCachePolicy, MachineEventCatalog,
    MachineEventPayloadField, MachineEventScope, MachineEventSelector, MachineEventSourceKind,
    MachineEventTypeSpec, MachineGraphEdge, MachineGraphEdgeActivation, MachineGraphRiskPlane,
    MachineMemoryField, MachineRecoveryPolicy, MachineSilencePolicy, MachineState,
    MachineTemplateKind, MachineTransition, QsScalarTypeKind, QsTypeRef, RuntimeTradingMode,
    StateGroup, TransitionConflictPolicy, V4CapabilityReportDiagnosticSeverity,
    V4CapabilityReportVerdict, V4CompileTimeCapabilityReport, V4CompileTimeCapabilityRequest,
    V4MachineContract, V4MachineGraphContract, V4StaticContractBundle,
    V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION, V4_MACHINE_CONTRACT_VERSION,
    V4_MACHINE_EVENT_CATALOG_VERSION, V4_MACHINE_GRAPH_CONTRACT_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

pub const V4_QS_STATIC_AUDIT_REPORT_VERSION: &str = "quantpilot/qs-v4-static-audit-report/v1";
pub const V4_QS_RUNTIME_HANDOFF_REPORT_VERSION: &str = "quantpilot/qs-v4-runtime-handoff-report/v1";

#[derive(Debug, Clone, PartialEq)]
pub struct V4QsStaticAuditReport {
    pub schema_version: String,
    pub verdict: V4QsStaticAuditVerdict,
    pub graph_id: Option<String>,
    pub parsed_graph: Option<V4MachineGraphContract>,
    pub capability_request: Option<V4CompileTimeCapabilityRequest>,
    pub capability_report: Option<V4CompileTimeCapabilityReport>,
    pub diagnostics: Vec<Diagnostic>,
    pub runtime_attached: bool,
    pub lowering_attached: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct V4QsRuntimeHandoffReport {
    pub schema_version: String,
    pub accepted_for_runtime_handoff: bool,
    pub graph_id: Option<String>,
    pub venue_id: Option<String>,
    pub runtime_mode: Option<RuntimeTradingMode>,
    pub paper_simulated_start_allowed: bool,
    pub provider_order_submission_attached: bool,
    pub runtime_attached: bool,
    pub lowering_attached: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V4QsStaticAuditVerdict {
    Accepted,
    Rejected,
}

struct ParsedV4QsStaticDocument {
    graph: V4MachineGraphContract,
    request: V4CompileTimeCapabilityRequest,
}

struct PreparedLine {
    number: usize,
    text: String,
}

struct ParsedMachine {
    machine: V4MachineContract,
}

pub fn audit_v4_quant_script_static(
    input: &str,
    bundle: &V4StaticContractBundle,
) -> V4QsStaticAuditReport {
    let mut diagnostics = Vec::new();
    let parsed = parse_v4_static_document(input);

    let Some(parsed) = parsed.map_err(|errors| diagnostics.extend(errors)).ok() else {
        return V4QsStaticAuditReport {
            schema_version: V4_QS_STATIC_AUDIT_REPORT_VERSION.to_string(),
            verdict: V4QsStaticAuditVerdict::Rejected,
            graph_id: None,
            parsed_graph: None,
            capability_request: None,
            capability_report: None,
            diagnostics,
            runtime_attached: false,
            lowering_attached: false,
        };
    };

    diagnostics.extend(
        parsed
            .graph
            .validate_static_contract()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|message| {
                Diagnostic::error(
                    "QSV4200",
                    format!("v4 状态机图静态校验失败: {message}"),
                    Some(Span::module(parsed.graph.graph_id.clone())),
                )
            }),
    );

    let mut report_bundle = bundle.clone();
    report_bundle
        .machine_graphs
        .retain(|graph| graph.graph_id != parsed.graph.graph_id);
    report_bundle.machine_graphs.push(parsed.graph.clone());
    let capability_report =
        report_bundle.build_compile_time_capability_report(parsed.request.clone());
    diagnostics.extend(
        capability_report
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == V4CapabilityReportDiagnosticSeverity::Error)
            .map(|diagnostic| {
                Diagnostic::error(
                    "QSV4300",
                    format!(
                        "v4 编译期能力报告拒绝: {} {}: {}",
                        diagnostic.code, diagnostic.target, diagnostic.message
                    ),
                    Some(Span::module(parsed.graph.graph_id.clone())),
                )
            }),
    );

    let rejected = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        || capability_report.verdict != V4CapabilityReportVerdict::Accepted;

    V4QsStaticAuditReport {
        schema_version: V4_QS_STATIC_AUDIT_REPORT_VERSION.to_string(),
        verdict: if rejected {
            V4QsStaticAuditVerdict::Rejected
        } else {
            V4QsStaticAuditVerdict::Accepted
        },
        graph_id: Some(parsed.graph.graph_id.clone()),
        parsed_graph: Some(parsed.graph),
        capability_request: Some(parsed.request),
        capability_report: Some(capability_report),
        diagnostics,
        runtime_attached: false,
        lowering_attached: false,
    }
}

pub fn build_v4_qs_runtime_handoff(report: &V4QsStaticAuditReport) -> V4QsRuntimeHandoffReport {
    let mut diagnostics = Vec::new();
    if report.verdict != V4QsStaticAuditVerdict::Accepted {
        diagnostics.push("v4 QS static audit must be accepted before runtime handoff".to_string());
    }
    if report.parsed_graph.is_none() {
        diagnostics.push("runtime handoff requires a parsed v4 machine graph".to_string());
    }
    let capability_report = report.capability_report.as_ref();
    if capability_report
        .map(|item| item.verdict != V4CapabilityReportVerdict::Accepted)
        .unwrap_or(true)
    {
        diagnostics.push(
            "runtime handoff requires an accepted compile-time capability report".to_string(),
        );
    }
    if capability_report
        .map(|item| item.execution_submission_attached)
        .unwrap_or(false)
    {
        diagnostics
            .push("runtime handoff must not carry execution submission attachment".to_string());
    }

    let request = report.capability_request.as_ref();
    let runtime_mode = request.map(|item| item.runtime_mode);
    if runtime_mode != Some(RuntimeTradingMode::PaperSimulated) {
        diagnostics.push("current v4 runtime handoff only allows PaperSimulated start".to_string());
    }

    let accepted = diagnostics.is_empty();
    V4QsRuntimeHandoffReport {
        schema_version: V4_QS_RUNTIME_HANDOFF_REPORT_VERSION.to_string(),
        accepted_for_runtime_handoff: accepted,
        graph_id: report.graph_id.clone(),
        venue_id: request.map(|item| item.venue_id.clone()),
        runtime_mode,
        paper_simulated_start_allowed: accepted,
        provider_order_submission_attached: false,
        runtime_attached: false,
        lowering_attached: false,
        diagnostics,
    }
}

fn parse_v4_static_document(input: &str) -> Result<ParsedV4QsStaticDocument, Vec<Diagnostic>> {
    let lines = prepare_lines(input);
    let mut diagnostics = Vec::new();
    let Some(header) = lines.first() else {
        return Err(vec![diag(
            "QSV4000",
            "v4 QS 静态审计需要 v4_strategy 顶层块",
            1,
        )]);
    };

    let header_parts = split_words(&header.text);
    if header_parts.len() != 3 || header_parts[0] != "v4_strategy" || header_parts[2] != "{" {
        return Err(vec![diag(
            "QSV4001",
            "v4 QS 顶层语法必须是 `v4_strategy <graph_id> {`",
            header.number,
        )]);
    }
    let graph_id = header_parts[1].to_string();
    let mut venue_id = None;
    let mut runtime_mode = None;
    let mut required_execution_capabilities = Vec::new();
    let mut required_type_refs = Vec::new();
    let mut required_plugin_ids = Vec::new();
    let mut machines = Vec::new();
    let mut edges = Vec::new();
    let mut risk_plane = None;

    let mut index = 1;
    let mut edge_index = 0usize;
    while index < lines.len() {
        let line = &lines[index];
        if line.text == "}" {
            index += 1;
            break;
        }

        if let Some(rest) = line.text.strip_prefix("venue ") {
            venue_id = Some(rest.trim().to_string());
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("mode ") {
            match parse_runtime_mode(rest.trim()) {
                Ok(mode) => runtime_mode = Some(mode),
                Err(message) => diagnostics.push(diag("QSV4002", message, line.number)),
            }
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("require capability ") {
            match parse_execution_capability(rest.trim()) {
                Ok(capability) => required_execution_capabilities.push(capability),
                Err(message) => diagnostics.push(diag("QSV4003", message, line.number)),
            }
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("require type ") {
            match parse_qs_type_ref(rest.trim()) {
                Ok(type_ref) => required_type_refs.push(type_ref),
                Err(message) => diagnostics.push(diag("QSV4004", message, line.number)),
            }
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("require plugin ") {
            let plugin_id = rest.trim();
            if plugin_id.is_empty() {
                diagnostics.push(diag(
                    "QSV4005",
                    "require plugin 必须声明 plugin id",
                    line.number,
                ));
            } else {
                required_plugin_ids.push(plugin_id.to_string());
            }
            index += 1;
            continue;
        }
        if line.text.starts_with("machine ") {
            match parse_machine_block(&lines, index) {
                Ok((machine, next_index)) => {
                    machines.push(machine.machine);
                    index = next_index;
                }
                Err((errors, next_index)) => {
                    diagnostics.extend(errors);
                    index = next_index;
                }
            }
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("edge ") {
            match parse_edge(rest, edge_index, line.number) {
                Ok(edge) => {
                    edges.push(edge);
                    edge_index += 1;
                }
                Err(error) => diagnostics.push(error),
            }
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("risk_plane ") {
            match parse_risk_plane(rest, line.number) {
                Ok(parsed_risk_plane) => risk_plane = Some(parsed_risk_plane),
                Err(error) => diagnostics.push(error),
            }
            index += 1;
            continue;
        }

        diagnostics.push(diag(
            "QSV4006",
            format!("v4 QS 顶层不支持的语句: {}", line.text),
            line.number,
        ));
        index += 1;
    }

    if index < lines.len() {
        diagnostics.push(diag(
            "QSV4007",
            "v4_strategy 顶层块结束后不能继续声明内容",
            lines[index].number,
        ));
    }
    if venue_id.as_deref().unwrap_or_default().is_empty() {
        diagnostics.push(diag("QSV4008", "v4 QS 必须声明 venue", header.number));
    }
    if runtime_mode.is_none() {
        diagnostics.push(diag("QSV4009", "v4 QS 必须声明 mode", header.number));
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut graph = V4MachineGraphContract {
        schema_version: V4_MACHINE_GRAPH_CONTRACT_VERSION.to_string(),
        graph_id: graph_id.clone(),
        machines,
        edges,
        event_catalog: None,
        risk_plane,
        metadata: BTreeMap::new(),
    };
    graph.event_catalog = Some(derive_event_catalog(&graph));

    Ok(ParsedV4QsStaticDocument {
        request: V4CompileTimeCapabilityRequest {
            schema_version: V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION.to_string(),
            graph_id,
            venue_id: venue_id.unwrap_or_default(),
            runtime_mode: runtime_mode.unwrap(),
            required_execution_capabilities,
            required_type_refs,
            required_plugin_ids,
        },
        graph,
    })
}

fn parse_machine_block(
    lines: &[PreparedLine],
    start_index: usize,
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
                "v4 Phase 3 只允许扁平 state + state_group，嵌套 machine 仍是 reserved",
                line.number,
            ));
            index += 1;
            continue;
        }
        if let Some(rest) = line.text.strip_prefix("state ") {
            match parse_state(rest, line.number) {
                Ok(state) => states.push(state),
                Err(error) => diagnostics.push(error),
            }
            index += 1;
            continue;
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

fn parse_state(input: &str, line_number: usize) -> Result<MachineState, Diagnostic> {
    let parts = split_words(input);
    let Some(state_id) = parts.first() else {
        return Err(diag("QSV4110", "state 必须声明 state id", line_number));
    };
    Ok(MachineState {
        state_id: (*state_id).to_string(),
        group_id: None,
        initial: parts.iter().any(|part| *part == "initial"),
        terminal: parts.iter().any(|part| *part == "terminal"),
    })
}

fn parse_state_group(input: &str, line_number: usize) -> Result<StateGroup, Diagnostic> {
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

fn parse_memory(input: &str, line_number: usize) -> Result<MachineMemoryField, Diagnostic> {
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
    Ok(MachineMemoryField {
        name: name.trim().to_string(),
        type_name: (*type_name).to_string(),
        default_value: None,
        nullable: parts.iter().any(|part| *part == "nullable"),
    })
}

fn parse_transition(
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

fn parse_edge(
    input: &str,
    edge_index: usize,
    line_number: usize,
) -> Result<MachineGraphEdge, Diagnostic> {
    let parts = split_words(input);
    if parts.len() != 5 || parts[1] != "->" || parts[3] != "on" {
        return Err(diag(
            "QSV4120",
            "edge 语法必须是 `edge <source> -> <target> on <event>`",
            line_number,
        ));
    }
    Ok(MachineGraphEdge {
        edge_id: format!("edge.{edge_index}.{}.{}", parts[0], parts[2]),
        source_machine_id: parts[0].to_string(),
        target_machine_id: parts[2].to_string(),
        event_type: parts[4].to_string(),
        activation: MachineGraphEdgeActivation::Always,
        required: true,
        metadata: BTreeMap::new(),
    })
}

fn parse_risk_plane(input: &str, line_number: usize) -> Result<MachineGraphRiskPlane, Diagnostic> {
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

fn derive_event_catalog(graph: &V4MachineGraphContract) -> MachineEventCatalog {
    let risk_machine_ids = graph
        .risk_plane
        .as_ref()
        .map(|risk_plane| {
            risk_plane
                .machine_ids
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let mut emitters: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut consumers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for machine in &graph.machines {
        for transition in &machine.transitions {
            consumers
                .entry(transition.event.event_type.clone())
                .or_default()
                .insert(machine.machine_id.clone());
            if let Some(action) = &transition.action {
                for event_type in &action.emits {
                    emitters
                        .entry(event_type.clone())
                        .or_default()
                        .insert(machine.machine_id.clone());
                }
            }
        }
    }
    for edge in &graph.edges {
        emitters
            .entry(edge.event_type.clone())
            .or_default()
            .insert(edge.source_machine_id.clone());
        consumers
            .entry(edge.event_type.clone())
            .or_default()
            .insert(edge.target_machine_id.clone());
    }

    let event_types = emitters
        .keys()
        .chain(consumers.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let events = event_types
        .into_iter()
        .map(|event_type| {
            let allowed_emitters = emitters
                .remove(&event_type)
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            let allowed_consumers = consumers
                .remove(&event_type)
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            let source_kind = if allowed_emitters
                .iter()
                .any(|machine_id| risk_machine_ids.contains(machine_id.as_str()))
                || event_type.starts_with("risk.")
            {
                MachineEventSourceKind::RiskPlane
            } else if allowed_emitters.is_empty() && event_type.starts_with("market.") {
                MachineEventSourceKind::MarketData
            } else {
                MachineEventSourceKind::Machine
            };
            MachineEventTypeSpec {
                event_type,
                source_kind,
                scope: MachineEventScope::Graph,
                payload_fields: Vec::<MachineEventPayloadField>::new(),
                allowed_emitters,
                allowed_consumers,
                replayable: true,
            }
        })
        .collect();

    MachineEventCatalog {
        schema_version: V4_MACHINE_EVENT_CATALOG_VERSION.to_string(),
        events,
        metadata: BTreeMap::new(),
    }
}

fn parse_machine_template(input: &str) -> Result<MachineTemplateKind, String> {
    match input {
        "observation" => Ok(MachineTemplateKind::Observation),
        "decision" => Ok(MachineTemplateKind::Decision),
        "execution" => Ok(MachineTemplateKind::Execution),
        other => Err(format!("未知 machine template: {other}")),
    }
}

fn parse_runtime_mode(input: &str) -> Result<RuntimeTradingMode, String> {
    match input {
        "paper_actual" => Ok(RuntimeTradingMode::PaperActual),
        "paper_simulated" => Ok(RuntimeTradingMode::PaperSimulated),
        "live_actual" => Ok(RuntimeTradingMode::LiveActual),
        "live_simulated" => Ok(RuntimeTradingMode::LiveSimulated),
        other => Err(format!("未知 runtime mode: {other}")),
    }
}

fn parse_execution_capability(input: &str) -> Result<ExecutionCapabilityKind, String> {
    match input {
        "market" => Ok(ExecutionCapabilityKind::Market),
        "limit" => Ok(ExecutionCapabilityKind::Limit),
        "post_only" | "limit_maker" => Ok(ExecutionCapabilityKind::PostOnly),
        "stop_market" => Ok(ExecutionCapabilityKind::StopMarket),
        "stop_limit" => Ok(ExecutionCapabilityKind::StopLimit),
        "take_profit_market" => Ok(ExecutionCapabilityKind::TakeProfitMarket),
        "take_profit_limit" => Ok(ExecutionCapabilityKind::TakeProfitLimit),
        "ioc" => Ok(ExecutionCapabilityKind::Ioc),
        "fok" => Ok(ExecutionCapabilityKind::Fok),
        "oco_bracket" | "bracket_tp_sl" | "oco" => Ok(ExecutionCapabilityKind::OcoBracket),
        "trailing_stop" => Ok(ExecutionCapabilityKind::TrailingStop),
        "reduce_only" => Ok(ExecutionCapabilityKind::ReduceOnly),
        "close_only" => Ok(ExecutionCapabilityKind::CloseOnly),
        "open_long" => Ok(ExecutionCapabilityKind::OpenLong),
        "close_long" => Ok(ExecutionCapabilityKind::CloseLong),
        "open_short" => Ok(ExecutionCapabilityKind::OpenShort),
        "close_short" => Ok(ExecutionCapabilityKind::CloseShort),
        "one_way_position_mode" | "one_way" => Ok(ExecutionCapabilityKind::OneWayPositionMode),
        "hedge_position_mode" | "hedge" => Ok(ExecutionCapabilityKind::HedgePositionMode),
        "gtc" => Ok(ExecutionCapabilityKind::Gtc),
        "day" => Ok(ExecutionCapabilityKind::Day),
        "gtd" => Ok(ExecutionCapabilityKind::Gtd),
        "client_order_id" => Ok(ExecutionCapabilityKind::ClientOrderId),
        "cancel_replace_amend" | "cancel" | "replace" | "amend" => {
            Ok(ExecutionCapabilityKind::CancelReplaceAmend)
        }
        other => Err(format!("未知 execution capability: {other}")),
    }
}

fn parse_qs_type_ref(input: &str) -> Result<QsTypeRef, String> {
    let input = input.trim();
    if let Some(inner) = strip_wrapper(input, "optional") {
        return Ok(QsTypeRef::Optional {
            inner: Box::new(parse_qs_type_ref(inner)?),
        });
    }
    if let Some(inner) = strip_wrapper(input, "fresh") {
        return Ok(QsTypeRef::Fresh {
            inner: Box::new(parse_qs_type_ref(inner)?),
        });
    }
    if let Some(inner) = strip_wrapper(input, "stale") {
        return Ok(QsTypeRef::Stale {
            inner: Box::new(parse_qs_type_ref(inner)?),
        });
    }
    if let Some(inner) = strip_wrapper(input, "list") {
        let (item, max_items) = parse_items_type_args(inner)?;
        return Ok(QsTypeRef::List {
            item: Box::new(parse_qs_type_ref(item)?),
            max_items,
        });
    }
    if let Some(inner) = strip_wrapper(input, "map") {
        let args = split_top_level_args(inner);
        if args.len() != 3 {
            return Err("map 类型必须写成 map<key,value,max=N>".to_string());
        }
        let key = parse_qs_scalar_type(args[0])?;
        let max_items = parse_max_arg(args[2])?;
        return Ok(QsTypeRef::Map {
            key,
            value: Box::new(parse_qs_type_ref(args[1])?),
            max_items,
        });
    }
    Ok(QsTypeRef::Scalar {
        scalar: parse_qs_scalar_type(input)?,
    })
}

fn strip_wrapper<'a>(input: &'a str, wrapper: &str) -> Option<&'a str> {
    input
        .strip_prefix(wrapper)?
        .strip_prefix('<')?
        .strip_suffix('>')
        .map(str::trim)
}

fn parse_items_type_args(input: &str) -> Result<(&str, u32), String> {
    let args = split_top_level_args(input);
    if args.len() != 2 {
        return Err("list 类型必须写成 list<T,max=N>".to_string());
    }
    Ok((args[0], parse_max_arg(args[1])?))
}

fn parse_max_arg(input: &str) -> Result<u32, String> {
    let Some(value) = input.trim().strip_prefix("max=") else {
        return Err("容量参数必须写成 max=N".to_string());
    };
    value
        .parse::<u32>()
        .map_err(|_| "容量参数 max 必须是正整数".to_string())
}

fn split_top_level_args(input: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in input.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                args.push(input[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    args.push(input[start..].trim());
    args
}

fn parse_qs_scalar_type(input: &str) -> Result<QsScalarTypeKind, String> {
    match input {
        "bool" => Ok(QsScalarTypeKind::Bool),
        "int" => Ok(QsScalarTypeKind::Int),
        "decimal" => Ok(QsScalarTypeKind::Decimal),
        "time" => Ok(QsScalarTypeKind::Time),
        "duration" => Ok(QsScalarTypeKind::Duration),
        "price" => Ok(QsScalarTypeKind::Price),
        "quantity" => Ok(QsScalarTypeKind::Quantity),
        "notional" => Ok(QsScalarTypeKind::Notional),
        "percent" => Ok(QsScalarTypeKind::Percent),
        "ratio" => Ok(QsScalarTypeKind::Ratio),
        "fee" => Ok(QsScalarTypeKind::Fee),
        "slippage" => Ok(QsScalarTypeKind::Slippage),
        "leverage" => Ok(QsScalarTypeKind::Leverage),
        "symbol" => Ok(QsScalarTypeKind::Symbol),
        "venue" => Ok(QsScalarTypeKind::Venue),
        "account" => Ok(QsScalarTypeKind::Account),
        "side" => Ok(QsScalarTypeKind::Side),
        "position_side" => Ok(QsScalarTypeKind::PositionSide),
        "order_type" => Ok(QsScalarTypeKind::OrderType),
        "time_in_force" => Ok(QsScalarTypeKind::TimeInForce),
        "freshness" => Ok(QsScalarTypeKind::Freshness),
        "runtime_mode" => Ok(QsScalarTypeKind::RuntimeMode),
        "order_permission" => Ok(QsScalarTypeKind::OrderPermission),
        other => Err(format!("未知 QS 类型: {other}")),
    }
}

fn prepare_lines(input: &str) -> Vec<PreparedLine> {
    input
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let without_slash_comment = line.split_once("//").map(|(head, _)| head).unwrap_or(line);
            let without_comment = without_slash_comment
                .split_once('#')
                .map(|(head, _)| head)
                .unwrap_or(without_slash_comment);
            let text = without_comment.trim();
            if text.is_empty() {
                None
            } else {
                Some(PreparedLine {
                    number: index + 1,
                    text: text.to_string(),
                })
            }
        })
        .collect()
}

fn split_words(input: &str) -> Vec<&str> {
    input.split_whitespace().collect()
}

fn split_csv_words(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn diag(code: &'static str, message: impl Into<String>, line_number: usize) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        Some(Span::module(format!("line {line_number}"))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core_ir::v4::{
        CapabilitySupportSource, PluginKind, PluginManifestSpec, PluginNetworkPermission,
        PluginRuntimePermission, PluginSideEffect, VenueCapabilityMatrix,
    };

    const SAMPLE_V4_QS: &str = r#"
v4_strategy strategy.v4.sample {
  venue paper-local
  mode paper_simulated
  require capability market
  require type fresh<list<price,max=256>>
  require plugin pure.indicator.zscore

  machine data.market observation priority 8000 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on market.tick from idle to ready emit bar_closed write last_signal_at
  }

  machine intent.trend decision priority 5200 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on bar_closed from idle to ready emit intent.long write last_signal_at
  }

  machine risk.guard decision priority 9500 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on intent.long from idle to ready emit risk.approved write last_signal_at
  }

  machine execution.router execution priority 4000 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on risk.approved from idle to ready write last_signal_at
  }

  edge data.market -> intent.trend on bar_closed
  edge intent.trend -> risk.guard on intent.long
  edge risk.guard -> execution.router on risk.approved
  risk_plane risk.guard priority 9000
}
"#;

    fn bundle_with_market_support() -> V4StaticContractBundle {
        V4StaticContractBundle {
            venue_matrices: vec![market_supported_matrix()],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        }
    }

    fn market_supported_matrix() -> VenueCapabilityMatrix {
        let mut matrix = qrpc_core_ir::v4::unsupported_v4_first_wave_matrix("paper-local");
        let market = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
            .unwrap();
        market.source = CapabilitySupportSource::RuntimeSimulated;
        market.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
        matrix
    }

    fn sample_pure_plugin_manifest() -> PluginManifestSpec {
        PluginManifestSpec {
            plugin_id: "pure.indicator.zscore".to_string(),
            name: "ZScore".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::Pure,
            input_schema: Some(QsTypeRef::List {
                item: Box::new(QsTypeRef::Scalar {
                    scalar: QsScalarTypeKind::Price,
                }),
                max_items: 256,
            }),
            output_schema: Some(QsTypeRef::Scalar {
                scalar: QsScalarTypeKind::Decimal,
            }),
            deterministic: true,
            side_effect: PluginSideEffect::None,
            runtime_permission: PluginRuntimePermission::None,
            network_permission: PluginNetworkPermission::None,
            capability_matrix: None,
            test_fixture_id: "fixture.zscore.basic".to_string(),
        }
    }

    #[test]
    fn v4_static_audit_accepts_supported_state_machine_script_without_runtime() {
        let report = audit_v4_quant_script_static(SAMPLE_V4_QS, &bundle_with_market_support());

        assert_eq!(report.verdict, V4QsStaticAuditVerdict::Accepted);
        assert_eq!(report.graph_id.as_deref(), Some("strategy.v4.sample"));
        assert!(!report.runtime_attached);
        assert!(!report.lowering_attached);
        assert_eq!(
            report.capability_report.as_ref().unwrap().verdict,
            V4CapabilityReportVerdict::Accepted
        );
        assert_eq!(report.parsed_graph.as_ref().unwrap().machines.len(), 4);
    }

    #[test]
    fn v4_static_audit_builds_safe_paper_simulated_runtime_handoff() {
        let report = audit_v4_quant_script_static(SAMPLE_V4_QS, &bundle_with_market_support());
        let handoff = build_v4_qs_runtime_handoff(&report);

        assert!(handoff.accepted_for_runtime_handoff);
        assert!(handoff.paper_simulated_start_allowed);
        assert_eq!(handoff.graph_id.as_deref(), Some("strategy.v4.sample"));
        assert_eq!(handoff.venue_id.as_deref(), Some("paper-local"));
        assert_eq!(
            handoff.runtime_mode,
            Some(RuntimeTradingMode::PaperSimulated)
        );
        assert!(!handoff.provider_order_submission_attached);
        assert!(!handoff.runtime_attached);
        assert!(!handoff.lowering_attached);
        assert!(handoff.diagnostics.is_empty());
    }

    #[test]
    fn v4_static_audit_rejects_unsupported_required_capability() {
        let bundle = V4StaticContractBundle {
            venue_matrices: vec![qrpc_core_ir::v4::unsupported_v4_first_wave_matrix(
                "paper-local",
            )],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };

        let report = audit_v4_quant_script_static(SAMPLE_V4_QS, &bundle);

        assert_eq!(report.verdict, V4QsStaticAuditVerdict::Rejected);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QSV4300"));
        let handoff = build_v4_qs_runtime_handoff(&report);
        assert!(!handoff.accepted_for_runtime_handoff);
        assert!(handoff
            .diagnostics
            .iter()
            .any(|message| message.contains("static audit must be accepted")));
    }

    #[test]
    fn v4_static_audit_rejects_nested_machine_blocks() {
        let source = SAMPLE_V4_QS.replace(
            "state ready\n    state_group active idle ready",
            "state ready\n    machine nested decision {",
        );

        let report = audit_v4_quant_script_static(&source, &bundle_with_market_support());

        assert_eq!(report.verdict, V4QsStaticAuditVerdict::Rejected);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QSV4104"));
    }

    #[test]
    fn v4_static_audit_rejects_transition_without_event_syntax() {
        let source = SAMPLE_V4_QS.replace(
            "on risk.approved from idle to ready write last_signal_at",
            "transition idle -> ready",
        );

        let report = audit_v4_quant_script_static(&source, &bundle_with_market_support());

        assert_eq!(report.verdict, V4QsStaticAuditVerdict::Rejected);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QSV4105"));
    }

    #[test]
    fn v4_static_audit_rejects_mode_capability_source_mismatch() {
        let mut matrix = market_supported_matrix();
        let market = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
            .unwrap();
        market.source = CapabilitySupportSource::ProviderNative;
        let bundle = V4StaticContractBundle {
            venue_matrices: vec![matrix],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };

        let report = audit_v4_quant_script_static(SAMPLE_V4_QS, &bundle);

        assert_eq!(report.verdict, V4QsStaticAuditVerdict::Rejected);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("requires runtime_simulated")));
    }
}
