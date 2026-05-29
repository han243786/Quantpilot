# v4.16.0 runtime.backtest.experiment_sweep 单子叶等价基线
> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001V-01。  
> 前置: `106-runtime.backtest.replay单叶closeout.md`、`77-runtime.backtest单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 建立 `runtime.backtest.experiment_sweep` 单子叶等价基线；本批只冻结 experiment sweep route、参数网格、variant backtest 执行桥、实验记录持久化、save/discard 生命周期、父子通信和回归证据，不移动代码。  
> 代码动作: `no code movement`。

---

## 选择理由

`runtime.backtest.record_store` 与 `runtime.backtest.replay` 已完成单叶 closeout 并设置 `stop_split: true`，下一步必须回到 `runtime.backtest` sibling 队列。`runtime.backtest.experiment_sweep` 是当前仍留在 `src/runtime/backtest.rs` 的下一个真实 sibling。

1. 它覆盖 `/api/runtime/experiments/backtest-sweep`、`/api/runtime/experiments`、`/api/runtime/experiments/:experiment_id/save`、`/api/runtime/experiments/:experiment_id` 四组实验 API。
2. 它通过 `execute_backtest_request` 复用已抽离的 `runtime.backtest.execution_start`，因此必须先冻结兼容桥，避免后续迁移时把 sibling 横向直连扩大。
3. 它同时触碰 `state.experiments`、`experiment_store_dir`、backtest transient/persistent record、audit log 和 experiment response mapping，风险高于 replay，必须先做等价基线。
4. 它由 `tests/api_experiments.rs` 直接覆盖，并由 `tests/api_backtest.rs` 间接保护 variant backtest 行为。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | R5 backtest handler sibling 队列从 `runtime.backtest.replay` 转向 `runtime.backtest.experiment_sweep` | 推进 |
| 规范矩阵 | experiment route、参数网格、execution_start 复用桥、experiment persistence、save/discard lifecycle、audit、禁止横向连接 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` | 新增基线 |
| 模块树 | `runtime.backtest.experiment_sweep` | 新增 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep` |
| 当前真实文件 | `src/runtime/backtest.rs`、`src/runtime/backtest/execution_start.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/lib.rs` |
| public 方法 | `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record`、`execute_backtest_request`、`normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides`、`persist_experiment_record`、`list_experiment_records`、`load_experiment_record_from_state`、`experiment_list_item_from_record`、`experiment_detail_response_from_record`、`experiment_sweep_axes` |
| public 类型 | `FrontendExperimentRequest`、`FrontendExecutionAssumptionSweepGrid`、`ExperimentRecord`、`ExperimentDefinitionSummary`、`ExperimentVariantSummary`、`ExperimentListItem`、`ExperimentDetailResponse`、`FrontendExecutionAssumptionOverrides`、`FrontendBacktestReplaySource`、`DiscardRuntimeArtifactResponse` |
| 代表测试 | `tests/api_experiments.rs`、`tests/api_backtest.rs`、`tests/api_evidence_contract.rs` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `UserId` + `FrontendExperimentRequest` | auth middleware + JSON body | 必须继续执行 capability guard、runtime config guard 和 execution assumption override guard |
| 输入 | `graph_json` + `runtime_config` | frontend graph editor/runtime config | `graph_json` 缺失仍返回 `bad_request`；QS compile owner 不迁移 |
| 输入 | `parameter_grid` | `FrontendExecutionAssumptionSweepGrid` | fee/slippage 不能为负；空网格必须报错；variant 数量继续受 `MAX_EXPERIMENT_VARIANTS` 限制 |
| 输入 | `execute_backtest_request` bridge | `runtime.backtest.execution_start` | 只能作为父级 runtime 内部兼容桥复用，不得新增 sibling 横向直连 |
| 输出 | `ExperimentDetailResponse` | frontend/tests | 保留 experiment id、definition、variant summaries、saved 状态和 execution assumptions tag |
| 输出 | `PaginatedResponse<ExperimentListItem>` | frontend/tests | 保留排序、sweep axes、best variant 指标和分页语义 |
| 输出 | `DiscardRuntimeArtifactResponse` | frontend/tests | discard 只允许未 saved experiment，且清理 transient variant backtests |
| 输出 | audit entry | `GraphAuditAction::ExperimentCreated` | 仅 save 时按现有 actor 记录 audit，不迁移 audit owner |

---

## 兼容树

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> POST /api/runtime/experiments/backtest-sweep
  -> crate::runtime::start_backtest_experiment
  -> build_experiment_overrides
  -> execute_backtest_request
  -> ExperimentDetailResponse

backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> GET /api/runtime/experiments
  -> crate::runtime::list_experiments
  -> list_experiment_records
  -> experiment_list_item_from_record

backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> GET/DELETE/POST save experiment routes
  -> crate::runtime::{get_experiment_detail, save_experiment_record, discard_experiment_record}
  -> load_experiment_record_from_state
  -> experiment_detail_response_from_record
```

