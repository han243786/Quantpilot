# v4.16.0 runtime.backtest.experiment_sweep 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001V-02。  
> 基准: `107-runtime.backtest.experiment_sweep单子叶等价基线.md`、`106-runtime.backtest.replay单叶closeout.md`、`77-runtime.backtest单叶closeout.md`。  
> 判定: 建立 `runtime.backtest.experiment_sweep` 实际抽离方案；本批只落方案和门禁要求，`no code movement`，不移动代码。  
> 下一步: BE-001V-03 实际抽离记录。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001V experiment_sweep 从等价基线进入实际抽离方案 | 推进 |
| 规范矩阵 | 父级 re-export、route aggregate 保留、execution_start 复用桥、persistence/schema/mapping owner 保留、最小迁移边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` | 细化 |
| 模块树 | `runtime.backtest.experiment_sweep` | 补充实施计划 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep` |
| 当前真实文件 | `src/runtime/backtest.rs`、`src/runtime/backtest/execution_start.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/lib.rs` |
| 计划目标文件 | `src/runtime/backtest/experiment_sweep.rs` |
| public handler | `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 计划同迁 helper | `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides` |
| 保留 shared helper | `execute_backtest_request`、`persist_experiment_record`、`list_experiment_records`、`load_experiment_record_from_state`、`persist_backtest_record`、`delete_transient_backtest_record`、`experiment_list_item_from_record`、`experiment_detail_response_from_record`、`experiment_sweep_axes`、`persist_graph_audit_entry`、`build_graph_audit_entry` |
| 保留 public 类型 | `FrontendExperimentRequest`、`FrontendExecutionAssumptionSweepGrid`、`FrontendExecutionAssumptionOverrides`、`FrontendBacktestReplaySource`、`ExperimentRecord`、`ExperimentDefinitionSummary`、`ExperimentVariantSummary`、`ExperimentListItem`、`ExperimentDetailResponse`、`DiscardRuntimeArtifactResponse` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 抽离目标

第一轮实际抽离只允许移动 experiment sweep handler 和它们私有的参数网格 helper。

| 函数 | 当前职责 | 计划归属 | 保持不变 |
| --- | --- | --- | --- |
| `normalize_experiment_float_axis` | fee/slippage 轴去重、空轴回退、负数拒绝 | `src/runtime/backtest/experiment_sweep.rs` | error code、field name、base fallback |
| `normalize_experiment_latency_axis` | latency 轴去重、空轴回退 | `src/runtime/backtest/experiment_sweep.rs` | base fallback、去重顺序 |
| `build_experiment_overrides` | 参数网格展开、variant 上限、base assumptions | `src/runtime/backtest/experiment_sweep.rs` | `MAX_EXPERIMENT_VARIANTS`、空网格拒绝、展开顺序 |
| `start_backtest_experiment` | 创建 sweep、调用 backtest 执行桥、生成 preview record | `src/runtime/backtest/experiment_sweep.rs` | capability/config guard、`execute_backtest_request` 复用桥、preview persistence |
| `list_experiments` | 实验列表读取、倒序排序、分页 | `src/runtime/backtest/experiment_sweep.rs` | `list_experiment_records` owner、pagination 语义 |
| `get_experiment_detail` | scoped detail lookup | `src/runtime/backtest/experiment_sweep.rs` | `load_experiment_record_from_state` owner |
| `save_experiment_record` | 保存实验、持久化 variant backtests、清理 transient、audit | `src/runtime/backtest/experiment_sweep.rs` | backtest persistence owner、audit owner、saved 状态 |
| `discard_experiment_record` | 丢弃未保存 experiment、清理 transient variant backtests | `src/runtime/backtest/experiment_sweep.rs` | saved conflict、path sanitize、state cleanup |

本方案不移动 experiment routes。`/api/runtime/experiments/*` 当前仍由 `src/backend/runtime/routes.rs` 直接注册；本批只通过 `crate::runtime::*` 兼容出口保持 route 调用名不变。

---

## 实施方案

1. 新建 `src/runtime/backtest/experiment_sweep.rs`，只承载 experiment sweep 的 5 个 handler 和 3 个参数网格 helper。
2. 从 `src/runtime/backtest.rs` 移出上述 8 个函数。
3. 在 `src/runtime/mod.rs` 增加私有子模块和受控 re-export:

```rust
#[path = "backtest/experiment_sweep.rs"]
mod backtest_experiment_sweep;
pub(crate) use backtest_experiment_sweep::{
    discard_experiment_record, get_experiment_detail, list_experiments, save_experiment_record,
    start_backtest_experiment,
};
```

4. 保持 `src/backend/runtime/routes.rs` 不变，route aggregate 继续调用 `runtime_handlers::{start_backtest_experiment, list_experiments, get_experiment_detail, save_experiment_record, discard_experiment_record}`。
5. 保持 `execute_backtest_request` 留在 `runtime.backtest.execution_start`，只通过父级 runtime 内部桥被 experiment sweep 使用。
6. 保持 `src/runtime_persistence.rs` 继续拥有 experiment/backtest persistence helper，不私有化到 experiment_sweep。
7. 保持 `src/runtime_response_mapping.rs` 继续拥有 `experiment_list_item_from_record`、`experiment_detail_response_from_record` 和 `experiment_sweep_axes`。
8. 保持 `src/frontend_api_types.rs` 继续拥有 request/response/schema 类型。
9. 保持 `src/lib.rs` 继续拥有 `AppState`、`state.experiments`、store dir 和锁 owner。
10. 如果 `src/runtime/backtest.rs` 在本批迁移后变为空文件，本批只允许让它成为 drained parent include；删除旧文件或移除 `include!("backtest.rs")` 必须在后续 closeout/父叶残余判断中单独确认。
11. 代码移动后再补 BE-001V-03 实际抽离记录，并用 `api_experiments`、`api_backtest`、`api_evidence_contract` 证明等价。

---

## 明确排除

| 排除项 | 原因 |
| --- | --- |
| experiment route facade 迁移 | 当前 route 真实 owner 是 `src/backend/runtime/routes.rs`；移动到 backtest route facade 或新 facade 必须另起方案 |
| `runtime.backtest.execution_start` | `execute_backtest_request` 是复用桥，不归 experiment_sweep 私有 |
| `runtime.backtest.record_store` | backtest list/detail/save/discard 已 closeout，不回流到 experiment_sweep |
| `runtime.backtest.replay` | replay 已 closeout，不回流到 experiment_sweep |
| `backtest_compare` | compare owner 在 `src/backtest_compare.rs`，不迁移 |
| experiment persistence owner | `persist_experiment_record`、`list_experiment_records`、`load_experiment_record_from_state` 继续留在 `src/runtime_persistence.rs` |
| backtest persistence/transient owner | `persist_backtest_record`、`delete_transient_backtest_record` 不私有化 |
| response mapping owner | `experiment_list_item_from_record`、`experiment_detail_response_from_record`、`experiment_sweep_axes` 继续留在 `src/runtime_response_mapping.rs` |
| schema owner | `FrontendExperimentRequest`、`ExperimentRecord`、`ExperimentDetailResponse` 等继续留在 `src/frontend_api_types.rs` |
| AppState / lock owner | `state.experiments`、store dirs 和锁 owner 不迁移 |
| graph audit owner | `persist_graph_audit_entry`、`build_graph_audit_entry` 不迁移 |
| frontend API | 不改 path、payload、flow、response schema 或 caller |
| 发布过渡 | 不主动提出横向连接或性能旁路。ASCII guard: `release transition guard` |

---

## 适配性风险与处理

| 风险 | 处理 |
| --- | --- |
| `include!("backtest.rs")` 与 re-export 重名 | 先从 `src/runtime/backtest.rs` 移出 handler/helper，再在 `src/runtime/mod.rs` re-export，避免 duplicate definition |
| 子模块无法访问 `execute_backtest_request` | 保持父级 runtime 内部桥导入，不改变为 public API；用 `cargo check -p quantpilot` 校验可见性 |
| 参数网格行为漂移 | `api_experiments` 必须覆盖 empty/default/dedup/variant summary 代表链路 |
| variant backtest id 或 suffix 漂移 | `start_backtest_experiment` 继续用 `experiment_{timestamp}` 与 `{}_v{index}` suffix |
| save lifecycle 漏清理 transient | `save_experiment_record` 必须继续调用 `persist_backtest_record` 与 `delete_transient_backtest_record` |
| discard lifecycle 漏清理 in-memory/state | `discard_experiment_record` 必须继续删除 experiment 文件、state 和 transient variant backtests |
| route owner 被误迁移 | 本批 `src/backend/runtime/routes.rs` 不动；任何 route facade 调整必须另起方案 |
| mapping/schema 被误私有化 | response mapping 和 schema owner 保持原位，用 `cargo test --no-run` 与代表 API 测试保护 |

---

## 中止条件

进入代码移动时，只要出现以下任一情况，应中止并回到方案讨论:

1. 需要改变 `/api/runtime/experiments/*` path、method、payload、response schema 或 error code。
2. 需要移动 experiment route registration。
3. 需要把 persistence、response mapping、schema、AppState、audit 或 backtest transient owner 搬入 experiment_sweep 私有模块。
4. 需要改变 `execute_backtest_request` 可见性为公开 API，或让 sibling 直接横向调用。
5. 需要同时迁移 record_store、replay、compare、report/mutation 或 frontend caller。
6. `cargo check -p quantpilot` 暴露的可见性问题无法通过父级 re-export 或显式 import 解决。
7. `cargo test -p quantpilot --test api_experiments`、`api_backtest` 或 `api_evidence_contract` 出现行为回归。

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

下一批应进入 BE-001V-03 `runtime.backtest.experiment_sweep` 实际抽离记录: 按本方案移动 5 个 handler 和 3 个参数网格 helper 到计划目标文件，保持父级 re-export、route aggregate、execution_start 复用桥、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend route 和发布过渡边界不变。完成后再做单叶 closeout，判断本叶是否值得继续细拆。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep` 已有抽离方案时，必须说明本批 `no code movement`。不得宣称 experiment handler/helper 已迁移、route facade 已迁移、record_store、replay、compare、persistence owner、response mapping owner、schema owner、state owner、frontend caller、发布过渡、整理或重构已经完成。ASCII guard: `release transition guard`。

---

## 验收标准

1. `108-runtime.backtest.experiment_sweep抽离方案.md` 进入 v4.16 里程碑索引。
2. 模块树 `runtime.backtest.experiment_sweep` 节点标记实际抽离方案已建立，但代码尚未移动。
3. 全量树能定位本方案、真实文件和下一步计划目标。
4. 治理门禁能发现本方案文件、`no code movement`、下一步 BE-001V-03、禁止迁移边界和回归证据缺失。
5. 后续 BE-001V 实际抽离必须引用本方案，不得把 route facade、execution_start、record_store、replay、compare、persistence、mapping、schema、state 或 frontend route 混入第一轮迁移。
