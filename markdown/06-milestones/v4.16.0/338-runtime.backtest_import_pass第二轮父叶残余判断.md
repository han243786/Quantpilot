# v4.16.0 runtime.backtest_import_pass 第二轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DH-01
> 基准: `337-runtime.backtest.replay_import_pass单叶closeout.md`
> 父叶: `runtime.backtest_import_pass`
> 判定: `runtime.backtest_import_pass stop_split: false`
> 当前剩余: root 1 / run 0 / backtest 9 / mutation 21 / test-only 1 / total 32
> 下一候选: `runtime.backtest.experiment_sweep_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DI-01 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DH-01 `runtime.backtest_import_pass` 第二轮父叶残余判断 | 父叶残余判断 |
| 规范矩阵 | staged explicit import pass、parent import bridge、stop_split、release transition guard | 候选选择 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass` | backtest import 父叶回流 |
| 模块树 | `runtime.backtest_import_pass` | `stop_split: false` |

---

## 当前分布

| 区域 | 剩余文件数 | 说明 |
| --- | ---: | --- |
| `runtime.root` | 1 | `src/runtime/mod.rs` parent bridge 仍存在 |
| `runtime.run` | 0 | `runtime.run_import_pass stop_split: true` 后已清零 |
| `runtime.backtest` | 9 | record_store / replay import pocket 已 closeout，剩余 experiment / execution 两组 |
| `runtime.mutation` | 21 | mutation import bridge 尚未进入本轮 |
| test-only | 1 | `src/runtime/run_guard.rs` test-only super import 尚未处理 |
| total | 32 | parent import bridge 尚未消除 |

---

## `runtime.backtest` 剩余候选文件

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/experiment_sweep.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/parameter_grid.rs
src/runtime/backtest/record_lifecycle.rs
src/runtime/backtest/start_orchestration.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
```

已关闭的 import pocket:

```text
runtime.backtest.record_store_import_pass stop_split: true
runtime.backtest.replay_import_pass stop_split: true
```

---

## 父叶判断

`runtime.backtest_import_pass` 不能关闭，维持:

```text
runtime.backtest_import_pass stop_split: false
```

理由:

1. `runtime.backtest` 仍有 9 个 parent bridge 依赖文件。
2. `experiment_sweep.rs`、`parameter_grid.rs`、`record_lifecycle.rs` 与 `start_orchestration.rs` 形成相对独立的 experiment sweep import pocket。
3. `execution_start.rs` 与 `legacy_dispatch.rs`、`v4_projection.rs`、`v4_request_resolution.rs`、`v4_runtime_execution.rs` 是更复杂的 execution pocket，风险高于 experiment sweep。
4. 当前仍存在清晰小批次候选，不能宣称 `runtime.backtest_import_pass` 已完成。
5. release transition 未启动，不能以性能优化名义新增 sibling horizontal link。

---

## 下一候选选择

下一步选择:

```text
BE-001DI-01 runtime.backtest.experiment_sweep_import_pass 单子叶等价基线
```

选择原因:

1. `experiment_sweep` 是 planned order 中 replay 之后的下一口袋。
2. 它比 `execution_start` 的 v4 / legacy 多层链更小，适合继续降低 backtest parent bridge 残余。
3. 等价证据可由 `api_experiments` 覆盖 experiment list/detail/save/discard 与 sweep creation，并由 `api_backtest` 覆盖复用的 backtest execution 边界。
4. 下一步仍必须先建等价基线，不能直接改写 Rust import。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/backtest/experiment_sweep.rs` import。
- 本批不处理 `src/runtime/backtest/execution_start.rs` 或其 v4 / legacy 子文件。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route facade、handler owner、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 父叶残余判断，提交前至少执行:

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

AI 声称 BE-001DH-01 完成时，必须说明:

1. 本批是 `no code movement` 父叶残余判断。
2. `runtime.backtest_import_pass stop_split: false`。
3. 当前剩余分布为 root 1 / run 0 / backtest 9 / mutation 21 / test-only 1 / total 32。
4. `runtime.backtest.record_store_import_pass` 与 `runtime.backtest.replay_import_pass` 均已 `stop_split: true`。
5. 下一步只能进入 BE-001DI-01 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线。
6. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest_import_pass` 已完成、`backend.runtime` 已完成、parent import bridge 已清除或 experiment sweep import 已经改写。

---

## 验收标准

1. `338-runtime.backtest_import_pass第二轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶明确保持 `runtime.backtest_import_pass stop_split: false`。
3. 下一候选固定为 BE-001DI-01 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
