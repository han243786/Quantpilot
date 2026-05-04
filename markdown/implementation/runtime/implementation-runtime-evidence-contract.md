# Runtime Evidence Contract

## Purpose

This document owns the active evidence-surface contracts introduced in Block 2
of the v0.2.0 upgrade. It is the field-level source of truth for timeline,
replay, compact evidence, retained key indexes, and report lifecycle payloads.

The evidence surface must not create a second runtime fact model. It projects
governed runtime/backtest events from the Runtime Governance Contract and keeps
the event envelope as the source of sequence, stage, retention, and governance
identity.

## Ownership

| Contract | Backend owner | Frontend owner | Persisted | Update rule |
|---|---|---|---|---|
| Timeline item | `runtime_event_projection` and `runtime_response_mapping` | `runtimeTimeline` reader and `GovernedTimelinePanel` | No, projected from governed records | Additive fields require API contract snapshot update and UI reader fallback. Breaking renames require a v2 contract. |
| Replay window | `runtime_api` replay handlers | `EventReplaySection` and `graphStoreRuntimeHistoryApi` | No, paged from persisted/current source events | Cursor, filter, or sequence semantics must update replay tests and contract snapshot. |
| Retained key-event index | `runtime_response_mapping` compact/key projection | `runtimeTimeline` reader | No, projected from detail source | Retention policy changes must preserve all `retention_class=key` and system governance events. |
| Compact evidence | `runtime_response_mapping` compaction projection | `runtimeTimeline`, `runtimeEvidenceSummary`, report UI | No, projected from detail source | Dropping policy changes must update policy version and snapshot fixture. |
| Report lifecycle record | `runtime_api` report store and source materialization | `RuntimeReportPanel` | Yes, report metadata store | New lifecycle fields must be source-linked, reload-safe, and covered by snapshot tests. |
| Report artifact export | `runtime_response_mapping` report artifact projection | export/reveal links | Derived from report metadata | Export payload must never copy raw event logs; it links to source identity, range, governance, policy, digest, and loading strategy. |
| Evidence health | `runtime_api` evidence health handler and in-memory evidence metrics | Operations/status UI or smoke checks | No, runtime counters only | New counters must be additive and must not alter user-visible report/timeline behavior. |
| Evidence cleanup | `runtime_persistence` cleanup policy and cleanup handler | Manual operations action only | Applies only to transient generation outputs | Cleanup must never delete persisted report JSON records or saved run/backtest/experiment artifacts. |

## Timeline Item

Every detail, replay, compact, and report input path must use the same timeline
item shape:

- `timeline_item_version`: contract version, currently `1`
- `event_id` and `event_type`: copied from the governed runtime event
- `sequence_no`: event envelope sequence number; primary replay cursor
- `occurred_at_ms` and `ingested_at_ms`: envelope timing fields
- `stage`: typed envelope stage such as `system`, `data`, `risk`, `execution`
- `retention_class`: typed retention class such as `key`, `summary`, `debug`
- `severity`, `module_key`, `node_id`, `summary`, `reason_code`
- `governance`: `capability_hash`, `deployment_revision`, `strategy_version`,
  and `parameter_version`
- `payload_version` and `compactability`

Timeline readers may repair legacy missing values only through restrictive
defaults. New code must not infer stage or governance from display text.

## Replay Window

Replay responses page over the timeline source by sequence, not by wall-clock
time. The active fields are:

- source identity: `kind`, `record_id`, `graph_id`
- source counts: `source_event_count`, `total_events`
- cursors: `cursor`, `sequence_cursor`, `previous_*`, `next_*`, `window_end`
- filters: `stage`, `severity`, `retention_class`, `module_key`, `key_only`
- evidence arrays: legacy `events` wrapper and governed `timeline`
- summary context: `fill_event_count`, `account`, `checkpoints`

Frontend replay controls must consume returned cursor metadata. They must not
derive page boundaries from locally sliced arrays when backend sequence metadata
is available.

## Retained Key-Event Index

