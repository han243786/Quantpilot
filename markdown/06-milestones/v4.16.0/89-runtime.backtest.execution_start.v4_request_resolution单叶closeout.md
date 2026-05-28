# v4.16.0 runtime.backtest.execution_start.v4_request_resolution 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001P-04。  
> 基准: `86-runtime.backtest.execution_start.v4_request_resolution单子叶等价基线.md`、`87-runtime.backtest.execution_start.v4_request_resolution抽离方案.md`、`88-runtime.backtest.execution_start.v4_request_resolution抽离记录.md`、`13-递归模块化全局根流程.md`。  
> 判定: `runtime.backtest.execution_start.v4_request_resolution` 已完成等价 closeout，并设置 `stop_split: true`。本叶不继续细拆；后续若继续，应回到父叶 `runtime.backtest.execution_start` 另起候选基线，优先评估 `runtime.backtest.execution_start.v4_runtime_execution`，不得从 request resolution 继续外扩。

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001P 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级私有 helper、错误 code 等价、fallback bridge 等价、`stop_split: true`、禁止横向连接 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_request_resolution` | closeout |
| 模块树 | `runtime.backtest.execution_start.v4_request_resolution` 白箱节点 | 更新状态与下一候选 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_request_resolution` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.v4_request_resolution` |
| 父模块 | `runtime.backtest.execution_start` |
| 真实文件 | `src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/execution_start.rs` |
| sibling 文件 | `src/runtime/backtest/v4_projection.rs` |
| 保留 owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`AppState` |
| 关键 public 方法 | 无新增 public API；父级只通过四个 `pub(super)` helper 调用 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树、UTF-8、diff check |

---

## 等价 closeout 结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| 父级调用 | 等价 | `execution_start.rs` 只私有导入 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` |
| v4 path detection | 等价 | `runtime_kind = v4`、`metadata/artifacts/v4_machine_graph`、`metadata/v4_machine_graph`、formal source `v4_strategy` 判定顺序不变 |
| graph resolution | 等价 | v4 graph pointer 优先级、static contract validation、formal QS handoff、Core IR compatibility bridge 均不变 |
| symbol resolution | 等价 | request symbols 优先、metadata fallback、`normalize_v4_backtest_symbols` 默认行为均不变 |
| event type resolution | 等价 | market data event 优先、`bar`/`price` 优先、fallback 到首个 event、`v4_event_catalog_missing` 错误均不变 |
| 错误 code | 等价 | `v4_graph_invalid`、`v4_runtime_handoff_rejected`、`v4_graph_missing`、`v4_event_catalog_missing`、`ERR_QSC_CONTRACT_INVALID` 均保留 |
| owner 边界 | 保留 | replay/runtime execution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller 均未迁移 |
| 发布过渡 | 未启动 | `release transition guard` 生效；未新增横向直连、缓存旁路或性能优化提案 |

---

## 当前白箱结构

| helper | 当前 owner | 细分判断 |
| --- | --- | --- |
| `is_v4_backtest_request` | `src/runtime/backtest/v4_request_resolution.rs` | 保留在本叶。它只判断是否进入 v4 path，单独拆文件会增加父级导入面 |
| `resolve_v4_backtest_graph` | `src/runtime/backtest/v4_request_resolution.rs` | 保留在本叶。虽然内部包含 pointer/formal QS/Core IR bridge 三段 fallback，但它们共同维护同一个 graph resolution 顺序 |
| `resolve_v4_backtest_symbols` | `src/runtime/backtest/v4_request_resolution.rs` | 保留在本叶。只服务 v4 replay symbols resolution |
| `resolve_v4_backtest_market_event_type` | `src/runtime/backtest/v4_request_resolution.rs` | 保留在本叶。只服务 replay market event type selection |

---

## 冻结符号表

