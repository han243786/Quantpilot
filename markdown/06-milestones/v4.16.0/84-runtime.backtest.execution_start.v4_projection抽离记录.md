# v4.16.0 runtime.backtest.execution_start.v4_projection 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001O-03。  
> 基准: `83-runtime.backtest.execution_start.v4_projection抽离方案.md`、`82-runtime.backtest.execution_start.v4_projection单子叶等价基线.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批完成 `runtime.backtest.execution_start.v4_projection` 第一轮物理抽离；只迁移 v4 projection helper 与现有两个单元测试，不迁移 request resolution、record write、artifact schema、response schema、state owner、persistence owner、frontend caller 或发布过渡连接。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001O 从抽离方案进入抽离记录 | 推进 |
| 规范矩阵 | projection helper 私有模块、父级私有导入、schema/owner 保留 | 落地 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_projection` | 物理抽离记录 |
| 模块树 | `runtime.backtest.execution_start.v4_projection` 白箱节点 | 更新实际文件 |

---

## 引导坐标

| 坐标类型 | 坐标 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_projection` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `/api/runtime/backtest/*` 与 backend runtime 文件索引 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `runtime.backtest.execution_start.v4_projection` |
| 真实文件 | `src/runtime/backtest/v4_projection.rs`、`src/runtime/backtest/execution_start.rs` |
| public 方法 | 无新增 public API；父级内部入口只使用 `pub(super)` |
| 测试 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run` |

---

## 实际改动

| 文件 | 改动 | 边界 |
| --- | --- | --- |
| `src/runtime/backtest/v4_projection.rs` | 新建 v4 projection 子模块，迁入 helper 与测试 | 只承载 projection，不拥有 request resolution 或 record write |
| `src/runtime/backtest/execution_start.rs` | 新增 path module 与父级私有导入 | 继续保留 `execute_v4_backtest_request`、request resolution、record write 和 transient spill |

---

## 已迁移清单

| 函数/测试 | 迁移后位置 | 可见性 |
| --- | --- | --- |
| `build_v4_backtest_output` | `src/runtime/backtest/v4_projection.rs` | `pub(super)` |
| `v4_equity_curve_from_artifact` | `src/runtime/backtest/v4_projection.rs` | `pub(super)` |
| `frontend_events_from_v4_backtest_artifact` | `src/runtime/backtest/v4_projection.rs` | `pub(super)` |
| `v4_win_rate_from_equity_curve` | `src/runtime/backtest/v4_projection.rs` | 子模块私有 |
| `v4_portfolio_from_artifact` | `src/runtime/backtest/v4_projection.rs` | 子模块私有 |
| `v4_frontend_event` | `src/runtime/backtest/v4_projection.rs` | 子模块私有 |
| `v4_win_rate_counts_up_steps_over_directional_steps` | `src/runtime/backtest/v4_projection.rs` test module | 子模块 test |
| `v4_equity_curve_empty_artifact_does_not_fabricate_zero_point` | `src/runtime/backtest/v4_projection.rs` test module | 子模块 test |

父级 `src/runtime/backtest/execution_start.rs` 只私有导入:

```rust
use v4_projection::{
    build_v4_backtest_output, frontend_events_from_v4_backtest_artifact,
    v4_equity_curve_from_artifact,
};
```

---

## 保持原位

| owner | 保留内容 | 原因 |
| --- | --- | --- |
| `src/runtime/backtest/execution_start.rs` | `start_backtest_run`、`execute_backtest_request`、`execute_v4_backtest_request` | 创建路径和 record write owner 不迁移 |
| `src/runtime/backtest/execution_start.rs` | `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` | v4 request resolution 不混入 projection |
| `src/backtest_artifacts.rs` | `build_backtest_artifact_views`、artifact views、manifest digest、transient spill helper | artifact schema owner 不迁移 |
| `src/runtime_response_mapping.rs` | `backtest_run_response` 与 response mapping | response schema owner 不迁移 |
| `src/runtime_persistence.rs` | saved/transient record IO | persistence owner 不迁移 |
| `src/frontend_api_types.rs` | API 类型 | schema owner 不迁移 |
| `src/runtime/backtest.rs` | record store、replay、experiment sibling | 后续另起基线 |
| `AppState` | `state.backtests`、store dirs、transient dirs、locks | state owner 与锁顺序不迁移 |

---

## 等价约束

- `execute_v4_backtest_request` 的执行顺序不变: request resolution、replay、artifact 构建、projection、governance envelope、record write、artifact view、transient spill。
- `V4BacktestArtifact`、`BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`PortfolioState`、`FrontendRuntimeEvent` 和 `RuntimeEventEnvelope` schema 不变。
- `v4_equity_curve_from_artifact` 仍对空 artifact 返回空数组，不补造 zero point。
- `v4_win_rate_from_equity_curve` 仍忽略 flat step 和非有限值。
- `frontend_events_from_v4_backtest_artifact` 仍按 event time 与 event id 排序。
- 不新增 public API，不新增 sibling 横向连接，不新增发布版本过渡。ASCII guard: `release transition guard`。

---

## 明确排除

- 不迁移 `execute_v4_backtest_request`。
- 不迁移 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type`。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不迁移 `build_backtest_artifact_views`、`maybe_spill_transient_backtest_record` 或 `backtest_run_response`。
- 不迁移 record store、replay、experiment、compare、artifact schema、persistence owner、response mapping owner、schema owner、state owner 或 frontend caller。
- 不改变 route path、route method、response schema、event envelope、state lock 或 persistence IO。
- 不进入整理、重构或发布版本过渡。

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

## 下一步

下一批进入 `BE-001O-04 runtime.backtest.execution_start.v4_projection 单叶 closeout`。closeout 必须确认:

1. projection 子模块等价成立。
2. 是否停止继续细分。
3. 是否需要另起 `runtime.backtest.execution_start.v4_request_resolution` 基线。
4. record store、replay、experiment、artifact schema、response schema、state owner、persistence owner、frontend caller 和发布过渡仍未迁移。

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.v4_projection` 已抽离时，必须说明只迁移了 projection helper 和现有两个单元测试。不得宣称 `execute_v4_backtest_request`、request resolution、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、整理、重构或发布过渡已经完成。

---

## 验收标准

1. `84-runtime.backtest.execution_start.v4_projection抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/backtest/v4_projection.rs` 存在并承载六个 projection helper 与两个单元测试。
3. `src/runtime/backtest/execution_start.rs` 只通过父级私有导入调用三个 projection 入口 helper。
4. request resolution、record write、artifact schema、response schema、state owner、persistence owner 和 frontend caller 不迁移。
5. 本批不引入发布版本过渡。
