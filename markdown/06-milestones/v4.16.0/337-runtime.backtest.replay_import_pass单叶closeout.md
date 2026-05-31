# v4.16.0 runtime.backtest.replay_import_pass 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DG-04
> 基准: `336-runtime.backtest.replay_import_pass抽离记录.md`
> 目标子叶: `runtime.backtest.replay_import_pass`
> 判定: `runtime.backtest.replay_import_pass stop_split: true`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.replay_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DH-01 `runtime.backtest_import_pass` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DG-04 `runtime.backtest.replay_import_pass` 单叶 closeout | closeout |
| 规范矩阵 | explicit import pass、replay import、stop_split、release transition guard | 阶段判定 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.replay_import_pass` | replay import 子叶收口 |
| 模块树 | `runtime.backtest.replay_import_pass` | `stop_split: true` |

---

## closeout 判定

`runtime.backtest.replay_import_pass` 当前不继续拆成更小 import 微叶，设置:

```text
runtime.backtest.replay_import_pass stop_split: true
```

理由:

1. 本叶目标只是收敛 `src/runtime/backtest/replay.rs` 的 parent wildcard import，不是迁移 replay 业务 owner。
2. `src/runtime/backtest/replay.rs` 已删除 `use super::*`，并改为显式输入。
3. `get_backtest_replay` 的 route path、handler name、status code、bad cursor error、response schema、record lookup、query normalization 和 replay page metric 均未改变。
4. 本文件只有 1 个 public route handler；继续拆 query normalization、record lookup、response mapping 或 metric owner 会进入业务 owner 重构，不属于 parent import bridge 阶段。
5. 本批未新增 sibling horizontal link，未启动 release transition。

---

## 当前事实

- `src/runtime/backtest/replay.rs` 不再包含 `use super::*` 或 `super::`。
- bad cursor 输出仍锚定 `bad_replay_cursor`。
- replay page metric 仍锚定 `record_replay_page`。
- runtime parent bridge 依赖文件数从 33 降为 32。
- 当前分布为 root 1 / run 0 / backtest 9 / mutation 21 / test-only 1 / total 32。
- `src/runtime/backtest/record_store.rs` 与 `src/runtime/backtest/replay.rs` 两个 direct route import pocket 均已 closeout。
- `src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest/execution_start.rs` 及其子文件仍属于 backtest 剩余队列。
- `src/runtime/mod.rs` root parent bridge、`src/runtime/mutation/**` 与 test-only `src/runtime/run_guard.rs` 仍未处理。
- parent import bridge 尚未消除，`backend.runtime` 尚未完成。

---

## 后续方向

下一步回到 `runtime.backtest_import_pass` 父叶残余判断:

```text
BE-001DH-01 runtime.backtest_import_pass 父叶残余判断
```

该判断只负责重新盘点剩余 32 个 parent bridge 依赖，并在 `runtime.backtest_import_pass` 内选择下一个 staged explicit import pass 候选。默认优先评估:

```text
runtime.backtest.experiment_sweep_import_pass
```

但必须由 BE-001DH-01 重新确认剩余分布、风险边界和候选顺序，不能在本 closeout 中直接进入 Rust import rewrite。

---

## 排除项

- 不修改 Rust 代码。
- 不继续拆 replay import 微叶。
- 不处理 `src/runtime/backtest/experiment_sweep.rs` 或其内部子文件。
- 不处理 `src/runtime/backtest/execution_start.rs` 或其内部子文件。
- 不处理 `src/runtime/mod.rs` root parent bridge。
- 不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 不迁移 record lookup、query schema、response mapping、metric、state、persistence 或 frontend caller owner。
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

AI 声称 BE-001DG-04 完成时，必须说明:

1. 本批次是 `no code movement` closeout。
2. `runtime.backtest.replay_import_pass stop_split: true`。
3. parent import bridge 尚未消除，剩余依赖文件数为 32。
4. 下一步只能进入 BE-001DH-01 `runtime.backtest_import_pass` 父叶残余判断。
5. experiment_sweep、execution_start、root bridge、mutation 子树与 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest_import_pass` 已完成、`backend.runtime` 已完成、Rust 重构已完成或 parent import bridge 已完全清除。

---

## 验收标准

1. `337-runtime.backtest.replay_import_pass单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. closeout 明确 `runtime.backtest.replay_import_pass stop_split: true`。
3. 下一步固定为 BE-001DH-01 `runtime.backtest_import_pass` 父叶残余判断。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
