# v4.16.0 runtime.root_entry_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CY-01
> 基准: `312-runtime.root_support_import_pilot单叶closeout.md`
> 目标子叶: `runtime.root_entry_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.root_entry_import_pass`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CY-01 `runtime.root_entry_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | parent import bridge、explicit import pass、root entry、test-only super import、release transition guard | 等价边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.root_entry_import_pass` | root entry 白箱依赖基线 |
| 模块树 | `runtime.root_entry_import_pass` | 新子叶登记 |

---

## 当前事实

BE-001CX-03 后，`src/runtime/**.rs` 中存在 `use super::*` 或 `super::` 依赖的文件数为 44。按层级拆分如下:

| 层级 | 文件数 | 说明 |
| --- | ---: | --- |
| `runtime.root` | 1 | `src/runtime/mod.rs` 自身仍通过 `use super::*` 获取上层 backend surface |
| `runtime.root_child` | 4 | `event_stream`、`evidence_health`、`report_ops`、`run_guard` |
| `runtime.run` | 4 | run 子树 |
| `runtime.backtest` | 11 | backtest 子树 |
| `runtime.report_ops` | 3 | report_ops 子树 |
| `runtime.mutation` | 21 | mutation 子树，依赖最密集 |

---

## 候选文件

本基线冻结 root entry import pass 的候选范围:

```text
src/runtime/event_stream.rs
src/runtime/evidence_health.rs
src/runtime/report_ops.rs
src/runtime/run_guard.rs
src/runtime/mod.rs
```

### 真实依赖判定

- `src/runtime/event_stream.rs`: route-facing SSE handler，当前通过 `use super::*` 取得 `auth::UserId`、`State`、`Path`、`AppState`、`Sse`、`Event`、`KeepAlive`、`Infallible`、`stream!`、`Duration`、`sleep`、`StatusCode`、`load_run_record_from_state`、`json_sse_event`、`SSE_EVENT_DELAY_MS`。
- `src/runtime/evidence_health.rs`: evidence health handlers，当前通过 `use super::*` 取得 `State`、`Json`、`AppState`、`StatusCode`、runtime evidence schema、runtime report store helpers、cleanup helpers、`current_time_ms`、`io_error`。
- `src/runtime/report_ops.rs`: report ops parent facade，当前通过 `use super::*` 向 `merge_generation_health`、`runtime_report`、`v1_report_endpoints` 子模块提供父级白箱输入；它自身还有 3 个 child 依赖文件。
- `src/runtime/run_guard.rs`: 当前 `use super::*` 只存在于 `#[cfg(test)] mod tests` 内部，用于测试访问本文件私有 item，不属于 parent import bridge 业务残余。
- `src/runtime/mod.rs`: 当前仍通过 `use super::*` 获取上层 backend surface，是 parent import bridge 根部残余，不能在 root entry pilot 中直接删除。

---

## 边界判定

本基线只冻结事实，不进行代码移动或 import 改写。BE-001CY-02 需要做出拆分方案:

1. 是否先处理 `event_stream` 与 `evidence_health` 两个 route-facing root child。
2. 是否将 `report_ops.rs` 单独拆成 `runtime.report_ops_import_pass`，避免把它与 root entry route handlers 混成一批。
3. 是否排除 `run_guard.rs`，因为其 `use super::*` 为 test-only super import。
4. 何时处理 `src/runtime/mod.rs` 的上层 `use super::*`，以及是否必须等 root child / subtrees 全部收敛后再处理。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不删除 `src/runtime/mod.rs` 的 `use super::*`。
- 本批不改写 `event_stream`、`evidence_health`、`report_ops`、`run_guard`。
- 本批不处理 run/backtest/mutation 子树。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 等价基线，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CY-02 runtime.root_entry_import_pass 抽离方案
```

BE-001CY-02 只能设计 root entry import pass 的最小批次、允许修改文件、排除项、回退点和验证门禁；不得直接改写 Rust import。

---

## 幻觉检查点

AI 声称 BE-001CY-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. parent import bridge 尚未消除，剩余依赖文件数为 44。
3. `run_guard.rs` 的 `use super::*` 是 test-only super import，不是业务父桥残余。
4. 下一步只能进入 BE-001CY-02 抽离方案。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成、parent import bridge 已完全清除或 release transition 已启动。

---

## 验收标准

1. `313-runtime.root_entry_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线冻结 root entry 候选文件和 test-only super import 判定。
3. 下一步固定为 BE-001CY-02 `runtime.root_entry_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
