use crate::*;

pub(super) fn build_signature_input(
    capability_hash: &str,
    strategy_version: &str,
    parameter_version: &str,
    core_ir_digest: &str,
    event_slice_bounds: &EventSliceBounds,
    created_at_ms: u64,
) -> serde_json::Value {
    json!({
        "capability_hash": capability_hash,
        "strategy_version": strategy_version,
        "parameter_version": parameter_version,
        "core_ir_digest": core_ir_digest,
        "event_slice_bounds": {
            "from_event_id": &event_slice_bounds.from_event_id,
            "to_event_id": &event_slice_bounds.to_event_id,
            "from_sequence": event_slice_bounds.from_sequence,
            "to_sequence": event_slice_bounds.to_sequence,
            "event_count": event_slice_bounds.event_count,
        },
        "created_at_ms": created_at_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_input_preserves_field_shape() {
        let bounds = EventSliceBounds {
            from_event_id: "evt-1".to_string(),
            to_event_id: "evt-9".to_string(),
            from_sequence: 1,
            to_sequence: 9,
            event_count: 9,
        };

        let input = build_signature_input(
            "sha256:cap",
            "strategy-v1",
            "params-v1",
            "sha256:ir",
            &bounds,
            42,
        );

        assert_eq!(input["capability_hash"], "sha256:cap");
        assert_eq!(input["strategy_version"], "strategy-v1");
        assert_eq!(input["parameter_version"], "params-v1");
        assert_eq!(input["core_ir_digest"], "sha256:ir");
        assert_eq!(input["created_at_ms"], 42);
        assert_eq!(input["event_slice_bounds"]["from_event_id"], "evt-1");
        assert_eq!(input["event_slice_bounds"]["to_event_id"], "evt-9");
        assert_eq!(input["event_slice_bounds"]["from_sequence"], 1);
        assert_eq!(input["event_slice_bounds"]["to_sequence"], 9);
        assert_eq!(input["event_slice_bounds"]["event_count"], 9);
    }
}
