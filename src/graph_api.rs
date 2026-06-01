//! Compatibility shim for the former root graph API module.
//!
//! The graph route and persistence implementation now lives in
//! `backend::graph_compile::graph`.

#[cfg(test)]
pub(super) use crate::backend::graph_compile::graph::resolve_graph_reveal_path_from_value;
