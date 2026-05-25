# v4.7.0 closeout — v4 AI 提案 + 性能优化

状态: v4 AI / sandbox 本地能力已落地；深度性能阈值保留后续基准。

## 已落实

- `src/runtime/mutation.rs`: v4 AI 提案目标必须锚定 backtest artifact，并提供 v4 trajectory 分析摘要函数。
- `src/sandbox_verification.rs`: 新增 v4 artifact replay-shape 对比，用 fill rate / symbols / trajectory coverage / Risk Plane rejection 判断候选是否退化。
- `frontend/src/capabilities/supportMatrix.js`: 声明 v4.5-v4.7 能力边界。
- `contracts/openapi/root.yaml`: 同步 v4 artifact 可选 tick/microstructure schema。

## 验证

- `cargo check`
- `cargo test v4_ai_proposal -- --nocapture`
- `cargo test v4_artifact_replay_shape -- --nocapture`

## 延后项

- 500 bar <5s 与 memory 峰值降低 >30% 属于性能基准项，需在稳定 runner 上采样后再关闭。
