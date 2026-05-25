# v4.6.0 closeout — v4 LiveActual + OKX 实盘边界

状态: 安全边界已落地；OKX testnet 真下单 E2E 延后。

## 已落实

- `qrpc_runtime/src/v4_runtime.rs`: `V4Runtime` alias 与 `new_for_mode(LiveActual)` 边界开放。
- LiveActual 强制要求 `risk_plane.required = true`。
- LiveActual 下 ProviderActual settlement 使用 ProviderNative capability gate，runtime_simulated 能力会被拒绝。
- `frontend-executor/src/components/V4EvidencePanel.jsx`: 展示 provider allowed / real path 状态。

## 验证

- `cargo check`
- `cargo test -p qrpc-runtime v4_live_actual -- --nocapture`

## 延后项

- OKX testnet 真实下单、撤单与订单簿可见性验证需要外部凭证和网络环境，未在本地门禁中伪造通过。
