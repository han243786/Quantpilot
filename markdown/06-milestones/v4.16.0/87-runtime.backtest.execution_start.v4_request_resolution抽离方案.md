# v4.16.0 runtime.backtest.execution_start.v4_request_resolution 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001P-02。  
> 基准: `86-runtime.backtest.execution_start.v4_request_resolution单子叶等价基线.md`、`85-runtime.backtest.execution_start.v4_projection单叶closeout.md`、`81-runtime.backtest.execution_start单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.backtest.execution_start.v4_request_resolution` 抽离方案，`no code movement`；下一批若实施，只允许迁移 v4 request detection、graph resolution、symbol resolution 和 market event type resolution helper，不得混入 replay/runtime execution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller 或发布过渡连接。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001P 从 v4 request resolution 基线进入抽离方案 | 推进 |
| 规范矩阵 | request resolution helper 最小移动、父级私有调用、错误码/兼容桥保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_request_resolution` | 抽离方案 |
| 模块树 | `runtime.backtest.execution_start.v4_request_resolution` 白箱节点 | 补方案状态 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_request_resolution` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.v4_request_resolution` |
| 父模块 | `runtime.backtest.execution_start` |
| 当前真实文件 | `src/runtime/backtest/execution_start.rs` |
| 下一批计划目标 | future src/runtime/backtest/v4_request_resolution.rs |
| 父级导入策略 | 在 `execution_start.rs` 中用 path module 接入 `v4_request_resolution`，只由父模块调用 |
| 对外 API 策略 | 不新增 public API；父模块可见 helper 使用 `pub(super)`，其余 helper 若存在则保持子模块私有 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 适配性校验

当前 `src/runtime/backtest/execution_start.rs` 中，v4 request resolution helper 已形成清晰连续区域:

| helper | 当前调用关系 | 方案可见性 |
| --- | --- | --- |
| `is_v4_backtest_request` | 被 `execute_backtest_request` 调用，用于决定是否进入 v4 path | `pub(super)` |
| `resolve_v4_backtest_graph` | 被 `execute_v4_backtest_request` 调用，解析或桥接 `V4MachineGraphContract` | `pub(super)` |
| `resolve_v4_backtest_symbols` | 被 `execute_v4_backtest_request` 调用，解析 replay symbols | `pub(super)` |
| `resolve_v4_backtest_market_event_type` | 被 `execute_v4_backtest_request` 调用，从 `MachineEventCatalog` 选择 replay event type | `pub(super)` |

该批 helper 依赖 `FrontendRunRequest`、`Value`、`V4MachineGraphContract`、`MachineEventCatalog`、`compile_runtime_protocol_via_qs`、`compile_runtime_protocol_config`、`audit_v4_quant_script_static`、`build_v4_qs_runtime_handoff`、`bridge_core_ir_to_v4_machine_graph`、`json_bad_request`、`json_bad_request_with_code`、`ERR_QSC_CONTRACT_INVALID` 和 `internal_error`。实际迁移时优先保留 `use super::*;`，不新建跨 sibling 依赖，不改错误码和 schema owner。

---

## 抽离目标

下一批实际抽离只允许做以下结构性移动:

1. 新建 src/runtime/backtest/v4_request_resolution.rs。
2. 在 `src/runtime/backtest/execution_start.rs` 内用 path module 接入 `v4_request_resolution`。
3. 从 `execution_start.rs` 移入四个 v4 request resolution helper。
4. 只把父级真实调用的 helper 暴露为 `pub(super)`。
5. 保持 `execute_backtest_request` 的 v4 path 判定和 `execute_v4_backtest_request` 的 graph/symbol/event resolution 调用顺序不变。
6. 保持 replay bars/ticks、`V4PaperSimulatedRuntime`、projection、governance envelope、record write、artifact view 构建和 transient spill 顺序不变。

建议形态:

```rust
#[path = "v4_request_resolution.rs"]
mod v4_request_resolution;

