# v4.16.0 runtime.run_import_pass 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DB-04
> 基准: `324-runtime.run_import_pass抽离记录.md`
> 目标子叶: `runtime.run_import_pass`
> 判定: `runtime.run_import_pass stop_split: true`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.run_import_pass`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DB-04 `runtime.run_import_pass` 单叶 closeout | closeout |
| 规范矩阵 | explicit import pass、run child import、stop_split、release transition guard | 阶段判定 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.run_import_pass` | 子叶收口 |
| 模块树 | `runtime.run_import_pass` | `stop_split: true` |

---

## closeout 判定

`runtime.run_import_pass` 当前不继续拆成 `runtime.run.v4_handoff_import_pass`、`runtime.run.session_start_import_pass`、`runtime.run.record_store_import_pass` 或 `runtime.run.replay_status_import_pass` 微叶，设置:

```text
runtime.run_import_pass stop_split: true
```

理由:

1. 本叶目标是收敛 4 个 run child 的 parent wildcard import，不是重新拆分 run handler 功能。
2. `src/runtime/run/v4_handoff.rs`、`src/runtime/run/session_start.rs`、`src/runtime/run/record_store.rs` 与 `src/runtime/run/replay_status.rs` 已完成显式 import。
3. run route path、handler name、response schema、status code、error code、run mutex 和 evidence metrics 均未改变。
4. 4 个 run child 已在前序递归中完成功能抽离；继续拆微叶只会增加治理噪声，不会带来新的边界收益。
5. 未新增 sibling horizontal link，未启动 release transition。

---

## 当前事实

- 4 个 run child 不再包含 `use super::*` 或 `super::`。
- runtime parent bridge 依赖文件数从 38 降为 34。
- `src/runtime/mod.rs` root parent bridge 仍未处理。
- `src/runtime/backtest/**`、`src/runtime/mutation/**` 与 test-only `src/runtime/run_guard.rs` 仍未处理。
- parent import bridge 尚未消除，`backend.runtime` 尚未完成。

---

## 后续方向

下一步回到父叶残余判断:

```text
BE-001DC-01 runtime.parent_import_bridge 父叶残余判断
```

该判断只负责重新盘点剩余 34 个 parent bridge 依赖，并按递归流程选择下一批 staged explicit import pass 候选。不得在 BE-001DB-04 closeout 中直接处理 `src/runtime/mod.rs`、backtest/mutation 子树或 release transition。

---

## 排除项

- 不修改 Rust 代码。
- 不继续拆 run import pass 微叶。
- 不处理 `src/runtime/mod.rs`。
- 不处理 `src/runtime/backtest/**`、`src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
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

AI 声称 BE-001DB-04 完成时，必须说明:

1. 本批次是 `no code movement` closeout。
2. `runtime.run_import_pass stop_split: true`。
3. parent import bridge 尚未消除，剩余依赖文件数为 34。
4. 下一步只能进入 BE-001DC-01 `runtime.parent_import_bridge` 父叶残余判断。
5. `src/runtime/mod.rs`、backtest/mutation 子树和 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成或 parent import bridge 已完全清除。

---

## 验收标准

1. `325-runtime.run_import_pass单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. closeout 明确 `runtime.run_import_pass stop_split: true`。
3. 下一步固定为 BE-001DC-01 `runtime.parent_import_bridge` 父叶残余判断。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
