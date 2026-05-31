# v4.16.0 runtime.root_support_import_pilot 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001CX-04
> 基准: `311-runtime.root_support_import_pilot抽离记录.md`
> 目标子叶: `runtime.root_support_import_pilot`
> 判定: `runtime.root_support_import_pilot stop_split: true`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.root_support_import_pilot`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CX-04 `runtime.root_support_import_pilot` 单叶 closeout | closeout |
| 规范矩阵 | explicit import pass、minimum batch、stop_split、release transition guard | 阶段判定 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.root_support_import_pilot` | 子叶收口 |
| 模块树 | `runtime.root_support_import_pilot` | `stop_split: true` |

---

## closeout 判定

`runtime.root_support_import_pilot` 当前不继续拆成 `query_support_import_pass` 与 `response_support_import_pass` 微叶，设置:

```text
runtime.root_support_import_pilot stop_split: true
```

理由:

1. 本叶目标只是验证 staged explicit import pass 是否能安全收敛 root support child 的 parent wildcard import。
2. `src/runtime/query_support.rs` 与 `src/runtime/response_support.rs` 已完成显式 import，且没有新增 sibling horizontal link。
3. 两个文件均为已 closeout 的 support child，本轮没有改变 handler owner、route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、AppState 或 lock order。
4. 继续把本 pilot 拆成两个微叶只会增加治理噪声，不会带来新的模块边界收益。

---

## 当前事实

- `src/runtime/query_support.rs` 不再使用 `use super::*`。
- `src/runtime/response_support.rs` 不再使用 `use super::*`。
- runtime parent bridge 依赖文件数从 46 降为 44。
- `src/runtime/mod.rs` 仍保留 `use super::*`，parent import bridge 尚未消除。
- `event_stream`、`evidence_health`、`report_ops`、`run`、`backtest`、`mutation` 仍未进入本轮 import pass。

---

## 后续方向

下一步进入新的单子叶等价基线:

```text
BE-001CY-01 runtime.root_entry_import_pass 单子叶等价基线
```

该基线应优先评估 runtime 直属入口文件，而不是直接进入 mutation/backtest 大子树:

```text
src/runtime/event_stream.rs
src/runtime/evidence_health.rs
src/runtime/report_ops.rs
src/runtime/run_guard.rs
src/runtime/mod.rs
```

是否把 `report_ops.rs` 从 root entry pass 中拆出，需要在 BE-001CY-01 基线中根据真实依赖判定。不得在 closeout 中直接改写这些文件。

---

## 排除项

- 不修改 Rust 代码。
- 不继续拆 `query_support` 或 `response_support`。
- 不处理 run/backtest/mutation/report_ops 子树。
- 不删除 `src/runtime/mod.rs` 的 `use super::*`。
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

AI 声称 BE-001CX-04 完成时，必须说明:

1. 本批次是 `no code movement` closeout。
2. `runtime.root_support_import_pilot stop_split: true`。
3. parent import bridge 尚未消除，剩余依赖文件数为 44。
4. 下一步只能进入 BE-001CY-01 `runtime.root_entry_import_pass` 单子叶等价基线。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成、parent import bridge 已完全清除或 release transition 已启动。

---

## 验收标准

1. `312-runtime.root_support_import_pilot单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. closeout 明确 `runtime.root_support_import_pilot stop_split: true`。
3. 下一步固定为 BE-001CY-01 `runtime.root_entry_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
