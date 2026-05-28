# v4.16.0 runtime.backtest.execution_start.v4_projection 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001O-01。  
> 基准: `81-runtime.backtest.execution_start单叶closeout.md`、`80-runtime.backtest.execution_start抽离记录.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.backtest.execution_start.v4_projection` 等价基线，`no code movement`；不迁移代码、不拆 request resolution、不改 v4 artifact schema、不改 response schema、不改 frontend caller。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001O 从 `runtime.backtest.execution_start` closeout 进入下一轮子叶基线 | 推进 |
| 规范矩阵 | v4 projection helper、单元测试归属、父子通信、禁止横向连接 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_projection` | 新增单子叶基线 |
| 模块树 | `runtime.backtest.execution_start.v4_projection` 白箱候选 | 新增 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_projection` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.v4_projection` |
| 父模块 | `runtime.backtest.execution_start` |
| 当前真实文件 | `src/runtime/backtest/execution_start.rs` |
| 保留父级文件 | `src/runtime/mod.rs`、`src/runtime/backtest.rs`、`src/backend/runtime/routes/backtest.rs` |
| 保留 shared owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs` |
| 当前 helper 群 | `build_v4_backtest_output`、`v4_win_rate_from_equity_curve`、`v4_equity_curve_from_artifact`、`v4_portfolio_from_artifact`、`frontend_events_from_v4_backtest_artifact`、`v4_frontend_event` |
| 现有单元测试 | `v4_win_rate_counts_up_steps_over_directional_steps`、`v4_equity_curve_empty_artifact_does_not_fabricate_zero_point` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 当前真实边界

`runtime.backtest.execution_start.v4_projection` 只覆盖 v4 backtest artifact 的投影层:

1. `execute_v4_backtest_request` 先完成 v4 request resolution、bar/tick replay、artifact 构建和 transient record 准备。
2. `v4_equity_curve_from_artifact` 从 `V4BacktestArtifact.final_snapshot.simulated_execution.asset_curve` 投影 `Vec<BacktestEquityPoint>`。
3. `build_v4_backtest_output` 根据 artifact 与 equity curve 构建 `BacktestOutput`，包括 summary、final portfolio 和 `v4_artifact` 保留。
4. `v4_portfolio_from_artifact` 从 final snapshot 投影 `PortfolioState`。
5. `frontend_events_from_v4_backtest_artifact` 将 equity curve、risk decisions、execution capability sources、machine trajectory 投影为 `FrontendRuntimeEvent`。
6. `v4_frontend_event` 统一构建单个 frontend runtime event，保留默认 `RuntimeEventEnvelope`。

本子叶不拥有 request resolution、record write、artifact schema、response schema、state lock、persistence 或 frontend caller。

---

## 输入输出白箱

| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `V4BacktestArtifact` | `execute_v4_backtest_request` | `qrpc_core_ir::v4::V4BacktestArtifact` | 不改变 schema、trajectory、risk decision、execution capability source 或 final snapshot 语义 |
| `equity_curve` | `v4_equity_curve_from_artifact` | `Vec<BacktestEquityPoint>` | 不补造空 artifact 的 zero point |
| `backtest_id` | parent execution path | `&str` | 只用于 frontend event id / trace id 前缀 |
| `final_snapshot` | v4 artifact | JSON Value | 只读取 `simulated_execution` 下的现金、持仓市值和 asset curve |

| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `BacktestOutput` | `build_backtest_artifact_views`、`backtest_run_response` | `qrpc_core::BacktestOutput` | 不改变 `mode = v4_backtest`、summary、equity curve、final portfolio、`v4_artifact` 保留语义 |
| `BacktestEquityPoint` | output 和 frontend event projection | `Vec<qrpc_core::BacktestEquityPoint>` | 空 artifact 必须返回空数组 |
| `PortfolioState` | `BacktestOutput.final_portfolio` | `qrpc_core::PortfolioState` | 不改变 cash、net/gross notional 和 timestamp 映射 |
| `FrontendRuntimeEvent` | artifact views / frontend event stream view | governed event objects | 不改变 event type、source id、node id、severity、payload projection 或 sort order |

---

## 关键 helper 冻结

