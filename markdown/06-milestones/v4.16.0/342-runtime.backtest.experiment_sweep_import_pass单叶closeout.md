# v4.16.0 runtime.backtest.experiment_sweep_import_pass 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DI-04
> 基准: `341-runtime.backtest.experiment_sweep_import_pass抽离记录.md`
> 目标子叶: `runtime.backtest.experiment_sweep_import_pass`
> 判定: `runtime.backtest.experiment_sweep_import_pass stop_split: true`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.experiment_sweep_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DJ-01 `runtime.backtest_import_pass` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DI-04 `runtime.backtest.experiment_sweep_import_pass` 单叶 closeout | closeout |
| 规范矩阵 | four-file explicit import rewrite、parent whitebox handoff、stop_split、release transition guard | 阶段判定 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.experiment_sweep_import_pass` | experiment sweep import 子叶收口 |
| 模块树 | `runtime.backtest.experiment_sweep_import_pass` | `stop_split: true` |

---

## closeout 判定

`runtime.backtest.experiment_sweep_import_pass` 当前不继续拆成更小 import 微叶，设置:

```text
runtime.backtest.experiment_sweep_import_pass stop_split: true
```

理由:

1. 本叶目标是收敛 experiment sweep 四文件 pocket 的 parent wildcard / sibling super import，不迁移 experiment 业务 owner。
2. `src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/record_lifecycle.rs`、`src/runtime/backtest/start_orchestration.rs` 已不再包含 `use super::*` 或 `super::`。
3. `build_experiment_overrides` 仍由 `experiment_sweep.rs` 父级白箱输入面转交给 `start_orchestration.rs`，没有新增 sibling horizontal link。
4. `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 的 route path、handler name、status code、response schema、state cache、persistence owner、audit owner 和 transient cleanup 均未改变。
5. 若继续拆 grid validation、record lifecycle、variant orchestration 或 audit cleanup，会进入业务 owner 重构，不属于 parent import bridge 阶段。

---

## 当前事实

- 本批是 `no code movement` closeout。
- 四文件 explicit import rewrite 已完成。
- runtime parent bridge 依赖文件数从 32 降为 28。
- 当前剩余分布为 root 1 / run 0 / backtest 5 / mutation 21 / test-only 1 / total 28。
- `runtime.backtest.record_store_import_pass`、`runtime.backtest.replay_import_pass`、`runtime.backtest.experiment_sweep_import_pass` 三个 backtest import pocket 均已 closeout。
- `runtime.backtest_import_pass` 仍保持 `stop_split: false`，需要回到父叶残余判断。
- `src/runtime/mod.rs` root parent bridge、`src/runtime/mutation/**` 和 test-only `src/runtime/run_guard.rs` 仍未处理。
- parent import bridge 尚未清除，`backend.runtime` 尚未完成。

---

## 当前剩余

`runtime.backtest` 剩余 parent bridge 依赖已收敛到 execution_start 组:

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
```

---

## 后续方向

下一步回到 `runtime.backtest_import_pass` 父叶残余判断:

```text
BE-001DJ-01 runtime.backtest_import_pass 父叶残余判断
```

该判断只负责重新盘点剩余 28 个 parent bridge 依赖，并决定 `runtime.backtest_import_pass` 是否继续拆向 execution_start 组的 staged explicit import pass。不能在本 closeout 中直接进入 Rust import rewrite。

旧的“完成三个叶子节点后暂停”指令不再作为当前递归目标；后续只按父叶判断、子叶基线、抽离方案、实际抽离、单叶 closeout 的干净递归流程推进。

```text
old_three_leaf_pause_target_cancelled
```

---

## 排除项

- 不修改 Rust 代码。
- 不继续拆 experiment sweep import 微叶。
- 不处理 `src/runtime/backtest/execution_start.rs`、`legacy_dispatch.rs` 或 `v4_*` 文件。
- 不处理 `src/runtime/mod.rs` root parent bridge。
- 不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 不新增 sibling horizontal link。
- 不启动 release transition。

---

## 验证要求

本批为 `no code movement` closeout，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001DI-04 完成时，必须说明:

1. 本批次是 `no code movement` closeout。
2. `runtime.backtest.experiment_sweep_import_pass stop_split: true`。
3. parent import bridge 尚未清除，剩余依赖文件数为 28。
4. 下一步只能进入 BE-001DJ-01 `runtime.backtest_import_pass` 父叶残余判断。
5. execution_start 组、root bridge、mutation 子树和 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。
7. “完成三个叶子节点后暂停”不再是当前递归目标。

不得宣称 `runtime.backtest_import_pass` 已完成、`backend.runtime` 已完成、Rust 重构已完成或 parent import bridge 已完全清除。

---

## 验收标准

1. `342-runtime.backtest.experiment_sweep_import_pass单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. closeout 明确 `runtime.backtest.experiment_sweep_import_pass stop_split: true`。
3. 下一步固定为 BE-001DJ-01 `runtime.backtest_import_pass` 父叶残余判断。
4. 旧的三叶暂停目标不再污染当前递归队列。
5. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
