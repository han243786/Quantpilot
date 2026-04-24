# Feature Additions Roadmap

## Purpose

This document focuses on what should be added after the beta base is made trustworthy.

The organizing rule is:

- add features that increase research value first
- add features that improve explainability second
- add features that expand breadth only after the foundation is stable

## Priority bands

### Highest-value additions

- backtest module
- runtime diagnostics panel
- execution and risk explanation
- graph version management
- data quality monitoring

### Mid-term additions

- multi-symbol strategies
- portfolio-level risk controls
- parameter sweep and experiment management
- event replay
- strategy template library

### Longer-term additions

- plugin marketplace
- multi-user collaboration
- live adapters
- account permission model
- audit and governance workflows

## Feature proposals by module

### 1. Backtest module

#### Why it matters

This adds more practical value than a fake live button.
It lets users test strategy logic, sizing rules, and event semantics under repeatable conditions.

#### Core tasks by module

##### `qrpc_runtime/src/sandbox.rs`

- add a scheduled replay runner
- support backtest clock progression by bar or event
- expose checkpoints and snapshots through a stable API

##### `qrpc_runtime/src/data_module.rs`

- add historical provider mode
- add replay dataset reader
- expose historical source metadata and replay progress

##### `src/main.rs`

- add endpoints for:
  - create backtest run
  - query backtest run status
  - fetch backtest metrics
  - fetch backtest trades and event timeline

##### `qrpc_core/src/lib.rs`

- add backtest config and backtest result structures
- add metrics for:
  - return
  - max drawdown
  - win rate
  - trade count
  - fee impact
  - slippage impact

##### Frontend

Files:

- `frontend/src/components/EventStreamPanel.jsx`
- new backtest result components
- `frontend/src/store/graphStore.js`

Tasks:

- create a backtest run panel
- show equity curve and drawdown
- show trade list and summary metrics
- allow jumping from trade records back to runtime events

### 2. Runtime diagnostics panel

#### Why it matters

This turns the system from a demo into a debuggable platform.

#### Current landed slice

The first roadmap slice is now landed on `2026-04-23`:

- frontend runtime diagnostics projection now lives in
  `frontend/src/utils/runtimeDiagnosticsProjection.js`
- the reusable runtime diagnostics surface now lives in
  `frontend/src/components/RuntimeDiagnosticsPanel.jsx`
- property-panel node runtime mode now embeds that panel so the selected node
  exposes recent node events, latest input/output snapshots, and latest
  warning/error context without leaving the inspector
- workspace diagnostics mode now embeds the same panel beside compile
  diagnostics so runtime and compile signals stay in one repair flow
- direct coverage now lives in
  `frontend/src/utils/runtimeDiagnosticsProjection.test.js` and
  `frontend/src/components/RuntimeDiagnosticsPanel.test.jsx`

This slice deliberately reuses the current runtime event log and node
`runtime_state` instead of inventing a second diagnostics protocol.
It is a frontend-first diagnostics promotion, not a claim that backend
node-detail payloads are finished.

The second roadmap slice is now also landed on `2026-04-23`:

- backend `RunDetailResponse` and `BacktestDetailResponse` now both expose a
  single `runtime_diagnostics` payload derived from the same runtime events /
  backtest event log already used elsewhere
- the structured detail payload now lives behind
  `src/runtime_diagnostics.rs` and is attached from
  `src/runtime_response_mapping.rs`
- frontend runtime-history loading now keeps that payload on
  `runtime.diagnostics`, and
  `frontend/src/utils/runtimeDiagnosticsProjection.js` now prefers the
  backend-projected diagnostics when detail payloads already include them
- direct coverage now also includes the run/backtest API contracts and the
  structured frontend projection fallback path
- the first stabilization follow-up on this lane is also landed: SSE tests now
  run through `frontend/src/store/graphStoreRuntimeTransport.js` instead of a
  direct global `EventSource` dependency, and backtest-detail artifact coverage
  now asserts stable `data-testid` structure instead of coupling to full
  user-facing copy
