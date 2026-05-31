# v4.16.0 runtime.backtest.execution_start_import_pass 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DK-04
> 基准: `346-runtime.backtest.execution_start_import_pass抽离记录.md`
> 目标子叶: `runtime.backtest.execution_start_import_pass`
> 判定: `runtime.backtest.execution_start_import_pass stop_split: true`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.execution_start_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DL-01 `runtime.backtest_import_pass` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DK-04 `runtime.backtest.execution_start_import_pass` 单叶 closeout | closeout |
| 规范矩阵 | five-file explicit import rewrite、parent surface、execution_start pocket、release transition guard | 阶段判定 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.execution_start_import_pass` | execution_start import 子叶收口 |
| 模块树 | `runtime.backtest.execution_start_import_pass` | `stop_split: true` |

---

## closeout 判定

`runtime.backtest.execution_start_import_pass` 当前不继续拆成更小 import 微叶，设置:

```text
runtime.backtest.execution_start_import_pass stop_split: true
```

理由:

1. 本叶目标是清除 execution_start 组五文件 parent wildcard / super import，不迁移 backtest 业务 owner。
2. 目标文件已经全部显式输入，不再包含 `use super::*` 或 `super::`:

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
```

3. `v4_projection.rs` 的 test-scope `use super::*` 已同步收敛，避免测试作用域继续污染 parent bridge residual。
4. `start_backtest_run`、`execute_backtest_request`、`execute_v4_backtest_request`、`prepare_legacy_backtest_dispatch`、`run_legacy_backtest_dispatch`、`LegacyBacktestDispatchOutput`、`build_v4_backtest_output`、`frontend_events_from_v4_backtest_artifact`、`v4_equity_curve_from_artifact`、`is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type`、`run_v4_backtest_runtime_execution` 均保持父级白箱 handoff，未引入 sibling horizontal link。
5. 若继续拆 legacy dispatch、v4 projection、request resolution 或 runtime execution，会进入业务 owner 重构，不属于 parent import bridge 阶段。

---

## 当前事实

- 本批是 `no code movement` closeout。
- five-file explicit import rewrite 已完成。
- parent surface、execution_start pocket、release transition guard 已按 BE-001DK-03 验证。
- runtime parent bridge 依赖文件数已从 28 降为 23。
- `runtime.backtest` parent bridge residual 为 0。
- 当前剩余分布为 root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23。
- 剩余 parent bridge 依赖只集中在 `src/runtime/mod.rs`、`src/runtime/mutation/**` 和 `src/runtime/run_guard.rs`。
- `runtime.backtest_import_pass` 尚未 closeout，必须先回到父叶残余判断。
- `backend.runtime` 和完整 Rust parent import bridge 尚未完成。

```text
actual_parent_import_bridge_28_to_23
backtest 0
root 1
run 0
mutation 21
test-only 1
total 23
```

---

## 当前剩余

`runtime.backtest` 剩余 parent bridge import residual 已清零。剩余 parent bridge 文件为:

```text
src/runtime/mod.rs
src/runtime/mutation/**
src/runtime/run_guard.rs
```

---

## 后续方向

下一步回到 `runtime.backtest_import_pass` 父叶残余判断:

```text
BE-001DL-01 runtime.backtest_import_pass 父叶残余判断
```

该判断只负责确认 backtest residual 是否可以把 `runtime.backtest_import_pass` 设置为 `stop_split: true`，并把控制权交还 `runtime.parent_import_bridge`。不能在本 closeout 中直接进入 mutation、root bridge 或 release transition。

旧的“完成三个叶子节点后暂停”指令不再作为当前递归目标；后续只按父叶判断、子叶基线、抽离方案、实际抽离、单叶 closeout 的干净递归流程推进。

```text
old_three_leaf_pause_target_cancelled
```

---

## 排除项

- 不修改 Rust 代码。
- 不继续拆 execution_start import 微叶。
- 不处理 `src/runtime/mod.rs` root parent bridge。
- 不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 不新增 sibling horizontal link。
- 不启动 release transition。
- 不宣称 `runtime.backtest_import_pass`、`backend.runtime` 或完整 parent import bridge 已完成。

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

AI 声称 BE-001DK-04 完成时，必须说明:

1. 本批次是 `no code movement` closeout。
2. `runtime.backtest.execution_start_import_pass stop_split: true`。
3. 五文件 explicit import rewrite 已完成，目标文件不再包含 `use super::*` 或 `super::`。
4. runtime parent bridge 剩余 23 个依赖文件，其中 backtest residual 为 0。
5. 下一步只能进入 BE-001DL-01 `runtime.backtest_import_pass` 父叶残余判断。
6. root bridge、mutation 子树和 test-only `src/runtime/run_guard.rs` 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。
8. 旧的三叶暂停目标已取消，不再污染当前递归队列。

不得宣称 `runtime.backtest_import_pass` 已 closeout、`backend.runtime` 已完成、Rust 重构已完成或 parent import bridge 已完全清除。

---

## 验收标准

1. `347-runtime.backtest.execution_start_import_pass单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. closeout 明确 `runtime.backtest.execution_start_import_pass stop_split: true`。
3. 下一步固定为 BE-001DL-01 `runtime.backtest_import_pass` 父叶残余判断。
4. 旧的三叶暂停目标继续保持取消。
5. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