use v4_request_resolution::{
    is_v4_backtest_request,
    resolve_v4_backtest_graph,
    resolve_v4_backtest_market_event_type,
    resolve_v4_backtest_symbols,
};
```

`v4_request_resolution` 子模块内部保持:

```rust
use super::*;
```

---

## 允许迁移清单

| 函数 | 迁移原因 | 可见性策略 |
| --- | --- | --- |
| `is_v4_backtest_request` | v4 path detection 白箱边界 | `pub(super)` |
| `resolve_v4_backtest_graph` | v4 graph JSON / formal QS / core IR bridge resolution 白箱边界 | `pub(super)` |
| `resolve_v4_backtest_symbols` | request symbols 与 metadata symbols resolution 白箱边界 | `pub(super)` |
| `resolve_v4_backtest_market_event_type` | replay market event type resolution 白箱边界 | `pub(super)` |

---

## 必须保持原位

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

## 错误与兼容桥保持

| 场景 | 当前错误/路径 | 约束 |
| --- | --- | --- |
| v4 graph JSON parse 失败 | `v4_graph_invalid` | 不改变 bad request 类型 |
| static contract 失败 | `v4_graph_invalid` + `ERR_QSC_CONTRACT_INVALID` | 不改变错误 code |
| formal QS handoff rejected | `v4_runtime_handoff_rejected` + `ERR_QSC_CONTRACT_INVALID` | 不改变 diagnostics 拼接 |
| formal QS 无 parsed graph | `v4_graph_missing` + `ERR_QSC_CONTRACT_INVALID` | 不改变错误 code |
| core IR bridge 无 graph | `v4_graph_missing` + `ERR_QSC_CONTRACT_INVALID` | 不改变 bridge diagnostics 暴露 |
| event catalog 缺失或无 event | `v4_event_catalog_missing` + `ERR_QSC_CONTRACT_INVALID` | 不改变 replay 前置失败语义 |

---

## 明确排除

- 不迁移 `execute_v4_backtest_request`。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不迁移 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` 或 `src/runtime/backtest/v4_projection.rs`。
- 不迁移 replay bars/ticks、`V4PaperSimulatedRuntime`、record write、artifact view、transient spill 或 `state.backtests`。
- 不迁移 record store、replay、experiment、compare、artifact schema、persistence owner、response mapping owner、schema owner、state owner 或 frontend caller。
- 不改变 `V4BacktestArtifact`、`V4MachineGraphContract`、`BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`FrontendRuntimeEvent` 或 `RuntimeEventEnvelope` schema。
- 不新增发布版本过渡、横向直连、缓存旁路或性能优化提案。ASCII guard: `release transition guard`。

---

## 中止条件

下一批实际抽离只要出现以下任一情况，必须中止并回到方案讨论:

1. 需要改变 v4 request detection、graph/symbol/event type resolution 的行为或错误码。
2. 需要改变 replay bars/ticks、runtime execution、projection、artifact schema、response schema、event envelope、state lock、persistence IO 或 frontend caller。
3. 需要把 helper 变为 `pub(crate)` 或更宽的 public API。
4. 需要让 projection、record store、replay、experiment、compare 或 frontend caller 直接调用 request resolution 子模块。
5. 需要移动 `execute_v4_backtest_request` 或改变 record write / transient spill 顺序。
6. `cargo check -p quantpilot` 暴露的可见性问题不能通过父级私有导入解决。
7. `api_backtest`、`api_evidence_contract`、`api_run` 或 `cargo test --no-run` 出现行为回归。

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

下一批进入 `BE-001P-03 runtime.backtest.execution_start.v4_request_resolution 抽离记录`。实施范围只能是:

1. 新建 v4 request resolution 子模块。
2. 移入允许迁移清单中的四个 helper。
3. 在父级 `execution_start.rs` 私有导入四个入口 helper。
4. 保持 replay/runtime execution、projection、record write、artifact、response、persistence、schema、state、frontend 和发布过渡边界不变。

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.v4_request_resolution` 已有抽离方案时，必须说明本批 `no code movement`。不得宣称 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` 或任何 helper 已迁移；不得宣称 replay/runtime execution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、整理、重构或发布过渡已经完成。

---

## 验收标准

1. `87-runtime.backtest.execution_start.v4_request_resolution抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案明确下一批只允许迁移四个 request resolution helper。
3. 方案明确父级只私有导入 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type`。
4. 方案明确 replay/runtime execution、projection、record write、artifact schema、response schema、state owner、persistence owner 和 frontend caller 不迁移。
5. 本批不发生代码移动。
