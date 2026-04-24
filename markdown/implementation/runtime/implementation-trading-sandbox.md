# Trading Sandbox Implementation

This is the highest-priority implementation guide for the current stage of QuantPilot.

The goal is not to add more disconnected features.
The goal is to consolidate the current runtime, fill, risk, and data flow into one unified trading sandbox.

For the repeatability boundary used by CI, replay, and service-level tests, see [implementation-test-mode.md](D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-mode.md).

## Target modes

The sandbox should support three modes over time:

1. `RealTimeSandbox`
2. `FastBacktestSandbox`
3. `AccurateBacktestSandbox`

All of them should share:

- `NormalizedMarketData`
- `ExecutionPlan`
- `RiskChecker`
- `FillEngine`
- `RuntimeEvent`

## Existing base

The repository already contains most of the raw ingredients:

- `qrpc_core` has the core protocol objects
- `qrpc_compiler` already validates the main chain
- `qrpc_runtime` already has runtime coordination, fill handling, and portfolio refresh
- backend API already supports test runs and event streaming
- frontend already compiles graphs and consumes runtime events

Current frontend diagnostics promotion also reuses this base:

- runtime event log plus node `runtime_state` now feed
  `frontend/src/components/RuntimeDiagnosticsPanel.jsx`
- the selected-node diagnostics surface now appears in both the property panel
  and workspace diagnostics mode
- run-detail and backtest-detail responses now also expose a structured
  `runtime_diagnostics` payload derived from the same runtime event log /
  backtest event-log artifact
- this is still one runtime diagnostics protocol, not a parallel channel:
  frontend detail views now prefer the backend-projected diagnostics payload
  and only fall back to local event projection when no detail payload exists
- the same single diagnostics contract now also drives the research /
  event-stream surface: `EventStreamPanel.jsx` and
  `StrategyResearchConsole.jsx` now follow the backend-projected default node
  when detail payloads provide one, otherwise follow the explicitly selected
  node, and otherwise keep the full event stream visible instead of inventing
  a second diagnostics-specific filter model
- the immediate hardening pass on top of that contract is now also landed:
  runtime-session SSE tests depend on
  `frontend/src/store/graphStoreRuntimeTransport.js` instead of the raw global
  `EventSource`, and backtest-detail diagnostics coverage now anchors on stable
  section/card structure rather than full user-facing copy
- the next roadmap slice on top of the same runtime-event boundary is now also
  landed: risk decisions, execution plans, and fill-engine lifecycle events
  now expose structured explanation fields such as `reason_text`,
  `limit_triggered`, `sizing_source`, `order_type_decision_reason`,
  `lifecycle_stage`, and `explanation_summary`, and the event-stream surface
  renders those same fields directly instead of inventing a second execution-
  or risk-explanation protocol
- the follow-on detail slice is now also landed on that same contract:
  `runtime_diagnostics.node_details` now carries `explanation_rows`,
  `risk_detail_rows`, and `order_detail_rows`, so property-panel and workspace
  diagnostics surfaces can render order-detail and risk-detail explanations
  without introducing a second response family beside `runtime_diagnostics`
- the next surface follow-up is also landed on that same contract:
  selected run/backtest history cards in `EventStreamPanel.jsx` now reuse the
  same explanation rows, so event history, order history, and risk history all
  stay on one diagnostics/explanation payload family
- the persisted-detail follow-up is also landed on that same contract:
  `BacktestDetailPage.jsx` now renders the same explanation rows inside an
  explicit explanation section, so diagnostics, history, and detail views all
  stay on one payload family instead of forking a second explanation response
  shape
- the next narrow runtime follow-up is now also landed on that same contract:
  concentration, per-symbol-net-exposure, and portfolio-net-exposure guards now
  flow through `RiskPolicy`, `RiskDecisionProduced`, and
  `runtime_diagnostics.node_details.risk_detail_rows`, so the risk checker,
  runtime detail surfaces, and frontend diagnostics all stay on one payload
  family instead of opening a second portfolio-risk protocol
- the current honest stop-line on that lane is now explicit:
  daily-loss limits remain deferred until the sandbox carries a trustworthy
  session/day loss baseline, so this slice does not pretend current portfolio
  mark-to-market is a valid day-loss contract
- the first narrow parameter-sweep follow-up is now also landed on the same
  sandbox/backtest contract:
  the runtime route family now supports a persisted execution-assumptions sweep
  over `fee_bps`, `slippage_bps`, and `latency_ms`, but each variant still
  executes through the normal backtest path and surfaces through the existing
  backtest-detail payload family instead of introducing a second experiment-only
  runtime transport

So this is a consolidation step, not a rewrite-from-zero step.

## Suggested boundary

```rust
trait Sandbox {
    fn start(&mut self) -> anyhow::Result<()>;
    fn stop(&mut self) -> anyhow::Result<()>;
    fn submit_execution_plan(&mut self, plan: ExecutionPlan) -> anyhow::Result<FillResult>;
    fn on_market_data(&mut self, data: NormalizedMarketData) -> anyhow::Result<Vec<RuntimeEvent>>;
    fn snapshot(&self) -> anyhow::Result<SandboxSnapshot>;
}
```

