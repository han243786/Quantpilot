# v4.16.0 runtime.backtest.experiment_sweep.start_orchestration 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001Y-02。  
> 基准: `116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md`、`115-runtime.backtest.experiment_sweep父叶残余判断.md`、`110-runtime.backtest.experiment_sweep单叶closeout.md`。  
> 判定: 建立 `runtime.backtest.experiment_sweep.start_orchestration` 实际抽离方案；本批只落方案和门禁要求，`no code movement`，不移动代码。  
> 下一步: BE-001Y-03 实际抽离记录。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001Y start_orchestration 从等价基线进入实际抽离方案 | 推进 |
| 规范矩阵 | 创建编排 guard、父级私有子模块、route/schema/state/persistence owner 保留、禁止横向连接 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.start_orchestration` | 细化 |
| 模块树 | `runtime.backtest.experiment_sweep.start_orchestration` | 补充实施计划 |

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
| 当前复用桥文件 | `src/runtime/backtest/execution_start.rs` |
| 计划目标文件 | `src/runtime/backtest/start_orchestration.rs` |
| 目标方法 | `start_backtest_experiment` |
| 父级保留声明 | `mod parameter_grid;`、`use parameter_grid::build_experiment_overrides;` |
| 计划新增父级声明 | `mod start_orchestration;` 与受控 `pub(crate) use start_orchestration::start_backtest_experiment;` |
| 继续保留 sibling | `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 继续保留复用桥 | `execute_backtest_request` |
| 继续保留输出类型 | `ExperimentRecord`、`ExperimentVariantSummary`、`ExperimentDetailResponse`、`FrontendRunRequest` |
| 继续保留 shared owner | `schema`、`state`、`persistence`、`response mapping`、`audit`、`frontend caller`、`route registration` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 抽离目标

下一批实际抽离只允许把 `start_backtest_experiment` 从 `src/runtime/backtest/experiment_sweep.rs` 移入 planned start_orchestration child file，也就是计划目标文件 `src/runtime/backtest/start_orchestration.rs`。父级 `runtime.backtest.experiment_sweep` 继续作为白箱父节点，负责保留 record lifecycle sibling、参数网格私有 helper 接入和受控 re-export。

| 方法 | 当前职责 | 计划归属 | 保持不变 |
| --- | --- | --- | --- |
| `start_backtest_experiment` | 创建 backtest sweep、执行 guard、编译 QS、组装 variant request、调用 execution bridge、写 preview experiment record、返回 detail response | `runtime.backtest.experiment_sweep.start_orchestration` 私有子模块 | path/method/payload/response、error code、variant id/suffix、summary fallback、preview persistence、scoped cache |
| `build_experiment_overrides` | 参数网格展开 | `runtime.backtest.experiment_sweep.parameter_grid` | 只通过父级私有 helper 调用，不复制、不横向直连 |
| `execute_backtest_request` | variant backtest execution bridge | `runtime.backtest.execution_start` | 继续由 start orchestration 通过父级可见上下文调用，不迁移 owner |
| `experiment_detail_response_from_record` | detail response projection | `runtime_response_mapping` | 不在子叶内私有化 schema 或 mapping |

---

## 实施方案

1. 在 BE-001Y-03 新建计划目标文件 `src/runtime/backtest/start_orchestration.rs`。
2. 将 `start_backtest_experiment` 原样从 `src/runtime/backtest/experiment_sweep.rs` 移入该文件。
3. 在子文件顶部使用 `use super::*;` 复用父级上下文，避免扩大 public API。
4. 父级 `src/runtime/backtest/experiment_sweep.rs` 增加私有子模块声明:

```rust
mod start_orchestration;

pub(crate) use start_orchestration::start_backtest_experiment;
```

