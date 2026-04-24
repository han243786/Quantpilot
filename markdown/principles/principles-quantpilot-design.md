# QuantPilot Design Principles

## Product position

QuantPilot is not a single-strategy project.

It is a single-machine trading runtime platform whose first concern is stable infrastructure:

- unified protocol
- unified data semantics
- unified execution semantics
- centralized risk control

## Core principles

- infrastructure before strategy-specific shortcuts
- protocol before ad-hoc implementation
- unified data semantics before exchange-specific details
- unified execution semantics before mode-specific branching
- centralized risk before direct order placement
- single-machine efficiency before premature system complexity

## Text encoding rule

All user-facing text, docs, frontend source files, exported templates, and protocol examples must use `UTF-8` encoding without BOM.

Rules:

- never mix `GBK` / `GB2312` / locale-default encodings with frontend or docs assets
- frontend source files must be saved as `UTF-8` before commit
- user-facing strings must be reviewed in rendered UI, not only in source diff
- when importing old files, normalize encoding before editing content
- CI or pre-commit checks should reject replacement characters, mojibake, or non-UTF-8 text assets

This rule exists because text corruption is a product defect, not a cosmetic issue. Broken encoding damages validation messages, runtime feedback, exported scripts, and documentation trustworthiness.

## The main chain

QuantPilot uses one main chain:

`Data -> Intent -> Agent -> Risk -> Execution`

No strategy, script, or plugin should bypass this chain and place final orders directly.

This rule must hold in:

- real-time simulation
- fast backtest
- accurate simulation in the future
- live trading later

## Unified sandbox principle

The current project priority is a unified trading sandbox.

That means:

- all runtime modes share the same object semantics
- market data enters the system as normalized market data
- risk checks happen before fill and execution
- runtime events are structured and replayable
- state can be snapshotted and restored

Pluginization depends on this stable base.

## Layer execution rule

The system moves forward by layers.

Rules:

- layers are serial
- work inside one layer can be parallel
- parallel work must use the same frozen input snapshot
- layer outputs must be merged before the next layer starts
- internal mutable state should not leak across layers

This rule exists so that real-time and backtest modes can stay semantically aligned.

## Data rule

Only normalized market data should reach upper layers.

Upper layers should not depend on raw exchange fields directly.

The goal is not to preserve every raw field.
The goal is to keep one reusable, testable, replayable internal representation.

## Risk rule

Risk is the central gate of the system.

All agent outputs must pass through risk before execution.

Risk is responsible for things like:

- exposure limits
- leverage limits
- order size limits
- order frequency limits
- invalid action rejection
- structured risk events

## Persistence rule

QuantPilot is designed around hot state first and light persistence.

Persist the highest-value outputs:

- account snapshots
- runtime event logs
- backtest and replay reports
- audit traces

This keeps the system efficient on a single machine while preserving recovery and auditability.

## Long-term direction

After the unified sandbox is stable, the system can safely move toward:

- builtin modules expressed as plugins
- manifest and lock files
- dependency resolution
- local cache plus remote registry
- plugin lifecycle and ecosystem governance