- the next hardening follow-up on this lane is also landed: the heaviest
  frontend copy-coupled tests now anchor on explicit section/card ownership and
  route-local test hooks across `ModuleSidebar`, `StrategyCanvas`,
  `StrategyBacktestsPage`, and `EventStreamPanel.backtestHistory` instead of
  binding to large Chinese copy blocks

This keeps a single protocol boundary: backend detail responses now shape the
same diagnostics facts that the frontend previously inferred ad hoc, without
creating a parallel diagnostics channel.

The remaining product decision on this lane is now also landed on `2026-04-23`:

- the research/event-stream surface now adopts the same node-focused filter
  model instead of inventing a second diagnostics view contract
- `frontend/src/components/EventStreamPanel.jsx` and
  `frontend/src/components/StrategyResearchConsole.jsx` now consume the same
  runtime-diagnostics node ownership already used by the property panel and
  workspace diagnostics tab
- event-stream filtering now follows one rule:
  - use the backend-projected default node when detail payloads provide one
  - otherwise follow the currently selected node when the operator has chosen
    one explicitly
  - otherwise keep the full event stream visible
- this closes the runtime-diagnostics batch as a single-protocol feature lane;
  future diagnostics work should reopen only if the existing
  `runtime_diagnostics` payload can no longer support the current surfaces

#### Core tasks by module

##### Frontend

Files:

- new `frontend/src/components/RuntimeDiagnosticsPanel.jsx`
- `frontend/src/components/StrategyCanvas.jsx`
- `frontend/src/components/PropertyPanel.jsx`

Tasks:

- show per-node latest input
- show per-node latest output
- show per-node runtime state and elapsed time
- show per-node last warning or error
- allow selecting a node and filtering the event stream to that node

Near-term remainder after the landed slice:

- tighten event-to-node input/output shaping for more event kinds
- only add backend node-detail payloads when the current frontend-first
  projection stops being enough

##### `src/main.rs`

- expose structured node-level diagnostics in run detail responses

##### `qrpc_core/src/lib.rs`

- add diagnostic payload types for node input, node output, and node error summaries

### 3. Execution and risk explanation

#### Why it matters

Users need to understand:

- why an order was generated
- why it was clamped
- why it was rejected
- why it filled or stayed open

#### Current landed slice

The first roadmap slice is now landed on `2026-04-23`:

- `qrpc_runtime/src/risk_checker.rs` now emits structured risk-decision
  explanation payloads instead of bare status snapshots
- `RiskDecisionProduced` events now carry:
  - `reason_text`
  - `limit_triggered`
  - `explanation_summary`
  - `sizing_mode`
  - `pre_risk`
  - `post_risk`
- `qrpc_runtime/src/execution_module.rs` now emits execution-plan explanation
  payloads that include:
  - `sizing_source`
  - `order_type_decision_reason`
  - `explanation_summary`
  - `order_previews`
- `qrpc_runtime/src/fill_engine.rs` now stamps order-lifecycle events with
  explicit `lifecycle_stage`, `reason_text`, and `explanation_summary`
  fields for accepted / open / partial / cancelled / rejected / completed
  paths
- backend event summarization now prefers those explanation fields through
  `src/runtime_event_projection.rs`
- frontend event-stream rendering now surfaces the same explanation facts in
  `frontend/src/components/EventStreamPanel.jsx` instead of inventing a second
  execution- or risk-specific protocol surface
- direct coverage now includes runtime crate event-payload tests plus
  `frontend/src/components/EventStreamPanel.executionExplanation.test.jsx`

This still keeps a single protocol boundary:

- risk, execution, fill, detail views, and event-stream rendering all consume
  the same enriched runtime-event payload family
- no separate explanation channel has been introduced
- future work on this lane should extend the current event payload contract
  before adding any second transport or explanation DTO family

The next follow-on slice is now also landed on `2026-04-23`:

- the same `runtime_diagnostics` payload now carries structured
  `explanation_summary`, `explanation_rows`, `risk_detail_rows`, and
  `order_detail_rows` under each `node_details` entry
- `frontend/src/components/RuntimeDiagnosticsPanel.jsx` now renders those rows
  as explicit order-detail and risk-detail sections in the property panel and
  workspace diagnostics tab
