# v4.16.0 runtime.backtest_import_pass 第三轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DJ-01
> 基准: `342-runtime.backtest.experiment_sweep_import_pass单叶closeout.md`
> 父叶: `runtime.backtest_import_pass`
> 判定: `runtime.backtest_import_pass stop_split: false`
> 当前剩余: root 1 / run 0 / backtest 5 / mutation 21 / test-only 1 / total 28
> 下一候选: `runtime.backtest.execution_start_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DK-01 `runtime.backtest.execution_start_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DJ-01 `runtime.backtest_import_pass` 第三轮父叶残余判断 | 父叶残余判断 |
| 规范矩阵 | staged explicit import pass、parent import bridge、stop_split、release transition guard | 候选选择 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass` | backtest import 父叶回流 |
| 模块树 | `runtime.backtest_import_pass` | `stop_split: false` |

---

## 当前分布

| 区域 | 剩余文件数 | 说明 |
| --- | ---: | --- |
| `runtime.root` | 1 | `src/runtime/mod.rs` parent bridge 仍存在 |
| `runtime.run` | 0 | `runtime.run_import_pass stop_split: true` 后已清零 |
| `runtime.backtest` | 5 | record_store / replay / experiment_sweep import pocket 已 closeout，剩余 execution_start 组 |
| `runtime.mutation` | 21 | mutation import bridge 尚未进入本轮 |
| test-only | 1 | `src/runtime/run_guard.rs` test-only super import 尚未处理 |
| total | 28 | parent import bridge 尚未消除 |

---

## `runtime.backtest` 剩余候选文件

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
```

已关闭的 import pocket:

```text
runtime.backtest.record_store_import_pass stop_split: true
runtime.backtest.replay_import_pass stop_split: true
runtime.backtest.experiment_sweep_import_pass stop_split: true
```

---

## 父叶判断

`runtime.backtest_import_pass` 不能关闭，维持:

```text
runtime.backtest_import_pass stop_split: false
```

理由:

1. `runtime.backtest` 仍有 5 个 parent bridge 依赖文件。
2. 剩余文件全部属于 execution_start 组，且仍包含 `use super::*` 或 `super::`。
3. execution_start 组已有物理抽离历史文档，但当前阶段不是重复抽离业务 owner，而是收敛 parent wildcard import。
4. 当前仍存在清晰小批次候选，不能宣称 `runtime.backtest_import_pass` 已完成。
5. release transition 未启动，不能以性能优化名义新增 sibling horizontal link。

---

## 下一候选选择

下一步选择:

```text
BE-001DK-01 runtime.backtest.execution_start_import_pass 单子叶等价基线
```

选择原因:

1. `record_store_import_pass`、`replay_import_pass`、`experiment_sweep_import_pass` 均已 closeout。
2. execution_start 组是 backtest 下最后一组 parent bridge 依赖。
3. 该候选应先冻结五文件 pocket、public/internal 方法、父级输入面和等价风险，再判断是整组 import rewrite 还是继续拆 v4 / legacy pocket。
4. 下一步仍必须先建等价基线，不能直接改写 Rust import。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/backtest/execution_start.rs` 或其子文件 import。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route facade、handler owner、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。
- 本批不恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

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

AI 声称 BE-001DJ-01 完成时，必须说明:

1. 本批是 `no code movement` 父叶残余判断。
2. `runtime.backtest_import_pass stop_split: false`。
3. 当前剩余分布为 root 1 / run 0 / backtest 5 / mutation 21 / test-only 1 / total 28。
4. `runtime.backtest.record_store_import_pass`、`runtime.backtest.replay_import_pass` 与 `runtime.backtest.experiment_sweep_import_pass` 均已 `stop_split: true`。
5. 下一步只能进入 BE-001DK-01 `runtime.backtest.execution_start_import_pass` 单子叶等价基线。
6. release transition 未启动，未新增 sibling horizontal link。
7. 旧的三叶暂停目标仍为取消状态。

不得宣称 `runtime.backtest_import_pass` 已完成、`backend.runtime` 已完成、parent import bridge 已清除或 execution_start import 已经改写。

---

## 验收标准

1. `343-runtime.backtest_import_pass第三轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶明确保持 `runtime.backtest_import_pass stop_split: false`。
3. 下一候选固定为 BE-001DK-01 `runtime.backtest.execution_start_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
