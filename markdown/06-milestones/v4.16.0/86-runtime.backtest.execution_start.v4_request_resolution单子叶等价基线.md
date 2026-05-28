# v4.16.0 runtime.backtest.execution_start.v4_request_resolution 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001P-01。  
> 基准: `81-runtime.backtest.execution_start单叶closeout.md`、`85-runtime.backtest.execution_start.v4_projection单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.backtest.execution_start.v4_request_resolution` 等价基线，`no code movement`；不迁移代码、不拆 record write、不改 projection、不改 v4 artifact schema、不改 response schema、不改 frontend caller。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001P 从 `v4_projection` closeout 回到父叶下一候选基线 | 推进 |
| 规范矩阵 | v4 request detection、graph resolution、symbol resolution、event catalog resolution、父子通信 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_request_resolution` | 新增单子叶基线 |
| 模块树 | `runtime.backtest.execution_start.v4_request_resolution` 白箱候选 | 新增 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_request_resolution` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.v4_request_resolution` |
| 父模块 | `runtime.backtest.execution_start` |
| 当前真实文件 | `src/runtime/backtest/execution_start.rs` |
| 保留 sibling 文件 | `src/runtime/backtest/v4_projection.rs` |
| 保留父级文件 | `src/runtime/mod.rs`、`src/runtime/backtest.rs`、`src/backend/runtime/routes/backtest.rs` |
| 保留 shared owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs` |
| 当前 helper 群 | `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 当前真实边界

`runtime.backtest.execution_start.v4_request_resolution` 只覆盖 v4 backtest 创建路径中进入 replay/runtime 前的 request resolution:

1. `is_v4_backtest_request` 根据 `FrontendRunRequest.backtest_options.runtime_kind`、`graph_json.metadata.artifacts.v4_machine_graph`、`graph_json.metadata.v4_machine_graph` 和 formal QuantScript source 判断是否进入 v4 path。
2. `resolve_v4_backtest_graph` 依次尝试读取 `/metadata/artifacts/v4_machine_graph`、`/metadata/v4_machine_graph`、`/artifacts/v4_machine_graph`。
3. 若 graph JSON 未内嵌 v4 graph，`resolve_v4_backtest_graph` 会尝试 `audit_v4_quant_script_static` + `build_v4_qs_runtime_handoff`。
4. 若仍无 v4 graph，`resolve_v4_backtest_graph` 会走 `compile_runtime_protocol_via_qs`、`compile_runtime_protocol_config`、`bridge_core_ir_to_v4_machine_graph` 兼容桥。
5. `resolve_v4_backtest_symbols` 优先使用 request symbols，其次使用 `graph_json.metadata.artifacts.v4_symbols` 或 graph metadata symbols，最后使用 runtime 默认 normalize。
6. `resolve_v4_backtest_market_event_type` 从 `MachineEventCatalog` 选择 market data event，优先包含 `bar` 或 `price` 的 event type，否则退到第一个 market data event 或 catalog first event。

本子叶不拥有 v4 replay bars/ticks、runtime execution、projection、record write、artifact views、response mapping、state lock、persistence 或 frontend caller。

---

## 输入输出白箱

| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `FrontendRunRequest` | `execute_backtest_request` / `execute_v4_backtest_request` | request body | 不改变 `runtime_kind`、symbols、backtest options 或 capability guard 语义 |
| `graph_json` | request body | `serde_json::Value` | 不改变 v4 graph pointer 优先级或 fallback 顺序 |
| `V4MachineGraphContract` | graph JSON、formal QS handoff、core IR bridge | `qrpc_core_ir::v4::V4MachineGraphContract` | 必须继续执行 `validate_static_contract` |
| `MachineEventCatalog` | v4 machine graph | event catalog | 缺失或无可 replay event 时保持 `v4_event_catalog_missing` 错误 |

| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| v4 path 判定 | `execute_backtest_request` | bool | 不改变 runtime kind、v4 graph、formal source 的任一入口判定 |
| graph | `execute_v4_backtest_request` | `V4MachineGraphContract` | 不改变错误 code、validation code 或 fallback bridge 语义 |
| symbols | `execute_v4_backtest_request` | `Vec<String>` | 不改变 request symbols 优先级和 `normalize_v4_backtest_symbols` 行为 |
| event type | `execute_v4_backtest_request` | `String` | 不改变 market data event 选择优先级 |

---

## 关键 helper 冻结

| helper | 当前职责 | 基线约束 |
| --- | --- | --- |
| `is_v4_backtest_request` | 判断是否走 v4 backtest path | 不改变 `runtime_kind = v4`、v4 graph pointers、formal source `v4_strategy` 判定 |
| `resolve_v4_backtest_graph` | 解析或桥接 v4 machine graph | 不改变 pointer 顺序、formal QS handoff、core IR bridge、`ERR_QSC_CONTRACT_INVALID` |
| `resolve_v4_backtest_symbols` | 解析 v4 replay symbols | 不改变 request symbols 优先级、metadata fallback 和默认 normalize |
| `resolve_v4_backtest_market_event_type` | 选择 replay event type | 不改变 market data、`bar`/`price` 优先级和 `v4_event_catalog_missing` 错误 |

---

## 错误与兼容桥冻结

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
- 不迁移 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` 或 `src/runtime/backtest/v4_projection.rs`。
- 不迁移 replay bars/ticks、`V4PaperSimulatedRuntime`、record write、artifact view、transient spill 或 `state.backtests`。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不迁移 record store、replay、experiment、compare、artifact schema、persistence owner、response mapping owner、schema owner、state owner 或 frontend caller。
- 不改变 `V4BacktestArtifact`、`V4MachineGraphContract`、`BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`FrontendRuntimeEvent` 或 `RuntimeEventEnvelope` schema。
- 不引入发布版本过渡、横向直连、缓存旁路或性能优化提案。ASCII guard: `release transition guard`。
- 不删除旧实现，不进入整理/重构阶段。

---

## 第一轮抽离候选方案

下一批如果进入抽离方案，应只允许规划新建 request resolution 子模块，例如:

```text
src/runtime/backtest/v4_request_resolution.rs
```

允许迁移候选:

- `is_v4_backtest_request`
- `resolve_v4_backtest_graph`
- `resolve_v4_backtest_symbols`
- `resolve_v4_backtest_market_event_type`

父级 `runtime.backtest.execution_start` 只能通过父模块私有导入调用这些 helper；不得让 projection、record store、replay、experiment、frontend caller 或 persistence owner 横向接入。

---

## 暂停点

- 如果抽离需要改变 `execute_v4_backtest_request` 的 replay/runtime/record write 顺序，则暂停。
- 如果抽离需要改变 v4 artifact projection，则暂停，回到 `v4_projection` closeout 排除边界。
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

AI 声称 `runtime.backtest.execution_start.v4_request_resolution` 已建立基线时，必须说明本批 `no code movement`。不得宣称 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` 或任何 helper 已经迁移；不得宣称 replay/runtime execution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `86-runtime.backtest.execution_start.v4_request_resolution单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线明确 `v4_request_resolution` 的输入、输出、helper 群、错误 code、兼容桥和 API 回归证据。
3. 基线明确下一批只能规划 request resolution helper 抽离，不能直接移动代码。
4. 基线明确 projection、record write、artifact schema、response schema、state owner、persistence owner 和 frontend caller 不迁移。
5. 本批不发生代码移动。
