use serde::{Deserialize, Serialize};

use super::{evidence_status, StrategyConfigEvidenceDiffStatus};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigEvidenceMetricsDiff {
    pub(crate) status: StrategyConfigEvidenceDiffStatus,
    #[serde(default)]
    pub(crate) fields: Vec<StrategyConfigEvidenceFieldDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigEvidenceFieldDiff {
    pub(crate) key: String,
    pub(crate) status: StrategyConfigEvidenceDiffStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) left_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) right_value: Option<String>,
}

pub(crate) fn compare_evidence_metrics(
    left: &qrpc_core::BacktestSummary,
    right: &qrpc_core::BacktestSummary,
) -> StrategyConfigEvidenceMetricsDiff {
    let fields = vec![
        evidence_field("step_count", left.step_count, right.step_count),
        evidence_field("trade_count", left.trade_count, right.trade_count),
        evidence_field(
            "total_return_ratio",
            stable_float(left.total_return_ratio),
            stable_float(right.total_return_ratio),
        ),
        evidence_field(
            "max_drawdown_ratio",
            stable_float(left.drawdown_analysis.max_drawdown_ratio),
            stable_float(right.drawdown_analysis.max_drawdown_ratio),
        ),
        evidence_field(
            "final_equity",
            stable_float(left.final_equity),
            stable_float(right.final_equity),
        ),
        evidence_field(
            "net_profit",
            stable_float(left.net_profit),
            stable_float(right.net_profit),
        ),
        evidence_field(
            "win_rate",
            stable_float(left.win_rate),
            stable_float(right.win_rate),
        ),
    ];
    let changed = fields
        .iter()
        .any(|field| field.status == StrategyConfigEvidenceDiffStatus::Different);
    StrategyConfigEvidenceMetricsDiff {
        status: evidence_status(changed),
        fields,
    }
}

fn evidence_field<T: ToString + PartialEq>(
    key: &str,
    left: T,
    right: T,
) -> StrategyConfigEvidenceFieldDiff {
    let changed = left != right;
    StrategyConfigEvidenceFieldDiff {
        key: key.to_string(),
        status: evidence_status(changed),
        left_value: Some(left.to_string()),
        right_value: Some(right.to_string()),
    }
}

fn stable_float(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.8}")
    } else {
        "nan".to_string()
    }
}