- `frontend/src/components/EventStreamPanel.jsx` now also reuses those same
  rows inside the selected run/backtest history cards, so order-history and
  risk-history surfaces stay on the same explanation contract
- `frontend/src/pages/BacktestDetailPage.jsx` now renders the same rows inside
  an explicit explanation section, so persisted detail views, history cards,
  and diagnostics panels all stay on one explanation payload family
- run-detail and backtest-detail responses still reuse the same
  `runtime_diagnostics` payload family; no order-detail or risk-detail DTO
  branch has been added beside it
- direct coverage now also asserts the API contract plus frontend projection,
  panel rendering, history rendering, and backtest-detail rendering for those
  detail rows

This execution-and-risk-explanation lane is now considered closed for the
current roadmap batch:

- event stream, diagnostics panel, history cards, and persisted backtest-detail
  surfaces all consume the same explanation payload family
- backend contract tests now pin `explanation_summary`, `explanation_rows`,
  `risk_detail_rows`, and `order_detail_rows` more tightly instead of only
  checking that some rows exist
- the next roadmap batch should now move to graph version management rather
  than widen explanation transport again

#### Core tasks by module

##### `qrpc_runtime/src/risk_checker.rs`

- enrich reason codes and reason text
- include pre-risk and post-risk sizing values
- include which limit triggered the clamp or reject

##### `qrpc_runtime/src/execution_module.rs`

- include order sizing source in plan events
- include order type decision reason in execution plan events

##### `qrpc_runtime/src/fill_engine.rs`

- add clear lifecycle events for:
  - accepted
  - opened
  - partially filled
  - cancelled
  - rejected
  - completed

##### Frontend

- show human-readable execution explanation beside each order
- show risk explanation beside each risk node event

Near-term remainder after the landed slice:

- propagate the same explanation payload into more order-detail and risk-detail
  surfaces when those views still infer semantics ad hoc
- tighten tests around runtime-event contract stability so explanation fields
  stay explicit when payload shapes evolve

### 4. Graph version management

#### Why it matters

Once users iterate on strategies, they need version history instead of only `latest.json`.

#### Current landed slice

The first graph-version-management slice is now landed on `2026-04-23`:

- backend graph storage now persists versioned copies under
  `storage/graphs/versions/<graph_id>/<version_id>.json|.qs` while still
  maintaining the current canonical graph and `latest.json`
- the graph API now exposes:
  - `GET /api/graphs/:graph_id/versions`
  - `GET /api/graphs/:graph_id/versions/:version_id`
  - `POST /api/graphs/:graph_id/versions/:version_id/restore`
- save responses now include the persisted `version_id`, and restore creates a
  new latest version instead of mutating the historical snapshot in place
- `frontend/src/store/graphStore.js` now keeps:
  - the current working draft in `graph`
  - the persisted version index in `graphVersions`
  - an explicit historical preview in `graphVersionPreview`
- `frontend/src/store/graphStorePersistenceActions.js` now supports loading a
  historical version preview without overwriting the draft and restoring a
  persisted version through the same graph API surface
- `frontend/src/pages/StrategyWorkspaceVersionHistoryCard.jsx` now exposes a
  persisted-version history panel inside the workspace overview tab so operators
  can preview first and restore second
- direct coverage now includes:
  - backend graph-version list/load/restore contract coverage
  - frontend store coverage for preview-vs-draft separation and restore reload

The first reliability-hardening follow-up on this lane is now also landed on
`2026-04-23`:

- high-frequency frontend tests such as `AssetCandlesPanel`,
  `DiagnosticsPanel`, and `StrategyWorkspacePage.codeMode` now anchor on
  route- or section-owned `data-testid` hooks instead of large user-facing copy
  blocks
- runtime-session SSE access remains constrained to
  `frontend/src/store/graphStoreRuntimeTransport.js`; tests now keep mocking
  that seam instead of touching the global `EventSource` directly
- graph-store tests that previously depended on ad hoc graph shapes now reuse
  the shared validated sample-graph fixture where possible, so version-history,
  diagnostics, and backtest-artifact tests start from the same normalized graph
  contract rather than hand-rolled node arrays