The retained key-event index is a compact index over the shared timeline:

- `index_version`
- `policy_version`
- `source_event_count`
- `retained_event_count`
- `key_event_count`
- `system_event_count`
- `entries`

The policy keeps every `retention_class=key` item and system governance events
such as `CapabilitySnapshotTaken` and `SecurityViolationDetected`. It may drop
summary/debug events from compact paths, but it must keep enough sequence
metadata to re-open a detailed replay window later.

## Compact Evidence

Compact evidence is the preferred large-log review input. It exposes:

- `projection_version`
- `policy_version`
- `source_event_count`, `retained_event_count`, `dropped_event_count`
- `dropped_by_retention` and `dropped_by_stage`
- `key_event_count` and `system_event_count`
- `governance`
- `entries`

UI summary cards and report generation must read compact evidence first. If
compact entries are unavailable, they may fall back to the current detail
window and mark that a detail window is required.

## Report Lifecycle

Report records are persisted metadata, not copied logs. The active lifecycle
fields are:

- `report_id`
- `source_kind`, `source_id`, `graph_id`
- `status`: `requested`, `generating`, `ready`, `failed`, `expired`,
  `source_changed`
- `source_sequence_range`
- `source_event_count`, `retained_event_count`
- `governance`
- `generation_policy`
- `artifacts`
- `failure_reason` for compatibility
- `failure`: structured `reason_code`, `message`, and `retry_eligible`
- `created_at_ms`, `updated_at_ms`

Report list, detail, and export paths must materialize records against the
current saved source before claiming `ready`. If source graph id, sequence
range, source/retained counts, governance identity, or generation policy no
longer match, the report becomes `source_changed` and ready artifacts are
removed from the returned record.

## Report Artifact Export

Report exports are deterministic artifacts derived from report metadata:

- `schema_version`: `quantpilot/evidence-report-artifact/v1`
- source identity and lifecycle fields from the report record
- `evidence_digest`
- `loading_strategy`: `primary_source`, `source_event_count`,
  `retained_event_count`, `requires_detail_window`
- `sections`

Export payloads must not include raw `events` or compact `entries`. The digest
and source metadata are the trace back to the exact evidence chain.

## Evidence Health

The evidence health endpoint is `GET /api/runtime/evidence/health`. It exposes
operational counters and policy metadata without changing the user workflow.

Active fields:

- `status`: currently `ok`
- `metrics.report_generation_count`
- `metrics.report_generation_failure_count`
- `metrics.report_source_changed_count`
- `metrics.replay_page_count`
- `metrics.replay_page_latency_total_ms`
- `metrics.replay_page_latency_avg_ms`
- `metrics.compact_projection_source_event_count_total`
- `metrics.compact_projection_retained_event_count_total`
- `metrics.compact_detail_window_required_count`
- `persisted_report_count`
- `report_status_counts`
- `cleanup_policy`

The metrics are in-memory runtime counters. They are for health checks and
smoke diagnostics, not billing, compliance totals, or persisted analytics.

## Evidence Cleanup

The evidence cleanup endpoint is `POST /api/runtime/evidence/cleanup`. It
removes only transient report-generation outputs under the report store whose
file or directory name starts with one of the cleanup policy prefixes:

- `report-generation-tmp-`
- `report-generation-partial-`

The default TTL is 24 hours. Tests may pass `max_age_ms` to exercise the policy
deterministically.

Cleanup must retain:

- persisted report record JSON files
- saved run records
- saved backtest artifact directories
- saved experiment records
- compact evidence projections, because they are derived from source evidence
  rather than stored as standalone cache entries

The cleanup response reports the policy, removed transient output count, and
retained report record count.

## Change Checklist

Before changing any evidence contract field:

1. Update this document and the v0.2.0 worklist.
2. Update backend projection or materialization tests.
3. Update the API contract snapshot fixture.
4. Update frontend reader/UI tests when display semantics change.
5. Run UTF-8 Markdown checks for Chinese UI labels and documentation.