当前 route 真实 owner 是 `src/backend/runtime/routes.rs`，而不是 `src/backend/runtime/routes/backtest.rs`。本基线只记录现状；后续若抽离 route facade，必须另起方案，不能在本基线中顺手移动。

---

## owner 基线

| 子域 | 当前真实 owner | 代表方法/类型 | 当前处理 |
| --- | --- | --- | --- |
| experiment routes | `src/backend/runtime/routes.rs` | `/api/runtime/experiments/backtest-sweep`、`/api/runtime/experiments`、`/api/runtime/experiments/:experiment_id/save`、`/api/runtime/experiments/:experiment_id` | 本批不改 path/method/handler 调用名 |
| experiment handlers | `src/runtime/backtest.rs` | `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` | 本批不移动 |
| parameter grid | `src/runtime/backtest.rs` | `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides` | 不改去重、默认 base、负数校验或 variant 上限 |
| backtest execution bridge | `src/runtime/backtest/execution_start.rs` + `src/runtime/mod.rs` | `execute_backtest_request` | 只允许父级 runtime 内部复用，不新增公开 API |
| experiment persistence | `src/runtime_persistence.rs` | `persist_experiment_record`、`list_experiment_records`、`load_experiment_record_from_state` | 不改 memory -> file lookup、scoped key 或 storage path |
| response mapping | `src/runtime_response_mapping.rs` | `experiment_list_item_from_record`、`experiment_detail_response_from_record`、`experiment_sweep_axes` | 不迁移 response mapping owner |
| schema | `src/frontend_api_types.rs` | `FrontendExperimentRequest`、`ExperimentRecord`、`ExperimentListItem`、`ExperimentDetailResponse` | 不改 JSON schema |
| AppState | `src/lib.rs` | `experiments`、`experiment_store_dir`、`backtest_store_dir`、`transient_backtest_store_dir` | 不迁移 state owner 或锁 owner |
| audit | `src/graph_audit.rs` | `GraphAuditAction::ExperimentCreated`、`persist_graph_audit_entry` | save 时保留现有 audit 语义 |

---

## 等价冻结项