The next reliability-hardening follow-up on this lane is now also landed on
`2026-04-23`:

- `graphStore.startupRecovery.test.js` now also reuses the shared validated
  sample-graph fixture, including the non-runnable latest-graph recovery path
- `BacktestDetailPage.test.jsx`, `EventStreamPanel.backtestArtifacts.test.jsx`,
  and `PropertyPanel.compileSummary.test.jsx` now anchor on stable card or
  section boundaries instead of large copy-coupled assertions
- `StrategyCanvas.focus.test.jsx` now targets explicit focus, recommendation,
  and repair-path ownership hooks on canvas nodes instead of large visible-copy
  assertions across issue chips and recommendation text
- `EventStreamPanel.executionExplanation.test.jsx` now anchors execution / risk
  explanation checks on event-owned explanation hooks plus scoped row content,
  rather than asserting whole user-facing copy blocks
- `BacktestComparePage.test.jsx` and `PropertyPanel.layout.test.jsx` now anchor
  on page- and section-owned hooks instead of validating navigation and layout
  through dense title-copy assertions
- `EventStreamPanel.backtestArtifacts.test.jsx` and
  `PropertyPanel.compileSummary.test.jsx` now stay inside stable card
  boundaries and scoped metric content instead of leaning on dense heading-copy
  assertions
- `CompilePanel.integration.test.jsx`, `AssetCandlesPanel.test.jsx`, and
  `DiagnosticsPanel.test.jsx` now anchor on toolbar-, card-, and row-owned
  hooks instead of depending on dense visible-copy assertions
- `BacktestDetailPage.test.jsx`, `StrategyHubPage.test.jsx`,
  `StrategyResearchConsole.test.jsx`, and `ModuleSidebar.test.jsx` now anchor
  on page-, panel-, row-, and module-owned hooks instead of dense heading-copy
  and action-label assertions
- `TopToolbar.formalSourceMode.test.jsx`, `TopToolbar.capabilities.test.jsx`,
  `PropertyPanel.strategyIr.test.jsx`, and
  `EventStreamPanel.refreshFeedback.test.jsx` now anchor on toolbar-, panel-,
  and notice-owned hooks instead of copy-coupled button-name and banner-text
  assertions
- the remaining component/page `getByText` hotspots on
  `EventStreamPanel.backtestArtifacts`, `backtestAnalysisShared`, and
  `StrategyHubPage` are now also cleared; component/page tests on this lane no
  longer depend on dense `getByText` / `findByText` / `queryByText` lookups
- active-path history and hub activity surfaces are now also back on clean
  UTF-8 Chinese copy, and `StrategyHubPage.test.jsx` now waits on owned
  activity-card hooks instead of querying eager class counts before lazy
  sections resolve

This still keeps a single graph protocol boundary:

- the working draft remains the live in-memory graph
- persisted versions are snapshots behind the graph API, not a second graph DTO
- restore is modeled as a save of historical content into a new current version
  instead of a mutable edit to version history

The next graph-version-management slice is now also landed on `2026-04-23`:

- the graph API now also exposes
  `GET /api/graphs/:graph_id/versions/compare/:left_version_id/:right_version_id`
  on the same persisted-version route family instead of opening a second
  compare transport
- persisted version entries now carry optional `version_label` and `save_note`
  metadata in addition to timestamp ids
- `frontend/src/store/graphStorePersistenceActions.js` now supports:
  - saving persisted versions with label / note metadata
  - loading a structured persisted-version compare payload
- `frontend/src/pages/StrategyWorkspaceVersionHistoryCard.jsx` now exposes:
  - operator-authored version label and save-note inputs on the persisted-save
    path
  - a two-version compare queue
  - compare / diff sections for metadata, nodes, edges, and config
- direct coverage now includes:
  - `tests/api_graph_versions.rs`
  - `frontend/src/store/graphStore.versionHistory.test.js`
  - `frontend/src/pages/StrategyWorkspaceVersionHistoryCard.test.jsx`

