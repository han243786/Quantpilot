# Beta Optimization Roadmap

## Purpose

This document turns the beta roadmap into concrete module-level tasks.

The goal is not to add more surface area first.
The goal is to make the current product honest, stable, debuggable, and safe enough for a real beta workflow.

## Scope boundary

The beta target in this document means:

- paper trading only
- sandbox runtime only
- only expose symbols, exchanges, modes, and modules that are truly implemented
- every user-visible parameter must affect backend behavior
- every run must be explainable after the fact

## Module map

The main implementation surfaces in the current repository are:

- backend API and graph persistence: `src/main.rs`
- shared protocol types: `qrpc_core/src/lib.rs`
- runtime coordinator and sandbox: `qrpc_runtime/src/lib.rs`, `qrpc_runtime/src/sandbox.rs`
- runtime builtins:
  - data: `qrpc_runtime/src/data_module.rs`
  - intent: `qrpc_runtime/src/intent_module.rs`
  - agent: `qrpc_runtime/src/agent_module.rs`
  - risk: `qrpc_runtime/src/risk_checker.rs`
  - execution: `qrpc_runtime/src/execution_module.rs`
  - fill engine: `qrpc_runtime/src/fill_engine.rs`
- frontend graph compile and validation:
  - `frontend/src/graph/compileGraph.js`
  - `frontend/src/graph/validation.js`
  - `frontend/src/graph/quantscript.js`
- frontend module registry and node definitions:
  - `frontend/src/modules/builtinModules.js`
  - `frontend/src/modules/moduleRegistry.js`
- frontend runtime UI:
  - `frontend/src/components/EventStreamPanel.jsx`
  - `frontend/src/components/PropertyPanel.jsx`
  - `frontend/src/components/StrategyCanvas.jsx`
  - `frontend/src/store/graphStore.js`

## P0: Make the product tell the truth

### P0.1 Frontend and backend capability alignment

#### `frontend/src/modules/builtinModules.js`

- remove or hide `builtin.execution.live` until a real live adapter exists
- remove unsupported symbols such as `ETHUSDT` until backend symbol support exists
- remove config fields that are currently ignored by backend logic
- mark all remaining config fields with actual backend meaning
- align default values with runtime defaults to avoid UI/backend drift

#### `frontend/src/graph/compileGraph.js`

- stop generating runtime configs that imply unsupported capabilities
- block compile output when unsupported modules, modes, exchanges, or symbols are present
- emit a real `compile_summary` instead of defaulting to success
- include explicit backend support checks in compile output

#### `src/main.rs`

- reject unsupported execution module keys instead of silently ignoring them
- reject unsupported runtime modes instead of carrying them as metadata only
- reject unsupported symbols and exchanges with structured validation errors
- map only supported frontend modules into runtime config
- return compile and validation errors in a stable response shape

#### `qrpc_core/src/lib.rs`

- make supported enum values reflect actual backend support, or add a capability layer above the enums
- add structures for compile diagnostics so frontend and backend share one error model

### P0.2 Fill in backend validation

#### `src/main.rs`

- add `graph_id` sanitizer and safe filename policy
- validate graph metadata shape before saving
- validate runtime request shape before starting a run
- validate that required nodes exist for runnable graphs
- validate that execution, risk, agent, intent, and data chains are complete
- reject malformed QuantScript graph imports before converting them into graph JSON
- add request size limits and error categorization for save, run, and parse endpoints

#### `frontend/src/graph/validation.js`

- keep UI validation, but make its rule set match backend validation one-to-one
- add explicit unsupported capability errors, not just connection errors
- distinguish between:
  - invalid graph
  - valid but not runnable graph
  - runnable but contains unsupported backend features

#### `qrpc_compiler`

- add backend compile-time validation helpers for:
  - missing dependencies
  - incompatible source and intent pairings
  - incompatible symbol and exchange combinations
  - unsupported runtime mode
  - invalid execution topology

### P0.3 Remove misleading runtime logic

#### `qrpc_runtime/src/intent_module.rs`

- stop hardcoding moving-average windows and thresholds
- read intent parameters from `IntentConfig.params`
- reject intent configs that are missing required parameters
- support stable parameter naming conventions shared with frontend config fields

#### `qrpc_runtime/src/agent_module.rs`

- stop hardcoding decision thresholds and sizing rules that should come from config
- read agent decision thresholds from `AgentConfig.params`
- keep strategy-specific defaults only as fallback values
- make symbol selection come from input scopes instead of hardcoded `BTCUSDT`

