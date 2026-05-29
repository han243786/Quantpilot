# v4.16.0 runtime.backtest.experiment_sweep.record_lifecycle 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 标准。  
> 批次: BE-001AA-01。  
> 前置: `120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md`、`119-runtime.backtest.experiment_sweep.start_orchestration单叶closeout.md`、`114-runtime.backtest.experiment_sweep.parameter_grid单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 建立 `runtime.backtest.experiment_sweep.record_lifecycle` 单子叶等价基线；本批只冻结 experiment list/detail/save/discard 的 record lifecycle 边界、输入输出、持久化/state cache/audit/response mapping 排除边界，不移动代码。  
> 代码动作: `no code movement`。下一步只能进入 BE-001AA-02 抽离方案。

---

## 选择理由

BE-001Z-01 已确认 `runtime.backtest.experiment_sweep.parameter_grid` 与 `runtime.backtest.experiment_sweep.start_orchestration` 均已 closeout 并设置 `stop_split: true`，但父叶仍为 `stop_split: false`。父叶剩余的真实高聚合边界是 experiment record lifecycle:

1. `list_experiments` 与 `get_experiment_detail` 负责读路径、排序分页、用户作用域查找和 response projection。
2. `save_experiment_record` 与 `discard_experiment_record` 负责写路径、saved conflict、variant backtest 固化、transient cleanup、state cache 和 audit。
3. 四个 handler 共享 experiment record owner、persistence helper、response mapping owner 和 AppState cache，作为整体冻结比拆成 read/save/discard 三个微叶更稳。
4. 本子叶不拥有 route registration、schema owner、persistence owner、response mapping owner、audit owner、state owner、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AA 从 experiment_sweep 父叶残余判断进入 record lifecycle 等价基线 | 推进 |
| 规范矩阵 | list/detail/save/discard 生命周期语义、保存/丢弃约束、variant cleanup | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.record_lifecycle` | 新增基线 |
| 模块树 | `runtime.backtest.experiment_sweep.record_lifecycle` | 新增计划节点 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.record_lifecycle` |
| 父模块 | `runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep.record_lifecycle` |
| 当前真实文件 | `src/runtime/backtest/experiment_sweep.rs` |
| 已关闭 sibling | `runtime.backtest.experiment_sweep.parameter_grid`、`runtime.backtest.experiment_sweep.start_orchestration` |
| 当前 sibling 文件 | `src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/start_orchestration.rs` |
| 当前 route owner | `src/backend/runtime/routes.rs` |
| 当前 runtime re-export owner | `src/runtime/mod.rs` |
| 当前 drained parent include | `src/runtime/backtest.rs` |
| 当前 persistence owner | `src/runtime_persistence.rs` |
| 当前 response mapping owner | `src/runtime_response_mapping.rs` |
| 当前 schema owner | `src/frontend_api_types.rs` |
| 当前 transient backtest helper owner | `src/backtest_artifacts.rs` |
| 目标方法 | `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 输入类型 | `AppState`、`auth::UserId`、`PaginationQuery`、`Path<String>` |
| 输出类型 | `PaginatedResponse<ExperimentListItem>`、`ExperimentDetailResponse`、`DiscardRuntimeArtifactResponse` |
| 代表测试 | `cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract` |
| 治理门禁 | `tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `PaginationQuery` | experiment list route | 必须继续通过 `paginate` 统一处理，不私改分页 schema |
| 输入 | `experiment_id` | detail/save/discard route path | 必须继续用于 scoped lookup、文件清理和 response id |
| 输入 | `auth::UserId` | auth middleware | detail/save/discard 必须继续使用 user scoped key，不允许跨用户读取 |
| 输入 | `AppState` | backend state | 只借用 experiment/backtest store dir、in-memory cache 和 audit dir，不迁移 state owner |
| 输出 | `PaginatedResponse<ExperimentListItem>` | frontend/tests | 必须保持 created_at 倒序后再分页 |
| 输出 | `ExperimentDetailResponse` | frontend/tests | 必须继续通过 `experiment_detail_response_from_record` 生成 |
| 输出 | persisted variant backtests | `state.backtest_store_dir` | save 时必须先持久化每个 variant backtest，再删除 transient |
| 输出 | experiment record cache/file | `state.experiments` / `state.experiment_store_dir` | save 时 `saved=true` 并写回 scoped cache；discard 时删除未保存 experiment |
| 输出 | graph audit entry | audit store | actor 存在时 save 必须写 `GraphAuditAction::ExperimentCreated` |
| 输出 | `DiscardRuntimeArtifactResponse` | frontend/tests | discard 成功必须返回原 `experiment_id` 与 kind `experiment` |

---

## 等价冻结项

| 方法 | 当前语义 | 不得改变 |
| --- | --- | --- |
| `list_experiments` | `list_experiment_records` 读取文件，映射 `experiment_list_item_from_record`，按 `created_at_ms` 倒序，再调用 `paginate` | 不得改排序方向、分页顺序、response schema 或读取 owner |
| `get_experiment_detail` | `load_experiment_record_from_state(&state,&user_id,&experiment_id)` 后调用 `experiment_detail_response_from_record` | 不得绕过 scoped lookup 或私造 detail response |
| `save_experiment_record` | 先加载 experiment record，再逐个加载 variant backtest、持久化正式 backtest、删除 transient backtest | 不得在 variant 固化失败后继续写 saved experiment |
| `save_experiment_record` | 设置 `record.saved = true`，持久化 experiment，再写 `state.experiments` scoped cache | 不得只写文件或只写内存，也不得创建新 experiment id |
| `save_experiment_record` | actor 存在时写 graph audit，action 为 `ExperimentCreated`，message 为 `Saved backtest sweep ...` | 不得吞掉 audit 失败或改 action 语义 |
| `discard_experiment_record` | saved experiment 返回 `StatusCode::CONFLICT`，未保存 experiment 才允许丢弃 | 不得允许 saved experiment discard |
| `discard_experiment_record` | 使用 `sanitize_storage_path_segment` 生成 experiment 文件路径 | 不得恢复未清洗路径拼接 |
| `discard_experiment_record` | 先从 `state.experiments` scoped cache 移除，再删除 experiment file | 不得遗漏 cache 或 file cleanup |
| `discard_experiment_record` | 只把尚未存在于正式 backtest store 的 variant 视为 transient，并从 `state.backtests` scoped cache 与 transient store 清理 | 不得误删已保存 variant backtest |
| `discard_experiment_record` | 返回 `DiscardRuntimeArtifactResponse { discarded_id, discarded_kind: "experiment" }` | 不得改 kind 或 response field |

---

## 父子通信规则

```text
backend.runtime.routes
  -> crate::runtime::{list_experiments,get_experiment_detail,save_experiment_record,discard_experiment_record}
  -> runtime.backtest.experiment_sweep.record_lifecycle
     -> runtime_persistence::{list/load/persist experiment/backtest record}
     -> backtest_artifacts::delete_transient_backtest_record
     -> runtime_response_mapping::{experiment_list_item_from_record,experiment_detail_response_from_record}
     -> graph audit helper
     -> AppState scoped cache
