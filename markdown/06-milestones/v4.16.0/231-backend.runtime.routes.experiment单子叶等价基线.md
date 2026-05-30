# v4.16.0 backend.runtime.routes.experiment 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001BS-01
> 基准: `230-backend.runtime.routes第二轮父叶残余判断.md`、`127-backend.runtime.routes父叶残余判断.md`、`126-runtime.backtest父叶残余判断.md`、`125-runtime.backtest.experiment_sweep第三轮父叶残余判断.md`
> 判定: 建立 `backend.runtime.routes.experiment` 单子叶等价基线。当前只冻结 experiment route group 的 path、method、handler owner、父级委托、测试证据和禁止迁移边界；本批 `no code movement`，`stop_split: pending`。下一步只能进入 BE-001BS-02 抽离方案。
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BS-01 experiment route group 单子叶等价基线 | 扩展 |
| 规范矩阵 | route owner、handler owner、父子通信、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.experiment` | 新增白箱节点 |
| 模块树 | `backend.runtime.routes.experiment` | 建立单子叶基线 |

---

## 选择理由

`backend.runtime.routes.experiment` 是 BE-001BR-01 后最高价值的下一候选:

1. 父 aggregate 仍直接持有四条 experiment route / 五个 handler 绑定。
2. handler 域 `runtime.backtest.experiment_sweep` 已完成当前范围 closeout，route facade 可以单独抽离而不碰 handler。
3. `tests/api_experiments.rs` 覆盖 create/list/detail/save/discard 与 transient cleanup，适合先冻结等价基线。
4. 本批只建立基线，不创建 `src/backend/runtime/routes/experiment.rs`，能让后续抽离方案保持最小迁移。

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.experiment` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `backend.runtime.routes.experiment` |
| 父模块 | `backend.runtime.routes` |
| route aggregate | `src/backend/runtime/routes.rs` |
| planned route child | `src/backend/runtime/routes/experiment.rs` |
| handler owner | `src/runtime/backtest/experiment_sweep.rs` |
| start handler owner | `src/runtime/backtest/start_orchestration.rs` |
| record lifecycle owner | `src/runtime/backtest/record_lifecycle.rs` |
| handler facade | `src/runtime/mod.rs` |
| app state owner | `AppState` |
| 主测试 | `tests/api_experiments.rs` |
| 下一批次 | BE-001BS-02 抽离方案 |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `FrontendExperimentRequest` | frontend、API caller、tests | 不改变 graph_json、runtime_config、backtest_options、parameter_grid 或 experiment_name 语义 |
| 输入 | experiment id | path param | 不改变 detail/save/discard scoped lookup |
| 输入 | pagination query | list route | 不改变 `PaginatedResponse<ExperimentListItem>` |
| 输入 | `AppState` | backend runtime state | 不迁移 `experiments`、`backtests`、store dir 或锁顺序 |
| 输出 | `ExperimentDetailResponse` | frontend、tests | 不改变 create/detail/save response shape |
| 输出 | `PaginatedResponse<ExperimentListItem>` | frontend、tests | 不改变 sorting、saved flag、variant_count 或 best_backtest_id |
| 输出 | `DiscardRuntimeArtifactResponse` | frontend、tests | 不改变 preview-only discard 与 saved conflict 语义 |

---

## route owner 基线

| route | method | handler | 当前处理 |
| --- | --- | --- | --- |
| `/api/runtime/experiments/backtest-sweep` | POST | `start_backtest_experiment` | 冻结 path/method，不移动 handler |
| `/api/runtime/experiments` | GET | `list_experiments` | 冻结 list response 与 pagination |
| `/api/runtime/experiments/:experiment_id/save` | POST | `save_experiment_record` | 冻结 saved transition 与 variant persistence |
| `/api/runtime/experiments/:experiment_id` | GET | `get_experiment_detail` | 冻结 detail lookup 与 not found semantics |
| `/api/runtime/experiments/:experiment_id` | DELETE | `discard_experiment_record` | 冻结 preview cleanup 与 saved conflict |

---

## 关键 public 方法

| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `backend.runtime.routes::register_routes` | Axum Router | runtime routes | `backend.runtime` | 不得在本批创建 experiment route child |
| `start_backtest_experiment` | user id、`AppState`、`FrontendExperimentRequest` | `ExperimentDetailResponse` | route aggregate | 不得迁移 handler 或改变 variant execution |
| `list_experiments` | `AppState`、pagination | paginated list | route aggregate | 不得改变 sorting 或 saved projection |
| `get_experiment_detail` | user id、experiment id | experiment detail | route aggregate | 不得绕过 scoped lookup |
| `save_experiment_record` | user id、experiment id | experiment detail | route aggregate | 不得改变 variant backtest persistence |
| `discard_experiment_record` | user id、experiment id | discard response | route aggregate | 不得改变 preview-only guard |

