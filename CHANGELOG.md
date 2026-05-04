# Changelog

## 0.1.0

Initial private-baseline candidate scope for QuantPilot.
Public release is not ready while the license, dependency, and repository
visibility blockers remain open.

### Added

- single-machine paper runtime sandbox
- graph editor with validation, compile, save/load, and runtime export paths
- runtime event stream, run history, backtest history, and backtest analysis pages
- QuantScript formal-source compile path with runtime compile fallback
- capability governance, support matrix, and release-time gate scripts

### Changed

- release documentation now states the real beta boundary explicitly
- graph-source import/export is now named as `strategy_graph` source instead of generic QuantScript in the editor UI and parser messages
- legacy section-based QuantScript parsing is explicitly marked compatibility-only; formal QuantScript remains the supported product path
- support notes now distinguish graph-source artifacts from formal QuantScript lowering ownership
- repository hygiene now distinguishes source files from local runtime artifacts
- environment examples are documented for frontend proxy and direct API access

### Known limitations

- paper runtime only
- supported exchange boundary limited to `binance` and `okx`
- supported symbol boundary limited to `BTCUSDT`
- no live trading support
- no research-grade backtest claim
- `LICENSE` remains placeholder-only; final outbound license selection is still
  pending owner confirmation before any public release
