# v4.16.0 runtime.evidence_health 抽离记录

> 版本类型: MINOR architecture / implementation  
> 执行档位: 标准  
> 批次: BE-001CJ-03  
> 基准: `275-runtime.evidence_health抽离方案.md`、`274-runtime.evidence_health单子叶等价基线.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.evidence_health`  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CJ-03 `runtime.evidence_health` 实际抽离 | 实施 |
| 规范矩阵 | 允许迁移清单、父级 re-export、禁止 owner 横移 | 执行 |
| 引导矩阵 | `root.backend.runtime.runtime.evidence_health` | child 文件落地 |
| 模块树 | `runtime.evidence_health` | 白箱真实文件更新 |

---

## 抽离结果

已创建:

```text
src/runtime/evidence_health.rs
```

已迁入 child:

- `runtime_report_status_counts`
- `get_runtime_evidence_health`
- `cleanup_runtime_evidence`

父级 `src/runtime/mod.rs` 只保留:

```rust
mod evidence_health;

pub(crate) use evidence_health::{cleanup_runtime_evidence, get_runtime_evidence_health};
```

`runtime_report_status_counts` 保持 child private helper，未对父级 re-export。

---

## 未变更边界

- `src/backend/runtime/routes/evidence.rs` 未修改，仍通过 `runtime_handlers::get_runtime_evidence_health` 与 `runtime_handlers::cleanup_runtime_evidence` 调用父级兼容出口。
- `/api/runtime/evidence/health` 与 `/api/runtime/evidence/cleanup` path/method 未改变。
- `RuntimeEvidenceHealthResponse`、`RuntimeEvidenceCleanupRequest`、`RuntimeEvidenceCleanupResponse` 与 `RuntimeEvidenceReportStatusCounts` 未迁移。
- `runtime_evidence_cleanup_policy`、`cleanup_transient_runtime_report_outputs`、`list_runtime_report_records` 与 `current_time_ms` 未迁移。
- `AppState`、metrics owner、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、shared helpers 与 release transition guard 均未迁移。

---

## 等价保护

BE-001CJ-03 的等价门禁:

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

`api_evidence_contract` 继续覆盖 health / cleanup 等价；`api_mutation` 继续覆盖 mutation metrics 联动。

---

## 下一步

下一步只允许进入:

```text
BE-001CJ-04 runtime.evidence_health 单叶 closeout
```

BE-001CJ-04 需要判断 `runtime.evidence_health` 是否值得继续拆分为 health / cleanup 微叶。不得跳过 closeout 回到 `backend.runtime` 父叶，不得迁移 schema、runtime persistence、metrics owner、`AppState` 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CJ-03 完成时，必须说明:

1. `src/runtime/evidence_health.rs` 已创建。
2. `runtime_report_status_counts`、`get_runtime_evidence_health`、`cleanup_runtime_evidence` 已从 `src/runtime/mod.rs` 迁入 child。
3. 父级只保留 `mod evidence_health` 与受控 `pub(crate) use evidence_health::{cleanup_runtime_evidence, get_runtime_evidence_health};`。
4. `src/backend/runtime/routes/evidence.rs`、schema owner、runtime persistence owner、metrics owner、`AppState` 和 release transition guard 均未迁移。
5. 下一步只能进入 BE-001CJ-04 单叶 closeout。

不得宣称本叶已 closeout、`stop_split: true` 已设置、backend 父叶已完成、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `276-runtime.evidence_health抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/evidence_health.rs` 进入全量树与模块树真实文件列表。
3. route facade、schema、persistence、metrics、state owner 均保持未迁移。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
