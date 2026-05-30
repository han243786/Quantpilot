# v4.16.0 runtime.evidence_health 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CJ-02  
> 基准: `274-runtime.evidence_health单子叶等价基线.md`、`273-backend.runtime第二轮父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.evidence_health`  
> 目标文件: `src/runtime/evidence_health.rs`  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CJ-02 `runtime.evidence_health` 抽离方案 | 方案冻结 |
| 规范矩阵 | 允许迁移清单、父级 re-export、测试门禁、回退点 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.evidence_health` | planned child 落地前置方案 |
| 模块树 | `runtime.evidence_health` | 抽离路径登记 |

---

## 方案结论

BE-001CJ-03 才允许实际抽离。允许迁移清单仅限:

- `runtime_report_status_counts`
- `get_runtime_evidence_health`
- `cleanup_runtime_evidence`

目标文件固定为:

```text
src/runtime/evidence_health.rs
```

父级 `src/runtime/mod.rs` 只允许新增:

```rust
mod evidence_health;

pub(crate) use evidence_health::{cleanup_runtime_evidence, get_runtime_evidence_health};
```

`runtime_report_status_counts` 作为 child private helper，不对父级 re-export。

---

## 现有测试策略

BE-001CJ-01 已确认 `tests/api_evidence_contract.rs` 直接覆盖 evidence health / cleanup:

- `runtime_evidence_health_tracks_metrics_and_cleanup_preserves_reports`
- `runtime_evidence_contract_snapshot_matches_fixture`

同时 `tests/api_mutation.rs` 覆盖 health endpoint 与 mutation metrics 的联动。因此本方案选择直接进入实际抽离，不再新建 endpoint smoke 文件。

---

## 实际抽离步骤

BE-001CJ-03 执行时只能:

1. 创建 `src/runtime/evidence_health.rs`。
2. 在 child 顶部使用 `use super::*;` 继承父模块既有 schema、state、persistence helper 和 error helper 可见面。
3. 将 `runtime_report_status_counts`、`get_runtime_evidence_health`、`cleanup_runtime_evidence` 从 `src/runtime/mod.rs` 原样迁入 child。
4. 在 `src/runtime/mod.rs` 增加 `mod evidence_health;`。
5. 在 `src/runtime/mod.rs` 增加受控 `pub(crate) use evidence_health::{cleanup_runtime_evidence, get_runtime_evidence_health};`。
6. 保持 `src/backend/runtime/routes/evidence.rs` 不变，使 route facade 继续调用 `runtime_handlers::get_runtime_evidence_health` 与 `runtime_handlers::cleanup_runtime_evidence`。

---

## 禁止事项

BE-001CJ-03 不得:

- 修改 `src/backend/runtime/routes/evidence.rs`。
- 修改 `/api/runtime/evidence/health` 或 `/api/runtime/evidence/cleanup` path/method。
- 迁移 `RuntimeEvidenceHealthResponse`、`RuntimeEvidenceCleanupRequest`、`RuntimeEvidenceCleanupResponse` 或 `RuntimeEvidenceReportStatusCounts`。
- 迁移 `runtime_evidence_cleanup_policy`、`cleanup_transient_runtime_report_outputs`、`list_runtime_report_records` 或 `current_time_ms`。
- 迁移 `AppState`、metrics owner、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner。
- 处理 `runtime.report_ops`、`backend.runtime.routes.evidence` 或 shared helpers。
- 启动 release transition guard。

---

## 父级通信规则

抽离后通信路径必须保持:

```text
backend.runtime.routes.evidence -> src/runtime/mod.rs re-export -> runtime.evidence_health
```

开发者未明确进入发布版本过渡前，不允许 route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner、metrics owner 或 `AppState` 横向直连 `src/runtime/evidence_health.rs`。

---

## 回退点

若 BE-001CJ-03 抽离后出现编译或等价失败，只能按本批迁移清单回退:

1. 删除 `mod evidence_health;` 与 `pub(crate) use evidence_health::{...}`。
2. 将三个迁移项放回 `src/runtime/mod.rs` 原位置。
3. 删除新建 child 文件。

不得借回退修改 route facade、schema、persistence、state 或 frontend caller。

---

## 验证要求

BE-001CJ-03 提交前必须执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_mutation
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CJ-03 runtime.evidence_health 实际抽离
```

BE-001CJ-03 完成后必须再进入 BE-001CJ-04 单叶 closeout，判断 `runtime.evidence_health` 是否还值得继续细拆。不得跳过 closeout 回到 `backend.runtime` 父叶。

---

## 幻觉检查点

AI 声称 BE-001CJ-02 完成时，必须说明:

1. 当前仍是 `no code movement` 抽离方案。
2. `src/runtime/evidence_health.rs` 尚未创建。
3. 允许迁移清单只有 `runtime_report_status_counts`、`get_runtime_evidence_health`、`cleanup_runtime_evidence`。
4. `src/backend/runtime/routes/evidence.rs`、schema owner、runtime persistence owner、metrics owner、`AppState` 和 release transition guard 均不得迁移。
5. 下一步只能进入 BE-001CJ-03 实际抽离。

不得宣称 handler 已迁移、child 已落地、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `275-runtime.evidence_health抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001CJ-03 的允许迁移清单、父级 re-export、测试门禁和回退点已冻结。
3. 治理门禁能阻止跳过 BE-001CJ-03 或迁移 handler 之外 owner。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
