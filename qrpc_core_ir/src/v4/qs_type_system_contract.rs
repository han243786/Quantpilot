use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use super::{default_true, V4_QS_TYPE_MAX_NESTING_DEPTH, V4_QS_TYPE_SYSTEM_VERSION};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QsTypeSystemContract {
    #[serde(default = "default_qs_type_system_version")]
    pub schema_version: String,
    #[serde(default = "default_qs_scalar_types")]
    pub scalar_types: Vec<QsScalarTypeKind>,
    #[serde(default = "default_qs_composite_type_specs")]
    pub composite_types: Vec<QsCompositeTypeSpec>,
    #[serde(default = "default_qs_type_max_nesting_depth")]
    pub max_nesting_depth: u8,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QsScalarTypeKind {
    Bool,
    Int,
    Decimal,
    Time,
    Duration,
    Price,
    Quantity,
    Notional,
    Percent,
    Ratio,
    Fee,
    Slippage,
    Leverage,
    Symbol,
    Venue,
    Account,
    Side,
    PositionSide,
    OrderType,
    TimeInForce,
    Freshness,
    RuntimeMode,
    OrderPermission,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum QsCompositeTypeKind {
    Optional,
    List,
    Map,
    Fresh,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QsCompositeTypeSpec {
    pub kind: QsCompositeTypeKind,
    #[serde(default)]
    pub max_items_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items_upper_bound: Option<u32>,
    #[serde(default = "default_true")]
    pub replay_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QsTypeRef {
    Scalar {
        scalar: QsScalarTypeKind,
    },
    Optional {
        inner: Box<QsTypeRef>,
    },
    List {
        item: Box<QsTypeRef>,
        max_items: u32,
    },
    Map {
        key: QsScalarTypeKind,
        value: Box<QsTypeRef>,
        max_items: u32,
    },
    Fresh {
        inner: Box<QsTypeRef>,
    },
    Stale {
        inner: Box<QsTypeRef>,
    },
}

pub const V4_FIRST_WAVE_QS_SCALAR_TYPES: [QsScalarTypeKind; 23] = [
    QsScalarTypeKind::Bool,
    QsScalarTypeKind::Int,
    QsScalarTypeKind::Decimal,
    QsScalarTypeKind::Time,
    QsScalarTypeKind::Duration,
    QsScalarTypeKind::Price,
    QsScalarTypeKind::Quantity,
    QsScalarTypeKind::Notional,
    QsScalarTypeKind::Percent,
    QsScalarTypeKind::Ratio,
    QsScalarTypeKind::Fee,
    QsScalarTypeKind::Slippage,
    QsScalarTypeKind::Leverage,
    QsScalarTypeKind::Symbol,
    QsScalarTypeKind::Venue,
    QsScalarTypeKind::Account,
    QsScalarTypeKind::Side,
    QsScalarTypeKind::PositionSide,
    QsScalarTypeKind::OrderType,
    QsScalarTypeKind::TimeInForce,
    QsScalarTypeKind::Freshness,
    QsScalarTypeKind::RuntimeMode,
    QsScalarTypeKind::OrderPermission,
];

pub const V4_FIRST_WAVE_QS_COMPOSITE_TYPES: [QsCompositeTypeKind; 5] = [
    QsCompositeTypeKind::Optional,
    QsCompositeTypeKind::List,
    QsCompositeTypeKind::Map,
    QsCompositeTypeKind::Fresh,
    QsCompositeTypeKind::Stale,
];

pub fn v4_first_wave_scalar_types() -> &'static [QsScalarTypeKind] {
    &V4_FIRST_WAVE_QS_SCALAR_TYPES
}

pub fn v4_first_wave_composite_types() -> &'static [QsCompositeTypeKind] {
    &V4_FIRST_WAVE_QS_COMPOSITE_TYPES
}

impl Default for QsTypeSystemContract {
    fn default() -> Self {
        default_v4_qs_type_system_contract()
    }
}

pub fn default_v4_qs_type_system_contract() -> QsTypeSystemContract {
    QsTypeSystemContract {
        schema_version: V4_QS_TYPE_SYSTEM_VERSION.to_string(),
        scalar_types: default_qs_scalar_types(),
        composite_types: default_qs_composite_type_specs(),
        max_nesting_depth: V4_QS_TYPE_MAX_NESTING_DEPTH,
        metadata: BTreeMap::new(),
    }
}

impl QsTypeSystemContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_QS_TYPE_SYSTEM_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_QS_TYPE_SYSTEM_VERSION
            ));
        }
        if self.max_nesting_depth == 0 {
            errors.push("max_nesting_depth must be greater than 0".to_string());
        }

        let mut scalar_set = BTreeSet::new();
        for scalar in &self.scalar_types {
            if !scalar_set.insert(*scalar) {
                errors.push(format!("duplicate scalar type `{:?}`", scalar));
            }
        }
        for scalar in v4_first_wave_scalar_types() {
            if !scalar_set.contains(scalar) {
                errors.push(format!(
                    "QS type system must declare scalar type `{:?}`",
                    scalar
                ));
            }
        }

        let mut composite_specs = BTreeMap::new();
        for spec in &self.composite_types {
            if composite_specs.insert(spec.kind, spec).is_some() {
                errors.push(format!("duplicate composite type `{:?}`", spec.kind));
            }
            if spec.max_items_required {
                match spec.max_items_upper_bound {
                    Some(0) => errors.push(format!(
                        "composite type `{:?}` max_items_upper_bound must be greater than 0",
                        spec.kind
                    )),
                    None => errors.push(format!(
                        "composite type `{:?}` requires max_items_upper_bound",
                        spec.kind
                    )),
                    Some(_) => {}
                }
            }
            if !spec.replay_safe {
                errors.push(format!(
                    "composite type `{:?}` must be replay_safe",
                    spec.kind
                ));
            }
        }
        for composite in v4_first_wave_composite_types() {
            if !composite_specs.contains_key(composite) {
                errors.push(format!(
                    "QS type system must declare composite type `{:?}`",
                    composite
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn validate_type_ref(&self, type_ref: &QsTypeRef) -> Result<(), Vec<String>> {
        let mut errors = self.validate_static_contract().err().unwrap_or_default();
        self.validate_type_ref_inner(type_ref, 1, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_type_ref_inner(&self, type_ref: &QsTypeRef, depth: u8, errors: &mut Vec<String>) {
        if depth > self.max_nesting_depth {
            errors.push(format!(
                "QS type nesting depth {} exceeds max_nesting_depth {}",
                depth, self.max_nesting_depth
            ));
            return;
        }

        match type_ref {
            QsTypeRef::Scalar { scalar } => {
                if !self.scalar_types.contains(scalar) {
                    errors.push(format!("scalar type `{:?}` is not declared", scalar));
                }
            }
            QsTypeRef::Optional { inner } => {
                self.validate_composite_ref(QsCompositeTypeKind::Optional, None, errors);
                self.validate_type_ref_inner(inner, depth + 1, errors);
            }
            QsTypeRef::List { item, max_items } => {
                self.validate_composite_ref(QsCompositeTypeKind::List, Some(*max_items), errors);
                self.validate_type_ref_inner(item, depth + 1, errors);
            }
            QsTypeRef::Map {
                key,
                value,
                max_items,
            } => {
                self.validate_composite_ref(QsCompositeTypeKind::Map, Some(*max_items), errors);
                if !self.scalar_types.contains(key) {
                    errors.push(format!("map key scalar type `{:?}` is not declared", key));
                }
                self.validate_type_ref_inner(value, depth + 1, errors);
            }
            QsTypeRef::Fresh { inner } => {
                self.validate_composite_ref(QsCompositeTypeKind::Fresh, None, errors);
                self.validate_type_ref_inner(inner, depth + 1, errors);
            }
            QsTypeRef::Stale { inner } => {
                self.validate_composite_ref(QsCompositeTypeKind::Stale, None, errors);
                self.validate_type_ref_inner(inner, depth + 1, errors);
            }
        }
    }

    fn validate_composite_ref(
        &self,
        kind: QsCompositeTypeKind,
        max_items: Option<u32>,
        errors: &mut Vec<String>,
    ) {
        let Some(spec) = self.composite_types.iter().find(|spec| spec.kind == kind) else {
            errors.push(format!("composite type `{:?}` is not declared", kind));
            return;
        };

        if spec.max_items_required {
            match max_items {
                Some(0) | None => errors.push(format!(
                    "composite type `{:?}` requires max_items greater than 0",
                    kind
                )),
                Some(value) => {
                    if let Some(limit) = spec.max_items_upper_bound {
                        if value > limit {
                            errors.push(format!(
                                "composite type `{:?}` max_items {} exceeds upper bound {}",
                                kind, value, limit
                            ));
                        }
                    }
                }
            }
        }
    }
}

pub(in crate::v4) fn default_qs_type_system_version() -> String {
    V4_QS_TYPE_SYSTEM_VERSION.to_string()
}

fn default_qs_scalar_types() -> Vec<QsScalarTypeKind> {
    v4_first_wave_scalar_types().to_vec()
}

fn default_qs_composite_type_specs() -> Vec<QsCompositeTypeSpec> {
    vec![
        QsCompositeTypeSpec {
            kind: QsCompositeTypeKind::Optional,
            max_items_required: false,
            max_items_upper_bound: None,
            replay_safe: true,
        },
        QsCompositeTypeSpec {
            kind: QsCompositeTypeKind::List,
            max_items_required: true,
            max_items_upper_bound: Some(10_000),
            replay_safe: true,
        },
        QsCompositeTypeSpec {
            kind: QsCompositeTypeKind::Map,
            max_items_required: true,
            max_items_upper_bound: Some(10_000),
            replay_safe: true,
        },
        QsCompositeTypeSpec {
            kind: QsCompositeTypeKind::Fresh,
            max_items_required: false,
            max_items_upper_bound: None,
            replay_safe: true,
        },
        QsCompositeTypeSpec {
            kind: QsCompositeTypeKind::Stale,
            max_items_required: false,
            max_items_upper_bound: None,
            replay_safe: true,
        },
    ]
}

fn default_qs_type_max_nesting_depth() -> u8 {
    V4_QS_TYPE_MAX_NESTING_DEPTH
}
