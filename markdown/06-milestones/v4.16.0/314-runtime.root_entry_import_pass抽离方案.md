# v4.16.0 runtime.root_entry_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CY-02
> 基准: `313-runtime.root_entry_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.root_entry_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.root_entry_import_pass`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CY-02 `runtime.root_entry_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | parent import bridge、explicit import pass、root entry、minimum batch、release transition guard | 执行边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.root_entry_import_pass` | root entry import plan |
| 模块树 | `runtime.root_entry_import_pass` | 方案登记 |

---

## 方案判定

采用 two-handler root entry pilot，不把 `report_ops`、test-only import 或 `src/runtime/mod.rs` 根桥合并进同一批。

### 下一批允许修改

BE-001CY-03 只允许处理:

```text
src/runtime/event_stream.rs
src/runtime/evidence_health.rs
```

允许动作:

1. 将两个文件顶部的 `use super::*` 收敛为显式 import。
2. `event_stream` 只补足 `auth::UserId`、`AppState`、`State`、`Path`、`Sse`、`Event`、`KeepAlive`、`Infallible`、`stream!`、`Duration`、`sleep`、`StatusCode`、`SSE_EVENT_DELAY_MS`、`load_run_record_from_state`、`json_sse_event` 等真实依赖。
3. `evidence_health` 只补足 `State`、`Json`、`AppState`、`StatusCode`、`RuntimeEvidenceReportRecord`、`RuntimeEvidenceReportStatusCounts`、`RuntimeReportLifecycleStatus`、`RuntimeEvidenceHealthResponse`、`RuntimeEvidenceCleanupRequest`、`RuntimeEvidenceCleanupResponse`、runtime report store helpers、cleanup helpers、`current_time_ms`、`io_error` 等真实依赖。
4. 如 `cargo check` 证明某个 helper visibility 不足，只允许最小化调整父级受控 surface，并必须在抽离记录中说明原因。

---

## 明确排除

BE-001CY-03 不允许处理:

```text
src/runtime/report_ops.rs
src/runtime/run_guard.rs
src/runtime/mod.rs
```

排除理由:

1. `src/runtime/report_ops.rs` 是 report ops parent facade，且其 child 仍有 3 个 `super` 依赖文件；应另起 `runtime.report_ops_import_pass` 单子叶，不与 route-facing root child 混批。
2. `src/runtime/run_guard.rs` 的 `use super::*` 只存在于 `#[cfg(test)] mod tests`，属于 test-only super import；不应伪装成业务 parent import bridge 残余。
3. `src/runtime/mod.rs` 是 parent import bridge 根部，必须等 root child / subtrees 的 import pass 收敛后再单独判断。

---

## 回退点

若 BE-001CY-03 失败，回退范围仅限:

1. `src/runtime/event_stream.rs` 的显式 import 改写。
2. `src/runtime/evidence_health.rs` 的显式 import 改写。
3. 与 BE-001CY-03 同批新增的治理文档和门禁锚点。

不得回退 `runtime.root_support_import_pilot`、`runtime.root_entry_import_pass` 基线、`runtime.parent_include_cleanup` 或任何已 closeout 子模块。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001CY-03 实际 import rewrite 后至少补跑:

```powershell
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_sse
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_v1_ops_health
```

若实际改动影响 report output，还必须补跑 `cargo test -p quantpilot --test api_v1_reports`。

---

## 下一步

下一步只允许进入:

```text
BE-001CY-03 runtime.root_entry_import_pass 实际抽离
```

BE-001CY-03 只允许改写 `src/runtime/event_stream.rs` 与 `src/runtime/evidence_health.rs` 的 parent wildcard import。不得顺手处理 `report_ops`、`run_guard`、`src/runtime/mod.rs`、run/backtest/mutation 子树、sibling horizontal link 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001CY-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. BE-001CY-03 只允许处理 `event_stream` 与 `evidence_health` 两个文件。
3. `report_ops` 被拆出到后续 `runtime.report_ops_import_pass`。
4. `run_guard` 当前只是 test-only super import。
5. parent import bridge 尚未消除，剩余依赖文件数仍为 44。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成、parent import bridge 已完全清除或 release transition 已启动。

---

## 验收标准

1. `314-runtime.root_entry_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定 BE-001CY-03 只处理 `event_stream` 与 `evidence_health`。
3. `report_ops`、test-only `run_guard` 和 `src/runtime/mod.rs` 均被明确排除。
4. 下一步固定为 BE-001CY-03 `runtime.root_entry_import_pass` 实际抽离。
5. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