```

`runtime.backtest.experiment_sweep.record_lifecycle` 只能被父级 `runtime.backtest.experiment_sweep` 私有调用，并只能经父级 `runtime` re-export 和 `backend.runtime.routes` 暴露 API。不得让 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 本批次不做

- 不移动 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record`。
- 不新增 `src/runtime/backtest/record_lifecycle.rs`。
- 不修改 `src/runtime/backtest/experiment_sweep.rs`。
- 不修改 `src/runtime/backtest/parameter_grid.rs` 或 `src/runtime/backtest/start_orchestration.rs`。
- 不修改 `src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs` 或 `src/backtest_artifacts.rs`。
- 不迁移 route registration、runtime re-export、drained parent include。
- 不改变 saved conflict、created_at 倒序、pagination、scoped lookup、variant backtest persistence、transient cleanup、state cache、audit 或 response mapping。
- 不启动发布过渡，不提出 sibling 横向直连。
- 不进入整理或重构阶段。

---

## 后续队列

下一步只能进入 BE-001AA-02 `runtime.backtest.experiment_sweep.record_lifecycle` 抽离方案。方案阶段必须先确认目标物理文件名、父级私有可见性、re-export 方式、是否需要 focused API/unit test 和 rollback 点；不得直接移动代码，也不得把 parameter_grid、start_orchestration、route registration、schema、persistence owner、response mapping owner、audit owner 或 frontend caller 混入本子叶。

---

## 回归保护

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test --no-run`
- `cargo test -p quantpilot --test api_experiments`
- `cargo test -p quantpilot --test api_backtest`
- `cargo test -p quantpilot --test api_evidence_contract`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `git diff --check`

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep.record_lifecycle` 已建立基线时，必须说明本批是 `no code movement`，只冻结 list/detail/save/discard 的 record lifecycle 等价边界。不得宣称 record lifecycle 已抽离、`src/runtime/backtest/record_lifecycle.rs` 已存在、route registration、schema、state、persistence、response mapping、audit、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `121-runtime.backtest.experiment_sweep.record_lifecycle单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树新增 `runtime.backtest.experiment_sweep.record_lifecycle` 计划白箱节点。
3. 全量树覆盖本基线文档与下一步 BE-001AA-02。
4. 治理门禁能发现本基线、`no code movement`、四个目标 handler、saved conflict、variant backtest persistence、transient cleanup、state cache、audit、response mapping、禁止迁移边界和下一步。
5. 代表性治理门禁与 Rust 编译门禁继续通过。