5. 父级继续保留 `mod parameter_grid;` 与 `use parameter_grid::build_experiment_overrides;`，让 start orchestration 通过父级可见上下文调用 `build_experiment_overrides`。
6. 不移动 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record`，record lifecycle 必须等 start orchestration 抽离和 closeout 后重新判断。
7. 不改 `src/runtime/mod.rs` 的外部兼容 re-export 语义；route aggregate 仍通过 `crate::runtime::start_backtest_experiment` 进入。
8. 不改 `src/backend/runtime/routes.rs` 或 `src/backend/runtime/routes/backtest.rs` 的 route registration。
9. 不迁移 `execute_backtest_request`、`persist_experiment_record`、`experiment_detail_response_from_record`、schema、state、persistence、response mapping、audit 或 frontend caller。
10. 完成代码移动后补 BE-001Y-03 实际抽离记录，再进入 BE-001Y-04 单子叶 closeout。

---

## 必须保持的等价语义

| 行为 | 既有语义 | 不得改变 |
| --- | --- | --- |
| capability guard | `validate_runtime_capability_guard` 失败映射 `capability_boundary_violation` | 不得后置到 variant loop 后 |
| runtime config guard | `validate_runtime_config_capabilities` 失败映射 `capability_gated` | 不得吞掉 details |
| execution assumption guard | `validate_backtest_execution_assumption_overrides` 失败映射 `bad_request` | 不得让非法 override 进入 parameter grid |
| graph requirement | `graph_json` 缺失返回 `bad_request` | 不得 fallback 到空图 |
| QS compile | `compile_runtime_protocol_via_qs(graph_json)` 在 build grid 前执行 | 不得改 legacy compile 或跳过 protocol |
| base assumptions | `resolved_backtest_execution_assumptions` 生成 base fee/slippage/latency | 不得换成固定默认值 |
| parameter grid | 调用 `build_experiment_overrides(&request, &qs_protocol)` | 不得复制参数网格实现 |
| replay source | 缺失时 fallback `FrontendBacktestReplaySource::HistoricalReplay` | 不得改 mock/deterministic 默认 |
| identity | `experiment_{current_time_ms()}` 与 `created_at_ms` | 不得改成 graph id 或 variant id |
| actor/name | `normalize_actor_identity`，空白 experiment name 转 `None` | 不得保留空白 name |
| variant request | 逐个 override 组装 `FrontendRunRequest` | 不得丢失 actor、capability_context、runtime_config、graph_json、runtime_targets、backtest_options 字段 |
| execution bridge | `execute_backtest_request(&state, &user_id, &variant_request, Some("{experiment_id}_v{n}"))` | 不得横向直连 execution_start 内部 helper |
| summary fallback | 优先 artifact metrics summary，缺失时 fallback `record.backtest.summary` | 不得只读单一路径 |
| assumptions tag | 从 artifact metrics `execution_assumptions.list_tag` 读取 | 不得从 request 重算 |
| record assembly | `ExperimentRecord { saved: false, definition, variants, actor }` | 不得直接创建 saved record |
| preview persistence | 先 `persist_experiment_record`，再写 `state.experiments` scoped cache | 不得只写内存或只写文件 |
| response mapping | `experiment_detail_response_from_record(record)` | 不得在子叶私造 `ExperimentDetailResponse` |

---

## 明确排除

| 排除项 | 原因 |
| --- | --- |
| `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` | record lifecycle sibling，必须后续单独判断 |
| `parameter_grid` 内部 helper | 已完成 BE-001W-04 closeout，`stop_split: true` |
| `execute_backtest_request` | 属于 `runtime.backtest.execution_start` 复用桥 |
| route registration | route owner 仍是 `backend.runtime.routes` / backtest route facade |
| `src/runtime/mod.rs` 兼容出口语义 | 只允许保持当前 `crate::runtime::*` 行为，不扩大公开面 |
| drained parent include | `src/runtime/backtest.rs` 保持 drained parent include 事实 |
| persistence owner | `persist_experiment_record` 仍归 `src/runtime_persistence.rs` |
| response mapping owner | `experiment_detail_response_from_record` 仍归 `src/runtime_response_mapping.rs` |
| schema owner | `FrontendExperimentRequest`、`FrontendRunRequest`、`ExperimentRecord`、`ExperimentDetailResponse` 仍归 `src/frontend_api_types.rs` |
| AppState / lock owner | state、store dir 和 scoped cache owner 不迁移 |
| audit owner | start orchestration 不接管 save lifecycle audit |
| frontend caller | 不改 API path、payload、response schema 或 caller |
| 发布过渡 | 不主动提出横向连接或性能旁路。ASCII guard: `release transition guard` |

---

## 适配性风险与处理

| 风险 | 处理 |
| --- | --- |
| 子模块可见性失败 | 先用 `use super::*;`，只在必要时补显式 import，不新增 public API |
| `build_experiment_overrides` 在子模块不可见 | 保持父级 `use parameter_grid::build_experiment_overrides;`，让子模块通过 `super::*` 继承 |
| `execute_backtest_request` 可见性失败 | 保持现有父级 runtime 私有桥，不把 execution_start helper 改成 public |
| handler re-export 重名 | 父级先移除本地函数，再 `pub(crate) use start_orchestration::start_backtest_experiment;` |
| route aggregate 被误迁移 | BE-001Y-03 不改 route 文件，测试只证明调用路径等价 |
| response/schema 被误私有化 | 继续使用 `ExperimentRecord` 与 `experiment_detail_response_from_record` |
| record lifecycle 被顺手移动 | 发现 list/detail/save/discard 变更即中止并回到方案讨论 |
| 发布过渡旁路被提出 | 未收到开发者明确发布过渡指令时直接拒绝进入该路径 |

---

## 中止条件

进入代码移动时，只要出现以下任一情况，应中止并回到方案讨论:

1. 需要改变 `/api/runtime/experiments/backtest-sweep` path、method、payload、response schema 或 error code。
2. 需要迁移 route registration、schema、state、persistence、response mapping、audit 或 frontend caller。
3. 需要移动 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record`。
4. 需要改变 `build_experiment_overrides` 的展开语义、helper owner 或可见性为父级以外。
5. 需要把 `execute_backtest_request` 改成公开 API，或让 sibling 横向直连。
6. `cargo check -p quantpilot` 暴露的可见性问题无法通过私有子模块 import 解决。
7. 代表测试出现行为回归。

