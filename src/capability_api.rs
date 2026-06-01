//! Compatibility shim for the former root capability API module.
//!
//! The capability snapshot and contract implementation now lives in
//! `backend::capability::snapshot`.

pub(super) use crate::backend::capability::snapshot::*;
