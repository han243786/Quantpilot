use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::{default_true, V4_REPRODUCIBILITY_CONTRACT_VERSION};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReproducibilityContract {
    #[serde(default = "default_reproducibility_contract_version")]
    pub schema_version: String,
    #[serde(default = "default_reproducibility_evidence")]
    pub required_evidence: Vec<RunEvidenceKind>,
    #[serde(default = "default_event_envelope_fields")]
    pub required_event_envelope_fields: Vec<EventEnvelopeField>,
    #[serde(default = "default_true")]
    pub key_decision_path_replay_required: bool,
    #[serde(default)]
    pub full_tick_replay_required: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RunEvidenceKind {
    StrategyRunId,
    EventSequence,
    InputSnapshotId,
    MemoryChangeLog,
    CapabilityHash,
    DeploymentRevision,
    OrderCapabilitySource,
    RiskDecisionEvidence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventEnvelopeField {
    EventId,
    EventType,
    EventTime,
    Source,
    Payload,
    Freshness,
    Sequence,
    Replayable,
}

impl Default for ReproducibilityContract {
    fn default() -> Self {
        Self {
            schema_version: V4_REPRODUCIBILITY_CONTRACT_VERSION.to_string(),
            required_evidence: default_reproducibility_evidence(),
            required_event_envelope_fields: default_event_envelope_fields(),
            key_decision_path_replay_required: true,
            full_tick_replay_required: false,
        }
    }
}

impl ReproducibilityContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_REPRODUCIBILITY_CONTRACT_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_REPRODUCIBILITY_CONTRACT_VERSION
            ));
        }

        let evidence = self
            .required_evidence
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for kind in default_reproducibility_evidence() {
            if !evidence.contains(&kind) {
                errors.push(format!("reproducibility evidence `{:?}` is required", kind));
            }
        }

        let fields = self
            .required_event_envelope_fields
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for field in default_event_envelope_fields() {
            if !fields.contains(&field) {
                errors.push(format!("event envelope field `{:?}` is required", field));
            }
        }

        if !self.key_decision_path_replay_required {
            errors.push("key decision path replay must be required".to_string());
        }
        if self.full_tick_replay_required {
            errors.push("full tick replay is reserved for a later phase".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn default_reproducibility_contract_version() -> String {
    V4_REPRODUCIBILITY_CONTRACT_VERSION.to_string()
}

fn default_reproducibility_evidence() -> Vec<RunEvidenceKind> {
    vec![
        RunEvidenceKind::StrategyRunId,
        RunEvidenceKind::EventSequence,
        RunEvidenceKind::InputSnapshotId,
        RunEvidenceKind::MemoryChangeLog,
        RunEvidenceKind::CapabilityHash,
        RunEvidenceKind::DeploymentRevision,
        RunEvidenceKind::OrderCapabilitySource,
        RunEvidenceKind::RiskDecisionEvidence,
    ]
}

fn default_event_envelope_fields() -> Vec<EventEnvelopeField> {
    vec![
        EventEnvelopeField::EventId,
        EventEnvelopeField::EventType,
        EventEnvelopeField::EventTime,
        EventEnvelopeField::Source,
        EventEnvelopeField::Payload,
        EventEnvelopeField::Freshness,
        EventEnvelopeField::Sequence,
        EventEnvelopeField::Replayable,
    ]
}