---

## 验证计划

实际抽离批次必须至少运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一批应进入 BE-001Y-03 `runtime.backtest.experiment_sweep.start_orchestration` 实际抽离记录: 按本方案只移动 `start_backtest_experiment` 到 planned start_orchestration child file，保留父级私有 re-export、parameter_grid helper 调用、execution_start 复用桥、route aggregate、schema、state、persistence、response mapping、audit、frontend caller 和发布过渡边界。完成后再做 BE-001Y-04 单子叶 closeout，判断 `start_orchestration` 是否设置 `stop_split: true`，并决定父叶下一候选是否进入 `record_lifecycle`。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep.start_orchestration` 已有抽离方案时，必须说明本批 `no code movement`，只是确认 BE-001Y-03 的移动边界。不得宣称 `start_backtest_experiment` 已抽离、planned start_orchestration child file 已存在、record lifecycle 已抽离、route registration、schema、state、persistence、response mapping、audit、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `117-runtime.backtest.experiment_sweep.start_orchestration抽离方案.md` 进入 v4.16 里程碑索引。
2. 模块树 `runtime.backtest.experiment_sweep.start_orchestration` 节点标记抽离方案已建立，但代码尚未移动。
3. 全量树能定位本方案、当前真实文件、计划目标文件和下一步 BE-001Y-03。
4. 治理门禁能发现本方案、`no code movement`、`start_backtest_experiment`、`use super::*`、`build_experiment_overrides`、`execute_backtest_request`、排除边界、发布过渡保护和回归证据。
5. 后续 BE-001Y-03 实际抽离必须引用本方案，不得把 record lifecycle、route、execution_start、persistence、mapping、schema、state、audit 或 frontend caller 混入第一轮迁移。