Near-term remainder after the landed slice:

- decide whether version restore should expose a lightweight restore note in
  the UI history card or reuse the existing event-stream / activity surfaces
- once those are stable, move the next roadmap batch to data quality
  monitoring rather than widening version transport further

#### Core tasks by module

##### `src/main.rs`

- stop treating latest as the only meaningful graph state
- store graph versions with version ids and timestamps
- add endpoints for:
  - list graph versions
  - fetch graph version
  - compare versions
  - restore version

##### `frontend/src/store/graphStore.js`

- keep current working draft separate from persisted versions
- support loading a historical version without overwriting draft state

##### Frontend

- create graph version history panel
- add save note or version label support
- show diffs in metadata, nodes, edges, and config

### 5. Data quality monitoring

#### Why it matters

A quant platform becomes misleading if users cannot see stale or broken data conditions.

#### Current landed slice

The first data-quality-monitoring slice is now landed on `2026-04-23`:

- `qrpc_core/src/lib.rs` now exposes `DataQualitySnapshot` and a narrowed
  `SourceHealth` contract so normalized `KlineSeriesSnapshot` and
  `QuoteSnapshot` can carry:
  - `source_health`
  - `freshness_ms`
  - `stale_after_ms`
  - `gap_count`
  - `quality_flags`
- `qrpc_runtime/src/data_module.rs` now computes those fields for live data
  updates, attaches them to the normalized snapshots, and emits the same facts
  on `DataUpdated`, `RuntimeWarning`, and `RuntimeError` payloads instead of
  inventing a parallel quality channel
- `qrpc_runtime/src/sandbox.rs` now reuses the same data-quality helpers for
  replay / backtest collection so historical paths and live paths expose the
  same quality facts
- backend run-detail and backtest-detail responses now surface the same facts
  through `runtime_diagnostics.node_details[*].data_quality_rows`
- frontend runtime diagnostics now renders those rows directly in
  `RuntimeDiagnosticsPanel.jsx`, and runtime-history loading stores them on the
  same `runtime.diagnostics` contract rather than deriving a second payload
- data nodes now reuse the same metrics on the node card presentation so source
  health, freshness, and gap count can appear beside live data status
- `EventStreamPanel.jsx` now renders explicit source-health, freshness, gap,
  and quality-flag metadata on warning/error data events while continuing to
  reuse the same `explanation_summary` and runtime-event surface
- direct coverage now includes:
  - runtime module tests for delayed / missing data warnings
  - run-detail and backtest-detail API contract checks for
    `data_quality_rows`
  - frontend diagnostics-panel, projection, node-card, and event-panel tests

This lane still keeps a single data-quality surface:

- runtime events remain the source of truth for degraded data conditions
- `runtime_diagnostics` remains the structured detail surface
- frontend node cards, diagnostics panels, and event panels reuse those same
  facts instead of opening a second monitoring DTO family

The next narrow follow-up on this lane is now also landed on `2026-04-23`:

- selected run-history and backtest-history cards in `EventStreamPanel.jsx`
  now also reuse `runtime_diagnostics.node_details[*].data_quality_rows`
  through the same `HistoryExplanationCard` surface already used by risk/order
  explanations
- operators can now inspect source health, freshness, and gap rows from the
  currently loaded run/backtest detail without opening a second source-health
  history contract or inventing a dedicated data-quality DTO family
- direct frontend coverage now includes history-surface assertions for those
  selected run/backtest data-quality cards

The next narrow follow-up on this lane is now also landed on `2026-04-23`:

- `frontend/src/components/StrategyResearchConsole.jsx` now consumes the same
  data-quality facts through the existing diagnostics projection and renders a
  compact research-surface summary instead of deriving a second research-only
  quality model
- the research toolbar and summary cards now surface degraded-node counts,
  source-health status, and freshness / gap notes from the same
  `runtime_diagnostics` / runtime-event fact family already used by event
  stream, diagnostics, and node cards
- direct frontend coverage now includes `StrategyResearchConsole.test.jsx` for
  that summary surface

The closing narrow follow-up on this lane is now also landed on `2026-04-23`:

