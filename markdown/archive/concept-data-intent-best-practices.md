# Data Intent Best Practices (Archived, Normalized)

This file replaces a legacy concept note that had persistent encoding damage and
excess narrative noise.

## Historical Role

- Early guidance for strategy authors working on the data-to-intent path
- Background material only
- Not part of the active implementation baseline

## Retained Conclusions

- Strategy logic should consume normalized market data instead of raw exchange fields.
- Strategy outputs should stay at the intent layer, not jump directly to orders.
- Multi-strategy combination should happen through agent and risk layers, not inside
  isolated strategy scripts.
- New strategy work should extend stable config and protocol objects before adding
  one-off scripts.

## Current Source Of Truth

- [Data And Intent Layer Principles](D:/rust-js-pr/QuantPilot/quantpilot/markdown/principles/principles-data-and-intent-layer.md)
- [QuantPilot Design Principles](D:/rust-js-pr/QuantPilot/quantpilot/markdown/principles/principles-quantpilot-design.md)
- [Active QRPC RFC Index](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/README.md)