---

## 父子通信规则

```text
backend.runtime
  -> backend.runtime.routes
  -> planned backend.runtime.routes.experiment
  -> crate::runtime::{start_backtest_experiment, list_experiments, get_experiment_detail, save_experiment_record, discard_experiment_record}
  -> runtime.backtest.experiment_sweep handler owners
```

`backend.runtime.routes.experiment` 只能经父级 `backend.runtime.routes` 暴露 experiment routes。handler owner 仍是 `src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest/start_orchestration.rs` 和 `src/runtime/backtest/record_lifecycle.rs`；状态 owner 仍是 `AppState`。不得横向接管 evidence、report_ops、event_stream、backtest compare、artifact schema、frontend caller 或 executor。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

---

## 状态与持久化边界

| 状态/持久化 | 当前 owner | 基线约束 |
| --- | --- | --- |
| experiment memory | `AppState.experiments` | 不迁移、不改变 scoped key |
| variant backtests | `AppState.backtests` | 不迁移、不改变 transient cleanup |
| experiment store | `state.experiment_store_dir` / runtime persistence owner | 不迁移 file layout |
| backtest store | `state.backtest_store_dir` / transient store | 不改变 save/discard 迁移顺序 |
| audit store | `state.audit_store_dir` | 不改变 saved experiment audit |
| schema owner | `src/frontend_api_types.rs` | 不修改 request/response schema |

---

## 本批次不做

- 不移动 `src/backend/runtime/routes.rs` 中任何 route。
- 不创建 `src/backend/runtime/routes/experiment.rs`。
- 不迁移 `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record` 或 `discard_experiment_record`。
- 不修改 `src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest/start_orchestration.rs` 或 `src/runtime/backtest/record_lifecycle.rs`。
- 不迁移 `AppState`、schema owner、frontend caller、runtime persistence owner、artifact schema、compare owner、evidence、report_ops、event_stream 或 release transition guard。

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo fmt --check` | Rust 格式 | 后续实际抽离不制造格式漂移 |
| `cargo check -p quantpilot` | Rust 模块与 Axum route 类型 | route target 类型不漂移 |
| `cargo test --no-run` | 测试编译 | experiment route handler 仍可编译 |
| `cargo test -p quantpilot --test api_experiments` | experiment contract | create/list/detail/save/discard 与 saved conflict 不漂移 |
| `cargo test -p quantpilot --test api_backtest` | backtest 邻接路线 | experiment variants 与 backtest record 邻接不漂移 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence 邻接路线 | 父 aggregate 邻接 route 不被误伤 |
| `cargo test -p quantpilot --test api_run` | runtime run 邻接路线 | run route child 委托不漂移 |
| `tools\check-utf8.ps1` | 文档编码 | 新增基线保持 UTF-8 |
| `tools\check-matrix-governance.ps1` | 治理门禁 | 基线、模块树、全量树引用不缺失 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 | 新基线和真实文件可定位 |
| `git diff --check` | diff whitespace | 本批没有空白错误 |

---

## 下一步

1. BE-001BS-02 只能建立 `backend.runtime.routes.experiment` 抽离方案。
2. 抽离方案必须继续保持 `no code movement`，只允许规划 route facade 最小迁移，不迁移 handler。
3. 若后续进入实际抽离，必须保留父级 `backend.runtime.routes` 委托和全部 route path/method 等价。

---

## 幻觉检查点

AI 声称 BE-001BS-01 完成时，必须说明: 本批只建立 `backend.runtime.routes.experiment` 单子叶等价基线，且为 `no code movement`；experiment route 尚未抽离，planned `src/backend/runtime/routes/experiment.rs` 尚未创建，handler、`AppState`、schema owner、frontend caller、runtime persistence owner 和 release transition guard 均未改变。不得宣称 `backend.runtime.routes` 父叶完成、experiment handler 已迁移、整理或重构已经完成。

---

## 验收标准

1. `231-backend.runtime.routes.experiment单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `backend.runtime.routes.experiment` 白箱节点，包含 route、handler、state/persistence 和排除边界。
3. 治理门禁能发现本文档、`no code movement`、下一批 BE-001BS-02、关键 route/handler 和测试证据缺失。