| 类别 | 符号 | closeout 约束 |
| --- | --- | --- |
| 输入类型 | `FrontendRunRequest` | request body 语义不变 |
| 输入值 | `graph_json` | JSON pointer 优先级和 fallback 顺序不变 |
| graph 类型 | `V4MachineGraphContract` | static contract validation 不变 |
| event catalog | `MachineEventCatalog` | market data event 选择规则不变 |
| fallback helper | `compile_runtime_protocol_via_qs` | Core/QS fallback 顺序不变 |
| fallback helper | `compile_runtime_protocol_config` | runtime protocol 编译语义不变 |
| formal QS helper | `audit_v4_quant_script_static` | formal source audit 语义不变 |
| formal QS helper | `build_v4_qs_runtime_handoff` | rejected handoff 错误语义不变 |
| bridge helper | `bridge_core_ir_to_v4_machine_graph` | compatibility bridge 语义不变 |
| error helper | `json_bad_request` | status/error payload 语义不变 |
| error helper | `json_bad_request_with_code` | error code 绑定不变 |
| schema owner | `V4BacktestArtifact` | 不在本叶迁移或改 schema |
| schema owner | `BacktestOutput` | 不在本叶迁移或改 schema |
| schema owner | `BacktestRunResponse` | 不在本叶迁移或改 schema |
| schema owner | `BacktestRecord` | 不在本叶迁移或改 schema |
| frontend event | `FrontendRuntimeEvent` | 不在本叶迁移或改事件 schema |
| runtime event | `RuntimeEventEnvelope` | 不在本叶迁移或改 envelope schema |

---

## 细分价值判断

**最终判定**: `stop_split: true`。

理由:

1. 本叶没有 state、IO、锁、route、persistence、schema owner 或外部 API。
2. 四个 `pub(super)` helper 已经形成清晰父级私有边界；继续拆成 detection / graph / symbols / event type 会增加微文件和导入面。
3. `resolve_v4_backtest_graph` 内部三段 fallback 必须保持顺序原子性，拆开会制造跨 helper 认知负担，不能带来真实解耦收益。
4. 若未来发现 graph resolution 需要独立复用，必须由开发者明确提出新需求并另起基线；当前不得让 projection、record write、schema、state、persistence 或 frontend caller 横向接入。

---

## 后续递归队列

| 顺序 | 候选 | 进入条件 |
| --- | --- | --- |
| 1 | `runtime.backtest.execution_start.v4_runtime_execution` | 若继续父叶内部递归，必须先建单子叶等价基线，冻结 deterministic bars/ticks、`V4PaperSimulatedRuntime`、spawn_blocking 和 `V4BacktestArtifact` 输出 |
| 2 | `runtime.backtest.execution_start.legacy_dispatch` | 仅在 v4 runtime execution 完成或暂停后评估；不得混入 record write 或 artifact schema owner |
| 3 | `runtime.backtest.execution_start.record_write_bridge` | 当前不进入；涉及 transient spill、state owner、artifact views 和 persistence owner，必须另起决策暂停 |
| 4 | `runtime.backtest.record_store` | 只有当 execution_start 内部值得拆的候选完成或暂停后，才回到 `runtime.backtest` sibling 队列 |

---

## 本批次不做

- 不迁移 `execute_v4_backtest_request`。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不继续拆 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols` 或 `resolve_v4_backtest_market_event_type`。
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

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.v4_request_resolution` 已 closeout 时，必须说明: 本叶只完成四个 request resolution helper 的等价 closeout，并设置 `stop_split: true`。不得宣称 `execute_v4_backtest_request`、replay/runtime execution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树标记 `runtime.backtest.execution_start.v4_request_resolution` 已 closeout，并设置 `stop_split: true`。
3. 四个 request resolution helper 的父级私有调用、错误 code、fallback bridge 和排除边界均有白箱登记。
4. 下一候选回到父叶 `runtime.backtest.execution_start.v4_runtime_execution`，后续必须先建等价基线。
5. 治理门禁能发现 closeout 文档、`stop_split: true`、禁止迁移边界、下一候选和回归证据缺失。
