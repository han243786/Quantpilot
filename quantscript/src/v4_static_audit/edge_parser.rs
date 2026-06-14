use crate::Diagnostic;
use qrpc_core_ir::v4::{MachineGraphEdge, MachineGraphEdgeActivation};
use std::collections::BTreeMap;

use super::{diag, split_words};

pub(super) fn parse_edge(
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
