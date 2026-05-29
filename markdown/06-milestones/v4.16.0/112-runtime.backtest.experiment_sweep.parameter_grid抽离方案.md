# v4.16.0 runtime.backtest.experiment_sweep.parameter_grid 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001W-02。  
> 基准: `111-runtime.backtest.experiment_sweep.parameter_grid单子叶等价基线.md`、`110-runtime.backtest.experiment_sweep单叶closeout.md`。  
> 判定: 建立 `runtime.backtest.experiment_sweep.parameter_grid` 实际抽离方案；本批只落方案和门禁要求，`no code movement`，不移动代码。  
> 下一步: BE-001W-03 实际抽离记录。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001W parameter_grid 从等价基线进入实际抽离方案 | 推进 |
| 规范矩阵 | 私有子模块、父级调用、参数网格等价、禁止横向连接、测试策略 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.parameter_grid` | 细化 |
| 模块树 | `runtime.backtest.experiment_sweep.parameter_grid` | 补充实施计划 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.parameter_grid` |
| 父模块 | `runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep.parameter_grid` |
| 当前真实文件 | `src/runtime/backtest/experiment_sweep.rs`、`src/frontend_api_types.rs`、`tests/api_experiments.rs`、`tests/api_backtest.rs`、`tests/api_evidence_contract.rs` |
| 计划目标文件 | `src/runtime/backtest/experiment_sweep/parameter_grid.rs` |
| public 方法 | 本节点不新增 public 方法；只给父级 `experiment_sweep` 暴露 `pub(super)` helper |
| 计划迁移 helper | `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides` |
| 保留父级方法 | `start_backtest_experiment`、`resolved_backtest_execution_assumptions`、`execute_backtest_request` |
| 保留常量 | `MAX_EXPERIMENT_VARIANTS` |
| 保留类型 | `FrontendExperimentRequest`、`FrontendExecutionAssumptionSweepGrid`、`RuntimeProtocolCoreConfig`、`FrontendExecutionAssumptionOverrides` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 抽离目标

第一轮实际抽离只允许把 3 个参数网格 helper 从 `src/runtime/backtest/experiment_sweep.rs` 移入父叶私有子模块。`start_backtest_experiment` 继续留在父级 `experiment_sweep`，只调用新的 `build_experiment_overrides`。

| helper | 当前职责 | 计划归属 | 保持不变 |
| --- | --- | --- | --- |
| `normalize_experiment_float_axis` | fee/slippage 轴去重、空轴回退、负数拒绝 | `parameter_grid` 私有子模块 | error code、field name、base fallback、去重顺序 |
| `normalize_experiment_latency_axis` | latency 轴去重、空轴回退 | `parameter_grid` 私有子模块 | base fallback、去重顺序、`u64` 类型 |
| `build_experiment_overrides` | 参数网格展开、variant 上限、base assumptions | `parameter_grid` 私有子模块，`pub(super)` 给父级 handler 调用 | `MAX_EXPERIMENT_VARIANTS`、empty grid、负数校验、展开顺序、`Some` 输出 |

---

## 实施方案

1. 新建计划目标文件 `src/runtime/backtest/experiment_sweep/parameter_grid.rs`。
2. 在 `src/runtime/backtest/experiment_sweep.rs` 顶部增加私有子模块声明:

```rust
mod parameter_grid;

use parameter_grid::build_experiment_overrides;
```

3. 将 `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides` 移入 parameter_grid 子模块。
4. 在子模块中用 `use super::*;` 复用父级已导入的 `StatusCode`、`RuntimeProtocolCoreConfig`、`FrontendExperimentRequest`、`FrontendExecutionAssumptionOverrides`、`MAX_EXPERIMENT_VARIANTS`、`json_bad_request`、`resolved_backtest_execution_assumptions`。
5. `normalize_experiment_float_axis` 与 `normalize_experiment_latency_axis` 保持私有 `fn`。
6. `build_experiment_overrides` 改为 `pub(super) fn`，只允许父级 `runtime.backtest.experiment_sweep` 调用。
7. 不改 `start_backtest_experiment` 的调用位置、variant suffix、preview persistence、capability guard、runtime config guard 或 execution assumption guard。
8. 不新增 public API，不新增 crate root re-export，不改 `src/runtime/mod.rs`。
9. 不移动 `MAX_EXPERIMENT_VARIANTS`。若实际代码可见性要求必须调整常量位置，应中止并回到方案讨论。
10. 完成代码移动后补 BE-001W-03 实际抽离记录，再判断是否需要 focused tests。

---

## 测试策略

本轮方案不新增测试代码。BE-001W-03 实际抽离时先跑既有代表测试；只有发现当前覆盖不足以冻结 empty grid、负数校验、dedupe 或展开顺序时，才在抽离记录中提出 focused test 补强。