- data-node metric labels now surface source health, freshness versus stale
  threshold, source latency, and gap count directly in
  `frontend/src/nodes/nodeCardPresentation.js`
- this keeps freshness / latency / gap truth visible on the graph canvas
  itself without opening a second data-quality transport or a separate
  node-card-only DTO family
- direct frontend coverage now includes
  `frontend/src/nodes/nodeCardPresentation.test.js`

#### Next follow-up

- keep future work on this lane focused on data freshness / latency / gap
  truth rather than reopening graph-version or execution-explanation transport

## Mid-term feature additions

### 6. Multi-symbol strategy support

#### Current landed slice

The first narrow roadmap slice is now landed on `2026-04-23`:

- backend capability output now exposes a small supported multi-symbol market
  boundary of `BTCUSDT`, `ETHUSDT`, and `SOLUSDT`
- the frontend data-module `instrument` options now derive from that same
  capability surface instead of remaining hardcoded to `BTCUSDT`
- runtime capability validation still rejects symbols outside the declared beta
  profile, so this widens the supported boundary honestly instead of bypassing
  capability enforcement
- direct coverage now includes the capability snapshot plus frontend
  module-option propagation tests
- the next narrow slice on the same day is also landed:
  the weighted-agent graph/runtime config surface now exposes
  `rebalance_symbols`, `rebalance_schedule`, `rebalance_allocation_kind`,
  `rebalance_rank_method`, `rebalance_score_normalize`, and
  `rebalance_target_weights`; frontend graph compile lowers that same config
  into `portfolio_rebalance` core IR when enabled; frontend runtime mapping
  threads the same fields into backend runtime config; and runtime capability
  validation rejects unsupported rebalance symbols or malformed target weights
  instead of opening a second multi-symbol transport
- the active frontend graph/workspace surfaces are now hardened on top of that
  same slice: hub roster/activity views render normal UTF-8 copy again, lazy
  section tests wait on owned hooks instead of class timing, and the feature
  remains on the same graph/runtime configuration surface under full-suite
  regression checks

#### Module tasks

##### `qrpc_core/src/lib.rs`

- expand symbol abstractions beyond a single hardcoded symbol

##### `qrpc_runtime`

- thread symbol scope through data, intent, agent, risk, execution, and fill modules

##### Frontend

- keep multi-symbol graph configuration and symbol universe configuration on
  the same weighted-agent / runtime-config surface instead of opening a second
  frontend-only universe contract

### 7. Portfolio-level risk controls

#### Module tasks

##### `qrpc_runtime/src/risk_checker.rs`

- concentration limits are now landed on the existing `RiskPolicy` /
  `RiskDecisionProduced` surface
- per-symbol and portfolio net exposure controls are now also landed on that
  same `RiskPolicy` / `RiskDecisionProduced` surface
- daily loss limits remain deferred until the runtime carries a trustworthy
  session/day baseline instead of inferring one from current portfolio state
- add correlation-aware or grouped risk in later stages

##### Frontend

- portfolio risk policy is now exposed in the global risk node config without
  defaulting new optional guards into existing strategies
- runtime diagnostics now render concentration / per-symbol net exposure /
  portfolio net exposure rows from the same risk payload instead of a second
  frontend-only detail contract
- the next narrow follow-up on this lane is now also landed:
  risk-node metric labels and event-stream risk rows now surface the same
  active guard truth (`limit_triggered`, concentration, per-symbol net
  exposure, portfolio net exposure) from the existing runtime-event /
  `runtime_diagnostics` payload family instead of inventing a second
  frontend-only risk summary model

### 8. Parameter sweep and experiment management

#### Current landed slice

The first narrow slice on this lane is now landed on `2026-04-24`:

- backend experiment endpoints now live on the existing runtime route family:
  - `POST /api/runtime/experiments/backtest-sweep`
  - `GET /api/runtime/experiments`
  - `GET /api/runtime/experiments/:experiment_id`
- experiment definitions and result summaries are now persisted under
  `storage/experiments/`, while each variant still reuses the normal backtest
  record path instead of opening a second result transport