| 行为 | 当前语义 | 证据 |
| --- | --- | --- |
| route 入口 | experiment routes 仍由 `backend.runtime.routes::register_routes` 直接注册 | `src/backend/runtime/routes.rs` |
| capability guard | 创建 sweep 前继续校验 capability context 与 runtime config capability | `start_backtest_experiment` |
| execution assumption guard | backtest options 中的 execution assumptions override 继续先校验 | `validate_backtest_execution_assumption_overrides` |
| graph_json guard | 缺失 `graph_json` 继续返回 `bad_request` | `start_backtest_experiment` |
| grid normalization | 空网格报错；空轴回退 base；重复值去重；fee/slippage 负数报错 | `build_experiment_overrides` |
| variant limit | 展开数量继续受 `MAX_EXPERIMENT_VARIANTS` 限制 | `build_experiment_overrides` |
| variant execution | 每个 variant 继续构造 `FrontendRunRequest` 并调用 `execute_backtest_request` | `start_backtest_experiment` |
| preview persistence | 创建时继续持久化 experiment 元数据并写入 `state.experiments` | `persist_experiment_record` |
| list sorting | list 继续按 `created_at_ms` 倒序排序并分页 | `list_experiments` |
| save lifecycle | save 继续把 variant backtests 从 transient 转 persistent，并记录 audit | `save_experiment_record` |
| discard lifecycle | saved experiment 不能 discard；preview discard 清理 experiment 文件、state 和 transient variant backtests | `discard_experiment_record` |
| response schema | detail/list 继续保留 definition、variants、sweep axes、best variant summary | `runtime_response_mapping.rs` |

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo check -p quantpilot` | handler 可见性、runtime include、schema 类型 | 基线不破坏类型 |
| `cargo test --no-run` | 全测试编译 | 不引入测试编译漂移 |
| `cargo test -p quantpilot --test api_experiments` | sweep create/list/detail/save/discard、variant summaries、saved/preview lifecycle | experiment 行为不漂移 |
| `cargo test -p quantpilot --test api_backtest` | variant backtest 创建路径、record/replay/compare downstream | `execute_backtest_request` 复用桥不漂移 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence contract sibling 保护 | 本批不影响 evidence/report contract |
| `tools\check-matrix-governance.ps1` | 本基线、模块树、全量树和门禁锚点 | 治理入口不丢 |
| `tools\check-full-feature-tree.ps1` | 文件路径覆盖 | 新基线和真实文件可定位 |
| `tools\check-utf8.ps1` | 文档编码 | 新文档保持 UTF-8 |
| `git diff --check` | whitespace | 无尾随空白 |

`api_experiments` 中必须继续覆盖以下代表测试:

| 测试 | 覆盖 |
| --- | --- |
| `experiment_endpoints_expose_parameter_grid_and_variant_summaries` | sweep 创建、variant summaries、save、list、detail |
| `experiment_preview_can_be_discarded_before_save_only` | preview discard、saved conflict、transient variant 清理 |

---

## 本批次不做

- 不移动 `src/runtime/backtest.rs` 中的 experiment handler 或 helper。
- 不新建 `src/runtime/backtest/experiment_sweep.rs`。
- 不把 experiment routes 从 `src/backend/runtime/routes.rs` 移入 `src/backend/runtime/routes/backtest.rs`。
- 不改变 `/api/runtime/experiments/*` 的 path、method、payload、response schema 或 error code。
- 不迁移 `execute_backtest_request`、`runtime.backtest.execution_start`、record_store、replay、`backtest_compare`、artifact schema、state owner、persistence owner、response mapping owner、schema owner、frontend caller 或 report owner。
- 不拆 `experiment_sweep_axes`、`persist_experiment_record`、`load_experiment_record_from_state`、`experiment_detail_response_from_record` 等共享 owner。
- 不主动提出发布版本过渡或横向连接。ASCII guard: `release transition guard`。
- 不宣称 `runtime.backtest.experiment_sweep` 已完成抽离。

---

## 后续判断

若继续本子叶，下一步才允许做 `runtime.backtest.experiment_sweep` 抽离方案，即 BE-001V-02，且必须满足:

1. 只讨论是否把 experiment handler/helper 迁入计划目标文件，例如 `src/runtime/backtest/experiment_sweep.rs`。
2. 保持 `crate::runtime::{start_backtest_experiment, list_experiments, get_experiment_detail, save_experiment_record, discard_experiment_record}` 兼容出口不变。
3. 明确 route 真实 owner 当前仍是 `backend.runtime.routes`；若要移动 route facade，必须单独说明适配性并纳入方案。
4. 保持 `execute_backtest_request` 只作为父级 runtime 内部复用桥，不允许 sibling 直接横向连接。
5. 保持 `api_experiments`、`api_backtest` 和 `api_evidence_contract` 作为等价证据。
6. 若引入新文件，必须同步模块树、全量树和治理门禁。

---

## 验证计划

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
git diff --check
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
```

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep` 已建立基线时，必须说明本批 `no code movement`，只冻结 experiment routes、参数网格、execution_start 复用桥、experiment persistence、save/discard lifecycle、audit、response mapping 和排除边界。不得宣称 handler 已迁移、`src/runtime/backtest/experiment_sweep.rs` 已存在、route facade 已迁移、record_store、replay、compare、artifact schema owner、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `107-runtime.backtest.experiment_sweep单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树出现 `runtime.backtest.experiment_sweep` 白箱节点。
3. 全量树能定位本基线和真实 runtime/backtest 文件。
4. 治理门禁能发现本基线文件、`no code movement`、下一候选 BE-001V-02、禁止迁移边界和回归证据缺失。
5. 后续 experiment_sweep 抽离必须引用本基线，不得绕过父模块直接迁移 handler 或 route facade。
