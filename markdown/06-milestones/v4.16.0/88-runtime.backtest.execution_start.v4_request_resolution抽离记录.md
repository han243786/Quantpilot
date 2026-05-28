# v4.16.0 runtime.backtest.execution_start.v4_request_resolution 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001P-03。  
> 基准: `87-runtime.backtest.execution_start.v4_request_resolution抽离方案.md`、`86-runtime.backtest.execution_start.v4_request_resolution单子叶等价基线.md`、`85-runtime.backtest.execution_start.v4_projection单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批完成 `runtime.backtest.execution_start.v4_request_resolution` 第一轮物理抽离；只迁移四个 request resolution helper，不迁移 replay/runtime execution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller 或发布过渡连接。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001P 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | request resolution helper 父级私有导入、错误 code 保持、fallback bridge 保持 | 落地 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_request_resolution` | 物理抽离 |
| 模块树 | `runtime.backtest.execution_start.v4_request_resolution` 白箱节点 | 补真实文件 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_request_resolution` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.v4_request_resolution` |
| 父模块 | `runtime.backtest.execution_start` |
| 新真实文件 | `src/runtime/backtest/v4_request_resolution.rs` |
| 父级文件 | `src/runtime/backtest/execution_start.rs` |
| public API | 无新增 public API；四个入口 helper 为父模块可见 `pub(super)` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 实际改动

| 文件 | 改动 | 边界 |
| --- | --- | --- |
| `src/runtime/backtest/v4_request_resolution.rs` | 新建 v4 request resolution 子模块 | 只承载 request detection、graph resolution、symbol resolution、event type resolution |
| `src/runtime/backtest/execution_start.rs` | 新增 path module 与父级私有导入 | 继续保留 `start_backtest_run`、`execute_backtest_request`、`execute_v4_backtest_request`、record write 和 transient spill |

父级私有导入形态:

```rust
#[path = "v4_request_resolution.rs"]
mod v4_request_resolution;

use v4_request_resolution::{
    is_v4_backtest_request, resolve_v4_backtest_graph, resolve_v4_backtest_market_event_type,
    resolve_v4_backtest_symbols,
};
```

子模块内部使用:

```rust
use super::*;
```

---

## 已迁移清单

| 函数 | 新位置 | 可见性 |
| --- | --- | --- |
| `is_v4_backtest_request` | `src/runtime/backtest/v4_request_resolution.rs` | `pub(super)` |
| `resolve_v4_backtest_graph` | `src/runtime/backtest/v4_request_resolution.rs` | `pub(super)` |
| `resolve_v4_backtest_symbols` | `src/runtime/backtest/v4_request_resolution.rs` | `pub(super)` |
| `resolve_v4_backtest_market_event_type` | `src/runtime/backtest/v4_request_resolution.rs` | `pub(super)` |

---

## 保持原位

| owner | 保留内容 | 原因 |
| --- | --- | --- |
| `src/runtime/backtest/execution_start.rs` | `start_backtest_run`、`execute_backtest_request`、`execute_v4_backtest_request` | 创建路径、runtime execution 和 record write owner 不迁移 |
| `src/runtime/backtest/v4_projection.rs` | `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` | projection 子叶已 closeout，不回流、不混入 request resolution |
| `src/backtest_artifacts.rs` | `build_backtest_artifact_views`、artifact views、manifest digest、transient spill helper | artifact schema owner 不迁移 |
| `src/runtime_response_mapping.rs` | `backtest_run_response` 与 response mapping | response schema owner 不迁移 |
| `src/runtime_persistence.rs` | saved/transient record IO | persistence owner 不迁移 |
| `src/frontend_api_types.rs` | API 类型 | schema owner 不迁移 |
| `src/runtime/backtest.rs` | record store、replay、experiment sibling | 后续另起基线 |
| `AppState` | `state.backtests`、store dirs、transient dirs、locks | state owner 与锁顺序不迁移 |

---

## 等价约束

- `execute_backtest_request` 的 v4 path 判定顺序不变。
- `execute_v4_backtest_request` 的 graph/symbol/event resolution 调用顺序不变。
- `FrontendRunRequest`、`Value`、`V4MachineGraphContract` 和 `MachineEventCatalog` 输入输出语义不变。
- `compile_runtime_protocol_via_qs`、`compile_runtime_protocol_config`、`audit_v4_quant_script_static`、`build_v4_qs_runtime_handoff`、`bridge_core_ir_to_v4_machine_graph` fallback 顺序不变。
- `json_bad_request`、`json_bad_request_with_code`、`ERR_QSC_CONTRACT_INVALID`、`v4_graph_invalid`、`v4_runtime_handoff_rejected`、`v4_graph_missing`、`v4_event_catalog_missing` 错误 code 与语义不变。
- `V4BacktestArtifact`、`BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`FrontendRuntimeEvent` 和 `RuntimeEventEnvelope` schema 不变。
- 不新增 public API，不新增 sibling 横向连接，不新增发布版本过渡、缓存旁路或性能优化提案。ASCII guard: `release transition guard`。

---

## 明确排除

- 不迁移 `execute_v4_backtest_request`。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不迁移 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` 或 `src/runtime/backtest/v4_projection.rs`。
- 不迁移 replay bars/ticks、`V4PaperSimulatedRuntime`、record write、artifact view、transient spill 或 `state.backtests`。
- 不迁移 record store、replay、experiment、compare、artifact schema、persistence owner、response mapping owner、schema owner、state owner 或 frontend caller。
- 不进入整理、重构、发布版本过渡或性能连接优化。

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

下一批进入 `BE-001P-04 runtime.backtest.execution_start.v4_request_resolution 单叶 closeout`。closeout 必须判断:

1. 四个 request resolution helper 的父级私有子模块等价是否成立。
2. 本叶是否值得继续细拆。
3. 若继续细拆，是否需要另起更低层基线，而不是直接迁移 record write、projection、schema、state、persistence 或 frontend。
4. 若不继续细拆，是否回到 `runtime.backtest.execution_start` 父叶或 `runtime.backtest` sibling 队列。

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.v4_request_resolution` 已抽离时，必须说明只迁移了 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` 到 `src/runtime/backtest/v4_request_resolution.rs`，且父级 `src/runtime/backtest/execution_start.rs` 只私有导入这四个 helper。不得宣称 replay/runtime execution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、整理、重构或发布过渡已经完成。

---

## 验收标准

1. `88-runtime.backtest.execution_start.v4_request_resolution抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/backtest/v4_request_resolution.rs` 存在并承载四个 request resolution helper。
3. `src/runtime/backtest/execution_start.rs` 只通过父级私有导入调用四个入口 helper。
4. replay/runtime execution、projection、record write、artifact schema、response schema、state owner、persistence owner 和 frontend caller 不迁移。
5. 本批不引入发布版本过渡。