- the current sweep surface is intentionally narrow:
  - `fee_bps`
  - `slippage_bps`
  - `latency_ms`
- frontend workspace overview now exposes a single experiment card that can:
  - submit a named sweep
  - list persisted experiments for the current graph
  - open experiment results
  - jump from a variant back into the existing backtest-detail surface
- this slice still keeps a single protocol boundary:
  experiment variants are summaries over existing backtest executions, not a
  second experiment-only runtime or DSL family
- the immediate hardening follow-up on this lane is also now landed:
  runtime-history warming now includes experiment history on the same sidebar
  load path, experiment-card tests are wrapped cleanly without `act(...)`
  warning noise, and the shared runtime-history failure-copy path now uses
  normal UTF-8 Chinese fallback text
- the current honest stop-line on this lane is explicit:
  this is not yet a generic experiment scheduler or a broad optimization
  system; it is a narrow execution-assumptions sweep over the existing
  backtest transport

#### Module tasks

##### `src/main.rs`

- add experiment run endpoints
- store experiment definitions and result summaries

##### `qrpc_runtime`

- support repeated run orchestration over parameter grids

##### Frontend

- add experiment setup UI
- compare result tables across runs

### 9. Event replay

#### Current landed slice

The first narrow slice on this lane is now landed on `2026-04-24`:

- backend runtime routes now expose:
  - `GET /api/runtime/runs/:run_id/replay`
  - `GET /api/runtime/backtests/:backtest_id/replay`
- replay responses reuse the existing persisted run/backtest records and wrap
  them as a paginated ordered timeline with:
  - `cursor`
  - `limit`
  - `window_end`
  - `previous_cursor`
  - `next_cursor`
  - checkpoint labels
  - stable `sequence_no` ordering
- backtest replay prefers the persisted backtest event-log artifact when
  present and falls back to the stored runtime-event list otherwise, so no
  second replay transport or replay-only DTO family was introduced
- `frontend/src/components/EventStreamPanel.jsx` now embeds a narrow replay
  scrubber through `frontend/src/components/EventReplaySection.jsx`
- that scrubber reuses the same selected persisted run/backtest context from
  the history sidebar and exposes:
  - on-demand loading
  - page-size selection
  - previous/next paging
  - checkpoint jumps
  - runtime events, fills, and account snapshot chips
- direct coverage now includes:
  - `tests/api_run.rs`
  - `tests/api_backtest.rs`
  - `frontend/src/components/EventReplaySection.test.jsx`

This lane still keeps a single replay surface:

- persisted runtime/backtest records remain the fact source
- replay is a paginated projection over that existing history, not a second
  runtime channel
- the current honest stop-line is narrow:
  page and checkpoint replay is landed, but full timeline scrubbing across
  reconstructed account state or a generalized time-travel debugger is not

#### Module tasks

##### `src/main.rs`

- persist timeline events with ordering guarantees
- expose replay endpoints with pagination or checkpoints

##### Frontend

- add a replay scrubber for runtime events, fills, and account state

### 10. Strategy template library

#### Current landed slice

The first narrow slice on this lane is now landed on `2026-04-24`:

- the frontend now owns a canonical local starter-template list in
  `frontend/src/templates/strategyTemplates.js`
- the current template browser now lives inside the strategy hub body instead
  of opening a second backend template transport
- the current landed starter set is intentionally narrow:
  - `dual_ma_trend`
  - `rsi_reversion`
  - `multi_symbol_rebalance`
- loading a template now replaces the current in-memory working draft through
  the existing graph-store editor surface and then opens the workspace for that
  draft
- template loading does not auto-persist a saved graph version and does not
  introduce a second graph DTO family beside the existing graph/runtime surface
- canonical template documentation now lives in
  `markdown/guides/guide-strategy-template-library.md` and is tied to the
  modules and symbols actually supported by the current product boundary

This lane still keeps a single graph/runtime surface:

- templates are starter graphs, not a second protocol
- persisted graphs still live behind the normal graph API and save flow
- future widening should happen only if the current local canonical list stops
  matching the supported product surface

#### Module tasks

##### Frontend

