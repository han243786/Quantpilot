# v4.16.0 runtime.evidence_health 单叶 closeout

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CJ-04  
> 基准: `276-runtime.evidence_health抽离记录.md`、`275-runtime.evidence_health抽离方案.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.evidence_health`  
> 判定: `runtime.evidence_health stop_split: true`  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CJ-04 `runtime.evidence_health` 单叶 closeout | closeout |
| 规范矩阵 | 单叶停止条件、禁止微拆、父级回归顺序 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.evidence_health` | 子叶收口 |
| 模块树 | `runtime.evidence_health` | `stop_split: true` |

---

## closeout 判定

`runtime.evidence_health` 当前不继续拆成 health / cleanup 微叶，设置:

```text
runtime.evidence_health stop_split: true
```

理由:

1. `get_runtime_evidence_health` 与 `cleanup_runtime_evidence` 共同构成 runtime evidence support surface。
2. 两个 handler 共享 `RuntimeEvidence*` schema、report store 读取、cleanup policy 和 persistence helper 边界。
3. `runtime_report_status_counts` 只是 health response 私有 helper，不形成独立 owner。
4. 继续拆成 `evidence_health.health` / `evidence_health.cleanup` 只会增加父级 re-export 和治理登记成本，不会形成独立状态机、schema owner、persistence owner、metrics owner 或 release transition guard。

---

## 当前真实结构

已落地 child:

```text
src/runtime/evidence_health.rs
```

父级 `src/runtime/mod.rs` 只保留:

```rust
mod evidence_health;

pub(crate) use evidence_health::{cleanup_runtime_evidence, get_runtime_evidence_health};
```

route facade:

```text
src/backend/runtime/routes/evidence.rs
```

仍保持不变。

---

## 明确排除

- 不继续细拆 health / cleanup 微叶。
- 不修改 `src/backend/runtime/routes/evidence.rs`。
- 不迁移 `RuntimeEvidenceHealthResponse`、`RuntimeEvidenceCleanupRequest`、`RuntimeEvidenceCleanupResponse` 或 `RuntimeEvidenceReportStatusCounts`。
- 不迁移 `runtime_evidence_cleanup_policy`、`cleanup_transient_runtime_report_outputs`、`list_runtime_report_records` 或 `current_time_ms`。
- 不迁移 `AppState`、metrics owner、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner。
- 不处理 `runtime.report_ops` 或 `backend.runtime.routes.evidence`。
- 不启动 release transition guard。

---

## 验证要求

本批为 `no code movement` closeout，提交前仍需执行:

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
BE-001CK-01 backend.runtime 第三轮父叶残余判断
```

BE-001CK-01 需要判断 `runtime.report_ops` 与 `runtime.evidence_health` 均 closeout 后，`backend.runtime` 是否仍存在其他值得继续抽离的 handler / helper 残余。不得从 `runtime.evidence_health` 继续细拆，不得启动 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CJ-04 完成时，必须说明:

1. 本批次是 `no code movement` closeout。
2. `runtime.evidence_health stop_split: true`。
3. 不继续拆 health / cleanup 微叶。
4. `src/runtime/evidence_health.rs` 仍承接两个 public handler 与 `runtime_report_status_counts`。
5. 下一步只能进入 BE-001CK-01 `backend.runtime` 第三轮父叶残余判断。
6. route facade、schema owner、runtime persistence owner、metrics owner、`AppState` 和 release transition guard 均未迁移。

不得宣称 backend 父叶已完成、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `277-runtime.evidence_health单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.evidence_health` 设置为 `stop_split: true`。
3. 全局递归下一步固定为 BE-001CK-01 `backend.runtime` 第三轮父叶残余判断。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