| 风险点 | 既有保护 | 额外要求 |
| --- | --- | --- |
| variant count 和基本展开 | `api_experiments` 的 2x1x2 sweep | 必须继续通过 |
| preview/save/discard lifecycle | `api_experiments` | 不因 helper 移动漂移 |
| backtest execution assumption downstream | `api_backtest` | 确认 `execute_backtest_request` 复用桥未漂移 |
| evidence/report sibling | `api_evidence_contract` | 确认未混入 evidence owner |
| helper 可见性 | `cargo check -p quantpilot`、`cargo test --no-run` | 不新增 public export |

---

## 明确排除

| 排除项 | 原因 |
| --- | --- |
| `start_backtest_experiment` | handler 编排留在父级 `experiment_sweep`，本批只抽参数网格 helper |
| experiment route registration | route owner 仍是 `src/backend/runtime/routes.rs` |
| `execute_backtest_request` | 属于 `runtime.backtest.execution_start` 复用桥，不归 parameter_grid |
| `MAX_EXPERIMENT_VARIANTS` | 本批不移动常量，避免影响父级/测试可见性 |
| `FrontendExecutionAssumptionSweepGrid` / `FrontendExecutionAssumptionOverrides` | schema owner 仍是 `src/frontend_api_types.rs` |
| persistence owner | experiment/backtest persistence 继续归 `src/runtime_persistence.rs` |
| response mapping owner | list/detail/axes mapping 继续归 `src/runtime_response_mapping.rs` |
| AppState / lock owner | state、store dir 和锁 owner 不迁移 |
| audit owner | `persist_graph_audit_entry` 与 `build_graph_audit_entry` 不迁移 |
| `runtime.backtest.record_store` / `runtime.backtest.replay` / `backtest_compare` | sibling 已有边界，不能横向直连 |
| 发布过渡 | 不主动提出横向连接或性能旁路。ASCII guard: `release transition guard` |

---

## 适配性风险与处理

| 风险 | 处理 |
| --- | --- |
| 子模块路径与父文件同名目录并存 | 使用 `mod parameter_grid;`，让 Rust 按 `experiment_sweep/parameter_grid.rs` 解析；不改父级 module 名 |
| helper 访问父级类型失败 | 子模块先用 `use super::*;`，若仍失败只补显式 import，不扩大 public API |
| 常量可见性失败 | 保持 `MAX_EXPERIMENT_VARIANTS` 在父级可见；若需要迁移常量，暂停讨论 |
| `json_bad_request` 可见性失败 | 保持通过 `super::*` 访问；不复制错误构造逻辑 |
| 展开顺序漂移 | 移动时不改循环结构，`fee -> slippage -> latency` 三层循环原样迁移 |
| 去重语义漂移 | 保留 `Vec::contains`，不引入排序、HashSet 或浮点近似比较 |
| empty grid 语义漂移 | 保留 provided value count 判断，不生成 base-only variant |
| tests 覆盖不足 | 先跑既有门禁；若实际抽离暴露缺口，BE-001W-03 中单独登记 focused test |

---

## 中止条件

进入代码移动时，只要出现以下任一情况，应中止并回到方案讨论:

1. 需要改变 `FrontendExecutionAssumptionSweepGrid` 或 `FrontendExecutionAssumptionOverrides` schema。
2. 需要改变 empty grid、负数校验、base fallback、dedupe、variant count 或 expansion order。
3. 需要移动 `MAX_EXPERIMENT_VARIANTS`。
4. 需要把 `build_experiment_overrides` 暴露到父级以外。
5. 需要迁移 route、handler orchestration、execution_start、persistence、response mapping、state、audit、frontend caller 或发布过渡连接。
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

下一批应进入 BE-001W-03 `runtime.backtest.experiment_sweep.parameter_grid` 实际抽离记录: 按本方案只移动 3 个 helper 到 parameter_grid 私有子模块，保持父级 handler、route aggregate、execution_start 复用桥、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend caller 和发布过渡边界不变。完成后再做单叶 closeout，判断 parameter_grid 是否设置 `stop_split: true`。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep.parameter_grid` 已有抽离方案时，必须说明本批 `no code movement`，只允许下一批迁移 3 个 helper 到父级私有子模块。不得宣称 helper 已迁移、parameter_grid 已 closeout、`stop_split: true` 已设置、schema、route、handler orchestration、execution_start、persistence、response mapping、state、audit、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `112-runtime.backtest.experiment_sweep.parameter_grid抽离方案.md` 进入 v4.16 里程碑索引。
2. 模块树 `runtime.backtest.experiment_sweep.parameter_grid` 节点标记抽离方案已建立，但代码尚未移动。
3. 全量树能定位本方案、真实文件和下一步 BE-001W-03。
4. 治理门禁能发现本方案、`no code movement`、计划目标文件、3 个 helper、禁止迁移边界和回归证据。
5. 后续 BE-001W-03 实际抽离必须引用本方案，不得把 route、handler orchestration、execution_start、persistence、mapping、schema、state、audit 或 frontend caller 混入第一轮迁移。