- add template browser and starter graphs
- support loading templates into the editor safely

##### Docs

- maintain canonical template docs tied to actual supported modules

## Longer-term feature additions

### 11. Plugin marketplace

#### Current landed slice

The first narrow slice on this lane is now landed on `2026-04-24`:

- `qrpc_core` now owns the canonical plugin manifest, capability-contract, and
  registry contracts instead of keeping a second copy in the app crate
- `qrpc_runtime` now exposes a narrow runtime plugin registry with lifecycle
  boundaries (`registered`, `active`, `stopped`, `faulted`) for external
  provider registration
- `frontend/src/modules/moduleRegistry.js` now validates external plugin
  metadata and manifest shape before registration and keeps unsupported entries
  in a marketplace catalog instead of silently merging them into the active
  module set
- this slice is still local-metadata only: it does not add remote install,
  dependency resolution, signatures, or a second frontend/backend plugin
  transport

This lane still keeps a single graph/runtime surface:

- builtin modules remain the active editor/runtime truth
- external plugin metadata is only admitted through the same module-registry and
  plugin-manifest contracts
- runtime provider activation stays behind lifecycle boundaries instead of
  becoming a second execution path

#### Module tasks

##### `frontend/src/modules/moduleRegistry.js`

- load external module metadata
- validate plugin schemas before registration

##### `qrpc_runtime`

- register external providers with lifecycle boundaries

##### `qrpc_core`

- define plugin manifest and capability contracts

### 12. Multi-user collaboration

Current landed slice on `2026-04-24`:

- graph save / restore and runtime run / backtest / experiment creation now
  accept a narrow actor identity on the existing graph/runtime request surface
- graph mutation and runtime creation now enforce a narrow owner-or-editor
  permission check against persisted graph collaboration metadata instead of
  silently trusting any caller
- graph audit history now persists as append-only graph-scoped records under
  the same app storage family and is exposed through the graph API instead of a
  second audit transport
- the workspace overview now renders owner, editors, last-saved / last-run
  actor context, and recent audit entries on the same working-draft surface
- this is still a local-actor collaboration slice, not a full auth system,
  shared-session coordination layer, or a second permissions contract family

Current freeze decision on `2026-04-24`:

- this lane is now intentionally frozen at the current local-actor slice
  because there is no short-term launch requirement for a real account system
- do not widen it into:
  - remote auth
  - shared presence
  - broader account-permission expansion
  - a second collaboration or permissions transport
- reopen this lane only through a new explicit product decision

#### Module tasks

##### Backend

- add user identity and actor metadata to graph and run records

##### Frontend

- show owner, editor, and last-run actor info

### 13. Live adapters

#### Module tasks

##### `qrpc_runtime/src/execution_module.rs`

- separate paper execution from exchange execution adapters

##### `qrpc_core/src/lib.rs`

- add account slot, adapter capability, and execution environment contracts

##### Backend

- add guarded execution mode routing, not just a frontend toggle

### 14. Account permissions and audit

Current freeze decision on `2026-04-24`:

- the current roadmap batch keeps the landed owner-or-editor checks and
  graph-scoped audit history, but freezes further account-permission growth
- treat richer account-permission and audit-governance workflows as deferred
  until there is a real launch requirement for user accounts
- do not widen this lane into a second permissions DTO family while it is
  frozen

#### Module tasks

##### Backend

- add permission checks around graph mutation and run creation
- persist audit trails

##### Frontend

- surface audit history and permissions in editor and run pages

## Suggested feature rollout order

### Step 1

- backtest module
- runtime diagnostics panel
- execution and risk explanation

### Step 2

- graph version management
- data quality monitoring
- multi-symbol support

### Step 3

- portfolio-level risk controls
- parameter sweep and experiment management
- event replay

### Step 4

- strategy template library
- plugin marketplace
- multi-user and audit features

## Product rule for all new features

Before any new feature is exposed in the frontend:

- backend support must exist
- protocol shape must be stable
- error behavior must be defined
- tests must cover the feature path
- diagnostics must explain failure cases

If those conditions are not met, the feature should stay hidden until it is real.
