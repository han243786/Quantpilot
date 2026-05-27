//! Backend parent module for v4.16 extraction.
//!
//! The first extraction pass keeps existing handlers and state owners in place,
//! while routing public backend boundaries through named leaf facades.

pub mod app_state_wiring;
pub mod capability;
pub mod graph_compile;
pub mod interface_boundary;
pub mod ops_governance;
pub mod runtime;
pub mod storage_security;
pub mod strategy_config;
pub mod test_support;
