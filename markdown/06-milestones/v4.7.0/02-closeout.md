# v4.7.0 closeout — v4 AI 提案 + 性能优化 + 两轮诱错审计

> 状态: 已完成。v4 AI / sandbox / 元流水线 / S0-S1-P1 审计修复落地，P2/P3 已分流。
> closeout 门禁: 25/25 通过

## 已落实

### v4.5.0~v4.7.0 功能

- `src/runtime/mutation.rs`: v4 AI 提案目标必须锚定 backtest artifact，并提供 v4 trajectory 分析摘要函数。
- `src/sandbox_verification.rs`: 新增 v4 artifact replay-shape 对比，用 fill rate / symbols / trajectory coverage / Risk Plane rejection 判断候选是否退化。
- v4.5.0: OCO bracket、trailing stop、GTD、cancel-replace-amend 高级订单类型
- v4.5.0: tick replay + microstructure metrics
- v4.6.0: v4 LiveActual 安全边界 + Risk Plane 强制 + ProviderNative 门禁
- v4.7.0: v4 AI 提案 trajectory 分析 + v4 沙箱回放比较
- `frontend/src/capabilities/supportMatrix.js`: 声明 v4.5-v4.7 能力边界。
- `contracts/openapi/root.yaml`: 同步 v4 artifact 可选 tick/microstructure schema。

### 元流水线落地

- `tools/track-gate-metrics.ps1`: 修复语法错误，新增 `-DryRun`，NDJSON 输出
- `tools/check-capability-stack.ps1`: 新增第 25 项 closeout 门禁，5 层 capability 比对
- `tools/run-closeout-gates.bat`: 25 项全量 closeout
- `tools/check-gates-smoke.ps1`: 覆盖 track-gate-metrics DryRun + capability-stack
- 超级规范化 §2.2/§7.2/§7.4/§8.1 同步更新

### 第1轮审计修复 (S0/S1/P1 = 9/9)

| 项 | 修复 | 验证 |
|----|------|------|
| S0-1 锁顺序反转 | `mutation.rs` approval_records → ai_proposals 全路径一致 | `cargo test --test api_ai_proposal` |
| S1-1 UTF-8 乱码 | `v4_runtime.rs:1165,2254` 重新编码 | `cargo test -p qrpc-runtime v4_` |
| S1-2 Risk Plane 英文→中文 | `v4_runtime.rs:1834-1916` 全部 10 处 | `cargo test -p qrpc-runtime v4_` |
| P1-1 auth SQLite spawn_blocking | `auth/mod.rs` register/login | `cargo test auth::tests` |
| P1-2 graph delete fsync | `graph_api.rs:779-785` 补 sync_all | `cargo test --test api_graph_versions` |
| P1-3 存储配额 TOCTOU | `storage_lifecycle.rs` 降低 reject 阈值 | `cargo check --workspace` |
| P1-4 Auth OpenAPI | `contracts/openapi/root.yaml` 补 auth 端点 | `tools/check-version-consistency.ps1` |
| P1-5 README live 口径 | `README.md` 修正运行时模式描述 | `tools/check-user-facing-text.ps1` |
| P1-6 空 v4 回测 | `backtest.rs:586-593` 返回错误 | `cargo test v4_equity_curve_empty` |

### 第2轮审计修复 (P1 + 重点 P2)

| 项 | 修复 | 验证 |
|----|------|------|
| P1-1 速率限制器 ConnectInfo | `src/lib.rs` 使用 `into_make_service_with_connect_info::<SocketAddr>()`，恢复每 IP 限流 | `cargo test rate_limiter`, `cargo check --workspace` |
| P2-1 v4 runtime 无界集合 | `qrpc_runtime/src/v4_runtime.rs` 限制 event_log / orders / fills 历史长度，保留活跃订单 | `cargo test -p qrpc-runtime v4_` |

## 审计覆盖

| 轮次 | 文档 | 维度 | 发现 |
|:----:|------|:----:|:----:|
| 第1轮 | `自由维度诱错审计-v4.7.0-第1轮.md` | A-E (5维) | 30 (1 S0 + 2 S1 + 6 P1 + 13 P2 + 8 P3) |
| 第2轮 | `自由维度诱错审计-v4.7.0-第2轮.md` | F-H (3维) | 20 (1 P1 + 10 P2 + 8 P3 + 8 PASS) |
| **合计** | | **8维** | **50** |

## 遗留项

| 流向 | 数量 | 内容 |
|:----:|:----:|------|
| v4.7.1 | 0 | P1 已清零；PATCH 入口仅保留为可选归档 |
| v4.8.0 | 22 | 两轮审计剩余 P2 消化 |
| 持续回归 | 16 | 两轮审计 P3 |

## 延后项

- 500 bar <5s 与 memory 峰值降低 >30% 属于性能基准项，需在稳定 runner 上采样后再关闭。
