# Data Intent Module Concept (Archived, Normalized)

This file replaces a legacy concept draft that is no longer normative.

## Historical Role

- Early design note for the intent-generation layer
- Background material only
- Not part of the active implementation baseline

## Retained Conclusions

- The intent layer sits between normalized market data and downstream agent/risk
  decision layers.
- Intent objects describe desired exposure or directional preference, not immediate
  order placement.
- Strategy generation should be configuration-driven and auditable.
- Multiple intent producers may run in parallel, but their outputs should be merged
  before entering the next layer.

## Current Source Of Truth

- [Data And Intent Layer Principles](D:/rust-js-pr/QuantPilot/quantpilot/markdown/principles/principles-data-and-intent-layer.md)
- [Trading Sandbox Implementation](D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-trading-sandbox.md)
- [Active QRPC RFC Index](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/README.md)