| helper | 当前职责 | 基线约束 |
| --- | --- | --- |
| `build_v4_backtest_output` | 构建 v4 backtest output、summary、portfolio 和 artifact 保留 | 不改变 total return、net profit、win rate、trade count、step count 和 artifact embedding |
| `v4_win_rate_from_equity_curve` | 从 equity curve 的方向性变化计算 win rate | 忽略相等步和非有限值；无方向性变化时返回 0 |
| `v4_equity_curve_from_artifact` | 从 final snapshot 的 asset curve 投影 equity point | 缺失或空 asset curve 返回空数组，不补造点 |
| `v4_portfolio_from_artifact` | 从 final snapshot 投影 final portfolio | 缺失字段使用安全默认值，不改变 ended timestamp |
| `frontend_events_from_v4_backtest_artifact` | 投影 portfolio、risk、execution capability、machine trajectory frontend events | 保持 event type、payload 字段、trace id、projection 字段和按时间/event_id 排序 |
| `v4_frontend_event` | 创建单个 frontend runtime event | 保持 `RuntimeEventEnvelope::default()` 和字段透传 |

---

## 现有证据

| 证据 | 覆盖 |
| --- | --- |
| `v4_win_rate_counts_up_steps_over_directional_steps` | win rate 忽略 flat step，按上涨/下跌方向步计算 |
| `v4_equity_curve_empty_artifact_does_not_fabricate_zero_point` | 空 v4 artifact 不伪造 equity point |
| `cargo test -p quantpilot --test api_backtest` | backtest start、v4/legacy artifact、compare、replay 和 governance evidence |
| `cargo test -p quantpilot --test api_evidence_contract` | runtime evidence contract 和 cleanup 语义 |
| `cargo test -p quantpilot --test api_run` | runtime run 侧旁路不受 backtest projection 影响 |
| `cargo test --no-run` | 编译单元测试，防止 helper 移动后作用域断裂 |

---

## 明确排除

- 不迁移 `execute_v4_backtest_request`。
- 不迁移 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type`。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不迁移 record store、replay、experiment、compare、artifact schema、persistence owner、response mapping owner、schema owner、state owner 或 frontend caller。
- 不改变 `V4BacktestArtifact`、`BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`FrontendRuntimeEvent` 或 `RuntimeEventEnvelope` schema。
- 不引入发布版本过渡、横向直连、缓存旁路或性能优化提案。ASCII guard: `release transition guard`。
- 不删除旧实现，不进入整理/重构阶段。

---

## 第一轮抽离候选方案

下一批如果进入实际抽离，建议只新建 projection 子模块，例如:

```text
src/runtime/backtest/v4_projection.rs
```

允许迁移候选:

- `build_v4_backtest_output`
- `v4_win_rate_from_equity_curve`
- `v4_equity_curve_from_artifact`
- `v4_portfolio_from_artifact`
- `frontend_events_from_v4_backtest_artifact`
- `v4_frontend_event`
- 现有两个 v4 projection 单元测试

父级 `runtime.backtest.execution_start` 只能通过父模块私有导入调用这些 helper；不得让 record store、replay、experiment、frontend caller 或 persistence owner 横向接入。

---

## 暂停点

- 如果抽离需要改变 v4 request resolution，则暂停，另起 `runtime.backtest.execution_start.v4_request_resolution` 基线。
- 如果抽离需要改变 artifact schema、response schema、event envelope、state lock、persistence 或 frontend caller，则暂停。
- 如果抽离需要把 helper 变为对外 public API，则暂停。
- 如果 `api_backtest`、`api_evidence_contract`、`api_run` 或 `cargo test --no-run` 发现回归，则先修复等价缺口。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.v4_projection` 已建立基线时，必须说明本批 `no code movement`。不得宣称 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` 或任何 helper 已经迁移；不得宣称 request resolution、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `82-runtime.backtest.execution_start.v4_projection单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线明确 `v4_projection` 的输入、输出、helper 群、现有单元测试和 API 回归证据。
3. 基线明确下一批实际抽离只允许移动 projection helper 和对应单元测试。
4. 基线明确 request resolution、record write、artifact schema、response schema、state owner、persistence owner 和 frontend caller 不迁移。
5. 本批不发生代码移动。