Suggested responsibilities:

- `Sandbox`: mode boundary
- `RiskChecker`: pre-execution checks
- `FillEngine`: fill and matching behavior
- `MarketDataFeed`: real-time or historical input source
- `EventLog`: structured events
- `SnapshotStore`: recovery and replay support

## Near-term tasks

### Task 1: sandbox abstraction

Goal:

- stop treating `RuntimeCoordinator` as the only runtime mode
- introduce one explicit runtime boundary for different modes

Suggested location:

- `qrpc_runtime/src/lib.rs`
- `qrpc_runtime/src/sandbox.rs`

### Task 2: fill engine boundary

Goal:

- stabilize `ExecutionPlan + MarketState -> FillResult`
- keep matching logic separate from higher-level runtime orchestration

Current base already includes:

- market orders
- limit orders
- IOC
- FOK
- resting continuation
- idempotent handling

Next steps:

- define `MarketState` more explicitly
- separate slippage model hooks
- leave room for L1 now and L2/L3 later
- make account update boundaries clearer

### Task 3: risk checker boundary

Goal:

- pull risk checks into a clearer module boundary
- make all execution plans pass through one consistent risk gate

Expected checks include:

- position limits
- leverage limits
- order frequency limits
- invalid action rejection

### Task 4: unified data input

Goal:

- make both real-time and historical paths produce the same normalized market data contract
- keep upper layers independent from raw source format

Current landed slice on `2026-04-23`:

- live data normalization now attaches `DataQualitySnapshot` to normalized
  kline and quote snapshots
- the live data module now emits `source_health`, `freshness_ms`,
  `stale_after_ms`, `gap_count`, `quality_flags`, and
  `explanation_summary` on `DataUpdated`, `RuntimeWarning`, and
  `RuntimeError`
- frontend research surfaces now also consume that same fact family through the
  shared runtime-diagnostics projection, so research summary cards do not fork
  a separate data-quality protocol beside diagnostics and event-stream views
- graph node cards now also consume that same fact family through
  `frontend/src/nodes/nodeCardPresentation.js`, so source health, freshness
  versus stale threshold, source latency, and gap count stay visible on the
  canvas without adding a second node-card transport
- the replay / fast-backtest path now reuses the same helper chain, so
  historical collection and real-time collection expose the same quality facts
- higher layers now consume those same facts through runtime events and
  `runtime_diagnostics`, rather than through a second data-quality-specific
  transport

### Task 5: logs, snapshots, and recovery

Goal:

- support replay, audit, and recovery using stable runtime outputs

Minimum expected outputs:

- structured runtime events
- account snapshots
- mode snapshots
- restore entry points

Current landed replay follow-up on `2026-04-24`:

- persisted run and backtest records now expose paginated replay projections
  through the backend runtime API
- replay ordering is explicit through stable `sequence_no` values and
  checkpoint labels instead of implicit array position assumptions
- the frontend event-stream sidebar now consumes that same persisted timeline
  through a narrow replay scrubber rather than a second event transport
- this remains a recovery/audit projection over existing runtime outputs, not a
  second runtime mode or a replay-only contract family

## Delivery order

### Stage 1: real-time sandbox

- reuse current event streaming path
- reuse current fill logic
- reuse current risk logic

### Stage 2: fast backtest sandbox

- feed K-line or L1 data
- use simplified matching logic
- keep runs deterministic enough for repeat execution

Deterministic repeatability at this stage should come from an explicit test mode contract rather than from accidental behavior.
If a replay or regression workflow needs fixed ordering, fixed clock behavior, or seed control, those assumptions should be declared through test mode instead of hidden in sandbox internals.

Current Week 1 implementation now exposes this through:

- `DeterministicTestMode`
- `DeterministicClockMode`
- `DeterministicEventOrdering`
- `DeterministicParallelismPolicy`

and keeps the selected configuration on both `RealTimeSandbox` and `FastBacktestSandbox`.

### Stage 3: accurate sandbox

- add L2/L3 data
- add queue position
- add latency model
- add higher-fidelity matching

## Relationship to pluginization

Sandbox work is the prerequisite for pluginization.

Do not freeze plugin contracts before the following boundaries are stable:

- `Sandbox`
- `RiskChecker`
- `FillEngine`
- `NormalizedMarketData`
- `RuntimeEvent`

## Deterministic test mode boundary

The sandbox should support deterministic test mode as a testing and replay aid, not as a separate product mode.

Near-term expectations:

- the same fixed input bundle should yield stable event ordering
- replay and backtest smoke runs should not depend on wall-clock timing variance
- capability gates should stay identical to the normal beta profile
- test-only controls should enter through explicit runtime configuration

Current Week 1 implementation also exports `RuntimeSupportBoundary` from `qrpc_runtime`, so the backend capability response and compile gating can consume the same runtime mode and execution-module boundary instead of maintaining a second copy.

This is especially important for Week 2 service-level API tests and frontend E2E smoke coverage.