#### `qrpc_runtime/src/risk_checker.rs`

- define one consistent position sizing semantic
- stop using loosely defined `quantity_ratio` as both leverage and budget proxy
- read risk thresholds from `RiskConfig`
- add validation around clamp math so risk decisions match execution sizing behavior
- add structured risk reason codes for unsupported or invalid actions

#### `qrpc_runtime/src/execution_module.rs`

- use the same sizing semantic chosen by risk
- stop inferring order budget with hidden multipliers such as `* 0.25`
- read execution-related parameters from runtime config instead of from first-node heuristics only
- make symbol and exchange flow through from decisions instead of hardcoding `BTCUSDT`

#### `qrpc_runtime/src/fill_engine.rs`

- preserve semantic consistency between accepted order size, reserved balances, fills, and portfolio state
- add clearer event payloads for accepted, partial, open, rejected, and filled states
- keep order lifecycle explainable for later UI diagnostics

### P0.4 Tighten safety boundaries

#### `src/main.rs`

- replace open CORS with a local frontend allowlist
- normalize file path construction for saved graph and script artifacts
- reject path traversal attempts in `graph_id`
- separate user validation errors from internal server errors
- avoid panics from malformed request graphs or malformed imported scripts

#### `qrpc_runtime/src/intent_module.rs`

- replace direct indexing like `input_data_ids[0]` with checked access
- return no signal or explicit diagnostics instead of panicking

#### `qrpc_runtime/src/lib.rs`

- remove `expect` and panic-prone flows where recoverable errors are better
- propagate runtime-stage errors as runtime events or typed results

### P0.5 Establish minimal observability

#### `src/main.rs`

- persist per-run config snapshot
- persist compile output used by the run
- persist runtime events, fills, and final account snapshot
- return stable run metadata for later retrieval

#### `qrpc_core/src/lib.rs`

- add typed run summary and diagnostics structures
- add stable event payload contracts for:
  - no-op decisions
  - rejected decisions
  - clamped decisions
  - empty data
  - unsupported configuration

#### Frontend runtime panels

Files:

- `frontend/src/components/EventStreamPanel.jsx`
- `frontend/src/components/PropertyPanel.jsx`
- `frontend/src/components/StrategyCanvas.jsx`

Tasks:

- show why a node produced no output
- show why a risk decision was clamp or reject
- show compile warnings and runtime warnings separately
- show run metadata, last compile id, and final account summary together

## P1: Make beta stable for repeated use

### P1.1 Make graph compilation strict

#### `frontend/src/graph/compileGraph.js`

- replace optimistic compile output with actual compile diagnostics
- compute topology order from edges instead of using node list order
- flag unused nodes and dangling subgraphs
- flag unsupported multiple-input or multiple-output combinations
- mark `compilable` false when graph cannot run safely

#### `frontend/src/graph/validation.js`

- add validation for:
  - missing runtime node
  - multiple execution nodes when unsupported
  - multiple risk owners when unsupported
  - missing required config fields after type conversion
  - inconsistent runtime mode and execution mode

#### `qrpc_compiler`

- own the authoritative compile step for runtime config validation
- expose diagnostics back to frontend instead of duplicating business rules ad hoc

### P1.2 Persist runtime state and artifacts

#### New persistence surface

Suggested paths:

- `storage/runs/`
- `storage/graphs/`

#### `src/main.rs`

- add run artifact write path
- add run list and run detail endpoints
- store:
  - run metadata
  - graph id and compile id
  - compile artifact
  - runtime events
  - fill reports
  - account snapshots

#### `frontend/src/store/graphStore.js`

- separate local editor state from persisted backend run history
- load latest graph safely
- support viewing prior graph versions and prior runs

### P1.3 Upgrade tests

#### `qrpc_compiler`

- add graph compile tests for invalid topology, unsupported modules, and invalid parameter sets

#### `qrpc_runtime`

- add scenario tests for:
  - empty input handling
  - invalid config rejection
  - multi-node graph execution
  - multi-exchange arbitrage behavior
  - clamp and reject behavior
  - partial fills and resting order transitions

#### Root workspace

- add end-to-end tests for:
  - graph save
  - graph compile
  - test run
  - SSE event stream
  - graph to QuantScript to graph conversion

### P1.4 Improve UI operability

#### `frontend/src/components/EventStreamPanel.jsx`

