# Data And Intent Layer Principles

This document defines the stable boundary for the data layer and the intent layer in QuantPilot.

## Position in the main chain

The relevant section of the chain is:

`Data -> Intent -> Agent -> Risk -> Execution`

Where:

- the data layer standardizes input semantics
- the intent layer standardizes strategy expression
- later layers combine, constrain, and execute those outputs

## Data layer purpose

The data layer is not about "support more exchange APIs" first.

Its real job is:

- receive a stable data request
- hide source-specific field differences
- produce normalized market data
- feed the rest of the system with one reusable input shape

The data layer should not:

- produce final trading decisions
- produce final orders
- leak exchange-specific payloads into upper layers

## Intent layer purpose

The intent layer converts normalized market data into structured trading intent.

Intent is not an order.

Intent expresses things like:

- directional view
- target exposure or target position
- adjustment strength
- validity window
- reason for the decision

The goal is to produce a stable intermediate trading statement, not an immediate order instruction.

## Shared rules

### Input must be unified

Upper layers consume normalized market data only.

### Output must be unified

Strategy output must become intent objects before any later decision or execution step.

### Layers are serial

Inside one layer, work may run in parallel, but:

- all workers use the same frozen input snapshot
- outputs are merged before the next layer starts
- no next-layer work should read half-finished current-layer output

### Runtime modes must reuse the same semantics

Real-time simulation, fast backtest, and future accurate simulation must share the same data and intent semantics.

## Data layer best practices

- normalize before upper-layer consumption
- extend adapters and config, not upper-layer contracts, when adding new sources
- keep time and source identity explicit
- prefer stable internal semantics over preserving every raw field
- make snapshots deterministic enough for replay and testing

## Intent layer best practices

- each intent generator should express its own judgment only
- dependencies and config version should be explicit
- intent should describe target state, not direct order details
- outputs should remain explainable, auditable, and replayable
- multiple intent generators may run in parallel, but layer-level merge must be explicit

Preferred wording:

- "Adjust BTCUSDT long exposure target to 15% of equity for the next 30 seconds"

Avoid:

- "Buy 0.3 BTC immediately at market"

## Relationship to later layers

The data layer and intent layer prepare stable input for:

- agent combination and arbitration
- centralized risk control
- execution and fill simulation

That means the following must not happen:

- data drives direct order placement
- intent bypasses agent and risk
- agent bypasses risk

## Engineering mapping

In the current repository, the main mapping is:

- `qrpc_core`: core structures and protocol objects
- `qrpc_compiler`: config compilation and chain validation
- `qrpc_runtime`: runtime coordination under one shared semantic model
- `frontend/src/graph`: graph compilation and main-chain validation

## Stability criteria

The data and intent boundary is stable enough only when:

- normalized market data shape is stable
- intent shape is stable
- the same config can be reused across runtime modes
- outputs are consumable by agent and risk uniformly
- results are auditable and replayable
- direct order control cannot bypass the intent layer
