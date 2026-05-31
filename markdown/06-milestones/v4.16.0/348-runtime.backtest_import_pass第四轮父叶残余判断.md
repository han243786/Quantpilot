# v4.16.0 runtime.backtest_import_pass 第四轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DL-01
> 基准: `347-runtime.backtest.execution_start_import_pass单叶closeout.md`
> 父叶: `runtime.backtest_import_pass`
> 判定: `runtime.backtest_import_pass stop_split: true`
> 当前剩余: root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23
> 下一候选: `runtime.parent_import_bridge`
> 代码动作: no code movement
> 下一步: BE-001DM-01 `runtime.parent_import_bridge` 父叶残余判断

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DL-01 `runtime.backtest_import_pass` 第四轮父叶残余判断 | 父叶残余判断 |
| 规范矩阵 | staged explicit import pass、parent import bridge、stop_split、release transition guard | 父叶关闭 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass` | backtest import 父叶收口 |
| 模块树 | `runtime.backtest_import_pass` | `stop_split: true` |

---

## 当前分布

| 区域 | 剩余文件数 | 说明 |
| --- | ---: | --- |
| `runtime.root` | 1 | `src/runtime/mod.rs` parent bridge 仍存在 |
| `runtime.run` | 0 | `runtime.run_import_pass stop_split: true` 后已清零 |
| `runtime.backtest` | 0 | backtest import pocket 全部 closeout |
| `runtime.mutation` | 21 | mutation import bridge 尚未进入本轮 |
| test-only | 1 | `src/runtime/run_guard.rs` test-only super import 尚未处理 |
| total | 23 | parent import bridge 尚未消除 |

---

## `runtime.backtest` 已关闭 pocket

```text
runtime.backtest.record_store_import_pass stop_split: true
runtime.backtest.replay_import_pass stop_split: true
runtime.backtest.experiment_sweep_import_pass stop_split: true
runtime.backtest.execution_start_import_pass stop_split: true
```

`runtime.backtest` 下已经不存在 `use super::*` 或 `super::` parent bridge residual。本轮不再拆 record store、replay、experiment sweep、legacy dispatch、v4 projection、request resolution 或 runtime execution 微叶。

---

## 父叶判断

`runtime.backtest_import_pass` 可以关闭，设置:

```text
runtime.backtest_import_pass stop_split: true
```

理由:

1. `runtime.backtest` residual 为 backtest 0。
2. record_store、replay、experiment_sweep、execution_start 四个 staged import pocket 均已完成 closeout。
3. backtest route、handler、state write、artifact views、record assembly、audit log、v4 projection 和 legacy execution 语义没有迁移 owner。
4. 剩余 parent bridge 已不属于 backtest 父叶，而属于上层 `runtime.parent_import_bridge`。
5. release transition 未启动，不能以性能优化名义新增 sibling horizontal link。

---

## 下一步回流

下一步回到上层父叶:

```text
BE-001DM-01 runtime.parent_import_bridge 父叶残余判断
```

该判断只负责重新盘点剩余 23 个 parent bridge 依赖，并决定下一候选是 root bridge、mutation import bridge 还是 test-only `run_guard`。不能在本父叶 closeout 中直接改写 Rust import。

旧的“完成三个叶子节点后暂停”指令不再作为当前递归目标；后续继续保持父叶判断、子叶基线、抽离方案、实际抽离、单叶 closeout 的干净递归流程。

```text
old_three_leaf_pause_target_cancelled
```

---

## 剩余 parent bridge

```text
src/runtime/mod.rs
src/runtime/mutation/**
src/runtime/run_guard.rs
```

当前分布:

```text
root 1
run 0
backtest 0
mutation 21
test-only 1
total 23
```

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route facade、handler owner、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。
- 本批不宣称 `backend.runtime` 或完整 parent import bridge 已完成。

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

AI 声称 BE-001DL-01 完成时，必须说明:

1. 本批是 `no code movement` 父叶残余判断。
2. `runtime.backtest_import_pass stop_split: true`。
3. 当前剩余分布为 root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23。
4. backtest 四个 import pocket 均已 `stop_split: true`。
5. 下一步只能进入 BE-001DM-01 `runtime.parent_import_bridge` 父叶残余判断。
6. release transition 未启动，未新增 sibling horizontal link。
7. 旧的三叶暂停目标仍为取消状态。

不得宣称 `backend.runtime` 已完成、parent import bridge 已清除、mutation import 已完成或 root bridge 已处理。

---

## 验收标准

1. `348-runtime.backtest_import_pass第四轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶明确设置 `runtime.backtest_import_pass stop_split: true`。
3. 下一步固定为 BE-001DM-01 `runtime.parent_import_bridge` 父叶残余判断。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