- add event type filters
- add severity filters
- group events by run phase or node
- show decision explanation text inline

#### `frontend/src/components/PropertyPanel.jsx`

- show validation errors next to config fields
- show backend support notes for each module capability
- show whether a field is active, ignored, or unsupported

#### `frontend/src/components/StrategyCanvas.jsx`

- show node runtime status directly on the canvas
- highlight invalid edges and blocked nodes
- allow jumping from issue panel to node

### P1.5 Add resource governance

#### `src/main.rs`

- add TTL or max retention for in-memory runs
- add pagination for run history endpoints
- add graph artifact cleanup strategy

#### Storage policy docs

- define how many graphs, runs, and artifacts are retained
- define which files are latest pointers and which are versioned artifacts

## P2: Make beta useful for real research

### P2.1 Add real read-only data sources

#### `qrpc_runtime/src/data_module.rs`

- split mock provider from real provider abstraction
- support:
  - historical K-line fetch
  - real-time quote or book ticker fetch
  - replay feed from stored datasets
- keep mock provider only for demo mode and tests

#### `qrpc_core/src/lib.rs`

- add source capability metadata and data quality status details

#### Frontend

- show source mode:
  - mock
  - historical
  - realtime
  - replay

### P2.2 Add historical replay and backtest

#### `qrpc_runtime/src/sandbox.rs`

- extend fast backtest into a real replay/backtest scheduler
- support fixed date ranges and stepwise replay
- snapshot intermediate portfolio states at replay checkpoints

#### `src/main.rs`

- add backtest run endpoints
- store backtest metrics and timeline artifacts

#### Frontend

- add backtest result panel
- add equity curve, drawdown, trades, and summary stats

### P2.3 Make parameters first-class

#### Frontend config path

- normalize field names across module definitions, compile output, and runtime params
- support named parameter presets

#### Runtime path

- pass config values into `params` maps consistently
- stop using hidden strategy constants when equivalent UI config exists

### P2.4 Stabilize graph-source and formal QuantScript boundaries

#### `frontend/src/graph/quantscript.js`

- preserve node type, config type, and edge semantics exactly
- avoid lossy conversion of boolean, numeric, and string fields
- keep `strategy_graph` graph-source serialization clearly separate from formal QuantScript terminology

#### `src/main.rs`

- improve graph-source parse validation and import diagnostics
- preserve graph-source metadata without overwriting prior formal-source artifacts unexpectedly

#### Tests

- add round-trip golden tests:
  - Graph -> strategy_graph source -> Graph
  - formal QuantScript -> runtime lowering diagnostics

## P3: Product-grade expansion after beta

### P3.1 Plugin system

#### `frontend/src/modules/moduleRegistry.js`

- support dynamic module registration
- validate plugin metadata and schema shape

#### `qrpc_runtime`

- support provider registration for data, intent, agent, risk, and execution providers

#### `qrpc_core`

- add manifest and capability contracts for plugins

### P3.2 Multi-symbol and multi-market

#### `qrpc_core/src/lib.rs`

- expand `Symbol`, `Exchange`, and related scope types
- move toward capability-based symbol support rather than ad hoc hardcoding

#### `qrpc_runtime`

- remove hardcoded `BTCUSDT`
- propagate symbol scope through intent, agent, risk, execution, and fill paths

### P3.3 Multi-account abstraction

#### `qrpc_core/src/lib.rs`

- add account slot and account state abstractions

#### `qrpc_runtime`

- separate portfolio state from account adapter state
- support paper account and simulated account slots first

### P3.4 Permission and audit model

#### `src/main.rs`

- record who saved a graph and who started a run
- stamp artifacts with actor metadata and timestamps

#### Frontend

- show audit metadata in graph and run views

## Immediate implementation checklist

### Must do now

- align frontend modules with backend support
- remove hidden or fake capabilities
- add backend validation for graph, run request, mode, exchange, symbol, and graph id
- unify risk and execution sizing semantics
- wire real config parameters into intent, agent, risk, and execution logic
- lock down file path and CORS boundaries
- make compile step capable of failing honestly

### Strongly recommended before beta

- persist run artifacts
- add run logs and diagnostic error categories
- improve graph compile and validation hints
- add account, position, fill, and reject explanation panels
- add workspace-level regression coverage

### Valuable after beta base is stable

- historical replay and backtest
- parameter sweep support
- pluginization
- multi-symbol support
- Graph and QuantScript round-trip hardening
