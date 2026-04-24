# Intent Flow Concept (Archived, Normalized)

This file replaces a legacy flow note that is now kept only as historical context.

## Historical Role

- Early draft of the layered runtime chain
- Background material only
- Not part of the active implementation baseline

## Retained Conclusions

- The core runtime path should remain layered:
  `Data -> Intent -> Agent -> Risk -> Execution`.
- Parallel work may happen inside a layer, but each layer should merge outputs before
  advancing.
- Agent logic should combine and arbitrate multiple intents instead of bypassing the
  layer model.
- Risk control should remain centralized and global.

## Current Source Of Truth

- [QuantPilot Design Principles](D:/rust-js-pr/QuantPilot/quantpilot/markdown/principles/principles-quantpilot-design.md)
- [Trading Sandbox Implementation](D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-trading-sandbox.md)
- [Active QRPC RFC Index](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/README.md)
