# v4.16.0 runtime.root_entry_import_pass 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001CY-04
> 基准: `315-runtime.root_entry_import_pass抽离记录.md`
> 目标子叶: `runtime.root_entry_import_pass`
> 判定: `runtime.root_entry_import_pass stop_split: true`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.root_entry_import_pass`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CY-04 `runtime.root_entry_import_pass` 单叶 closeout | closeout |
| 规范矩阵 | explicit import pass、minimum batch、stop_split、release transition guard | 阶段判定 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.root_entry_import_pass` | 子叶收口 |
| 模块树 | `runtime.root_entry_import_pass` | `stop_split: true` |

---

## closeout 判定

`runtime.root_entry_import_pass` 当前不继续拆成 `runtime.event_stream_import_pass` 与 `runtime.evidence_health_import_pass` 微叶，设置:

```text
runtime.root_entry_import_pass stop_split: true
```

理由:

1. 本叶目标是验证 two-handler root entry pilot 是否能安全收敛 route-facing root child 的 parent wildcard import。
2. `src/runtime/event_stream.rs` 与 `src/runtime/evidence_health.rs` 已完成显式 import，且未新增 sibling horizontal link。
3. 两个文件的 route path、SSE frame order、evidence health / cleanup schema、runtime report store helpers、cleanup helpers、`AppState` owner、lock order 与 frontend caller 均未改变。
4. 继续把本叶拆成两个微叶只会增加治理噪声，不会带来新的模块边界收益。

---

## 当前事实

- `src/runtime/event_stream.rs` 不再使用 `use super::*`。
- `src/runtime/evidence_health.rs` 不再使用 `use super::*`。
- runtime parent bridge 依赖文件数从 44 降为 42。
- `src/runtime/report_ops.rs` 仍是 root child parent facade，且其 child 仍有 3 个 `super` 依赖文件。
- `src/runtime/run_guard.rs` 的 `use super::*` 仍仅属于 test-only super import。
- `src/runtime/mod.rs` 仍保留 `use super::*`，parent import bridge 尚未消除。

---

## 后续方向

下一步进入新的单子叶等价基线:

```text
BE-001CZ-01 runtime.report_ops_import_pass 单子叶等价基线
```

该基线应优先冻结 `src/runtime/report_ops.rs` 及其 report_ops child 的真实 import 依赖:

```text
src/runtime/report_ops.rs
src/runtime/report_ops/runtime_report.rs
src/runtime/report_ops/v1_report_endpoints.rs
src/runtime/report_ops/merge_generation_health.rs
```

不得跳过 `runtime.report_ops_import_pass` 直接处理 `src/runtime/mod.rs`，也不得启动 release transition。

---

## 排除项

- 不修改 Rust 代码。
- 不继续拆 `event_stream` 或 `evidence_health` 微叶。
- 不处理 run/backtest/mutation 子树。
- 不处理 `src/runtime/report_ops.rs`、`src/runtime/run_guard.rs` 或 `src/runtime/mod.rs`。
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

AI 声称 BE-001CY-04 完成时，必须说明:

1. 本批次是 `no code movement` closeout。
2. `runtime.root_entry_import_pass stop_split: true`。
3. parent import bridge 尚未消除，剩余依赖文件数为 42。
4. `src/runtime/report_ops.rs` 尚未处理，下一步只能进入 BE-001CZ-01 `runtime.report_ops_import_pass` 单子叶等价基线。
5. `src/runtime/mod.rs` 父桥未处理，release transition 未启动。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成或 parent import bridge 已完全清除。

---

## 验收标准

1. `316-runtime.root_entry_import_pass单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. closeout 明确 `runtime.root_entry_import_pass stop_split: true`。
3. 下一步固定为 BE-001CZ-01 `runtime.report_ops_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
