# v4.16.0 runtime.backtest.experiment_sweep.start_orchestration 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 标准。  
> 批次: BE-001Y-01。  
> 前置: `115-runtime.backtest.experiment_sweep父叶残余判断.md`、`110-runtime.backtest.experiment_sweep单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 建立 `runtime.backtest.experiment_sweep.start_orchestration` 单子叶等价基线；本批只冻结 `start_backtest_experiment` 的创建编排边界、输入输出、guard、variant request、execution bridge、preview persistence 和排除边界，不移动代码。  
> 代码动作: `no code movement`。下一步只能进入 BE-001Y-02 抽离方案。

---

## 选择理由

`runtime.backtest.experiment_sweep` 父叶在 BE-001X-01 后仍保持 `stop_split: false`。`parameter_grid` 已完成 closeout 并设置 `stop_split: true`，剩余风险最高的是 `start_backtest_experiment` 创建路径。

1. 它同时接触 capability guard、runtime config guard、QS compile、base assumptions、variant request assembly、`execute_backtest_request` 复用桥和 preview experiment persistence。
2. 它是 experiment sweep 写入链路的入口，错误扩大会直接影响 variant backtest、experiment response 和 transient record。
3. 它不拥有 list/detail/save/discard 的 record lifecycle；后者只能在本子叶抽离后重新判断。
4. 它不拥有 route registration、schema、state owner、persistence owner、response mapping owner、audit owner 或 frontend caller。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001Y 从 experiment_sweep 父叶残余判断进入 start orchestration 等价基线 | 推进 |
| 规范矩阵 | 创建编排 guard、QS compile、variant request、execution bridge、preview persistence | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.start_orchestration` | 新增基线 |
| 模块树 | `runtime.backtest.experiment_sweep.start_orchestration` | 新增计划节点 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.start_orchestration` |
| 父模块 | `runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep.start_orchestration` |
| 当前真实文件 | `src/runtime/backtest/experiment_sweep.rs` |
| 当前父级子文件 | `src/runtime/backtest/parameter_grid.rs` |
| 当前父级子坐标 | `runtime.backtest.experiment_sweep.parameter_grid` |
| 当前复用桥 | `src/runtime/backtest/execution_start.rs` 的 `execute_backtest_request` |
| 当前 route owner | `src/backend/runtime/routes.rs` |
| 当前 re-export owner | `src/runtime/mod.rs` |
| 当前 drained parent include | `src/runtime/backtest.rs` |
| 当前 persistence owner | `src/runtime_persistence.rs` |
| 当前 response mapping owner | `src/runtime_response_mapping.rs` |
| 当前 schema owner | `src/frontend_api_types.rs` |
| 目标方法 | `start_backtest_experiment` |
| 保留 sibling | `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 保留后续候选 | `record_lifecycle` |
| 输入类型 | `auth::UserId`、`AppState`、`FrontendExperimentRequest`、`FrontendBacktestOptions` |
| 输出类型 | `ExperimentDetailResponse`、`ExperimentRecord`、`ExperimentVariantSummary`、`FrontendRunRequest` |
| 代表测试 | `cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract` |
| 治理门禁 | `tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `UserId` | auth middleware | 必须继续用于 variant execution 和 scoped experiment cache key |
| 输入 | `AppState` | backend state | 只借用 backtest execution bridge、experiment store、in-memory experiment cache，不迁移 state owner |
| 输入 | `FrontendExperimentRequest` | `POST /api/runtime/experiments/backtest-sweep` | 必须保留 actor、capability_context、runtime_config、graph_json、runtime_targets、backtest_options、parameter_grid |
| 输出 | `ExperimentDetailResponse` | frontend/tests | 必须继续通过 `experiment_detail_response_from_record` 生成 |
| 输出 | preview `ExperimentRecord` | experiment persistence/cache | 创建时 `saved=false`，并立即持久化 experiment metadata |
| 输出 | variant summaries | `ExperimentRecord.variants` | variant id、backtest id、created_at、fee/slippage/latency、summary、execution assumptions tag 语义不变 |

---

## 等价冻结项

| 行为 | 当前语义 | 不得改变 |
| --- | --- | --- |
| capability guard | 先调用 `validate_runtime_capability_guard`，失败 code 为 `capability_boundary_violation` | 不得后置到 variant loop 之后 |
| runtime config guard | `validate_runtime_config_capabilities` 失败 code 为 `capability_gated` | 不得吞掉 details |
| execution assumption guard | `validate_backtest_execution_assumption_overrides` 失败映射为 `bad_request` | 不得让非法 override 进入 parameter grid |
| graph requirement | `graph_json` 缺失返回 `bad_request` 和“实验请求必须包含 graph_json”语义 | 不得 fallback 到空图 |
| QS compile | `compile_runtime_protocol_via_qs(graph_json)` 在 build grid 前执行 | 不得改成 legacy compile 或跳过 protocol |
| base assumptions | `resolved_backtest_execution_assumptions` 继续用于 base fee/slippage/latency | 不得使用固定默认值替代 |
| parameter grid | 只调用父级私有 `build_experiment_overrides` | 不得重新实现 parameter grid 或跨 sibling 调用 |
| replay source | 缺失时回退 `FrontendBacktestReplaySource::HistoricalReplay` | 不得改为 deterministic/mock 默认 |
| experiment identity | `experiment_{current_time_ms()}` 与单独 `created_at_ms` | 不得改成 variant id 或 graph id |
| actor/name | `normalize_actor_identity`；`experiment_name` trim 后空值转 None | 不得保留空白 name |
| variant request | 每个 override 组装 `FrontendRunRequest`，保留 actor、capability_context、runtime_config、graph_json、runtime_targets、replay_mode、runtime_kind、symbols | 不得丢失 request 字段 |
| execution bridge | 每个 variant 调用 `execute_backtest_request(&state,&user_id,&variant_request,Some(\"{experiment_id}_v{n}\"))` | 不得横向直连 execution_start 内部 helper |
| summary fallback | 优先 artifact metrics summary，缺失时回退 `record.backtest.summary` | 不得只读取一个来源 |
| assumptions tag | 从 artifact metrics execution_assumptions 读取 `list_tag` | 不得由 request 重算 |
| variant defaults | fee/slippage 缺失回退 0.0，latency 缺失回退 0 | 不得改成 base assumptions |
| record assembly | `saved=false`，definition 包含 name、replay_source、base_execution_assumptions、parameter_grid | 不得创建即 saved |
| preview persistence | `persist_experiment_record` 写 `state.experiment_store_dir`，再写 `state.experiments` scoped cache | 不得只写内存或只写文件 |
| response mapping | 只通过 `experiment_detail_response_from_record(record)` 返回 | 不得在本叶私造 response schema |

---

## 父子通信规则

```text
backend.runtime.routes
  -> crate::runtime::start_backtest_experiment
  -> runtime.backtest.experiment_sweep.start_orchestration
     -> parameter_grid::build_experiment_overrides
     -> execution_start bridge::execute_backtest_request
     -> runtime_persistence::persist_experiment_record
     -> runtime_response_mapping::experiment_detail_response_from_record
```

`runtime.backtest.experiment_sweep.start_orchestration` 只能被父级 `runtime.backtest.experiment_sweep` 私有调用，并只能经父级 `runtime` re-export 和 `backend.runtime.routes` 暴露 API。不得让 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 本批次不做

- 不移动 `start_backtest_experiment`。
- 不新增 `src/runtime/backtest/start_orchestration.rs` 或其他子文件。
- 不修改 `src/runtime/backtest/experiment_sweep.rs`。
- 不修改 `src/runtime/backtest/parameter_grid.rs`。
- 不修改 `src/runtime/backtest/execution_start.rs` 或 `execute_backtest_request`。
- 不迁移 route registration、runtime re-export、drained parent include。
- 不迁移 persistence、response mapping、schema、state、audit、frontend caller 或测试资产。
- 不处理 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 的 record lifecycle。
- 不启动发布过渡，不提出 sibling 横向直连。
- 不进入整理或重构阶段。

---

## 后续队列

下一步只能进入 BE-001Y-02 `runtime.backtest.experiment_sweep.start_orchestration` 抽离方案。方案阶段必须先确认目标物理文件名、父级私有可见性、re-export 方式、是否需要 focused API/unit test 和 rollback 点；不得直接移动代码，也不得把 record lifecycle 混入本子叶。

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

AI 声称 `runtime.backtest.experiment_sweep.start_orchestration` 已建立基线时，必须说明本批是 `no code movement`，只冻结 `start_backtest_experiment` 的创建编排等价边界。不得宣称 start handler 已抽离、`src/runtime/backtest/start_orchestration.rs` 已存在、record lifecycle 已抽离、route registration、schema、state、persistence、response mapping、audit、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树新增 `runtime.backtest.experiment_sweep.start_orchestration` 计划白箱节点。
3. 全量树覆盖本基线文档与下一步 BE-001Y-02。
4. 治理门禁能发现本基线、`no code movement`、`start_backtest_experiment`、guard、variant request、execution bridge、preview persistence、禁止迁移边界和下一步。
5. 代表性治理门禁与 Rust 编译门禁继续通过。
