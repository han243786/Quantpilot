# v4.16.0 runtime.evidence_health 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CJ-01  
> 基准: `273-backend.runtime第二轮父叶残余判断.md`、`239-backend.runtime.routes.evidence单叶closeout.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.evidence_health`  
> 模块树坐标: `root.backend.runtime.runtime.evidence_health`  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CJ-01 `runtime.evidence_health` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | 父子通信、禁止跳步、已存在测试基线、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.evidence_health` | 新增 planned 子叶坐标 |
| 模块树 | `runtime.evidence_health` | 白箱登记 |

---

## 当前真实结构

已 closeout sibling / 父级:

- `backend.runtime.routes stop_split: true`
- `backend.runtime.routes.evidence stop_split: true`
- `runtime.report_ops stop_split: true`

本批冻结的父级残余:

- `get_runtime_evidence_health`
- `cleanup_runtime_evidence`
- `runtime_report_status_counts`

当前真实文件:

```text
src/runtime/mod.rs
src/backend/runtime/routes/evidence.rs
src/frontend_api_types.rs
src/runtime_persistence.rs
tests/api_evidence_contract.rs
tests/api_mutation.rs
```

planned child 文件尚未创建。BE-001CJ-01 只建立等价基线，不创建 `src/runtime/evidence_health.rs`，不迁移 handler 或 helper。

---

## 路由与 handler 基线

| Endpoint | Method | Handler | 当前 handler 文件 | Route facade |
| --- | --- | --- | --- | --- |
| `/api/runtime/evidence/health` | GET | `get_runtime_evidence_health` | `src/runtime/mod.rs` | `src/backend/runtime/routes/evidence.rs` |
| `/api/runtime/evidence/cleanup` | POST | `cleanup_runtime_evidence` | `src/runtime/mod.rs` | `src/backend/runtime/routes/evidence.rs` |

`backend.runtime.routes.evidence` 已经只承担 route registration；handler owner 仍在 `src/runtime/mod.rs`。后续若实际抽离，必须由 `src/runtime/mod.rs` 通过受控 re-export 暴露给 route facade，不能让 route facade 横向直连 planned child。

---

## 白箱边界

| public / helper | 输入 | 状态读取 / 依赖 | 输出 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `get_runtime_evidence_health` | `State<AppState>` | `state.report_store_dir`、`list_runtime_report_records`、`state.evidence_metrics.snapshot()`、`runtime_report_status_counts`、`runtime_evidence_cleanup_policy` | `RuntimeEvidenceHealthResponse` | 不得迁移 report store owner、metrics owner、cleanup policy owner 或 schema owner |
| `cleanup_runtime_evidence` | `State<AppState>`、`RuntimeEvidenceCleanupRequest` | `runtime_evidence_cleanup_policy`、`cleanup_transient_runtime_report_outputs`、`current_time_ms`、`list_runtime_report_records` | `RuntimeEvidenceCleanupResponse` | 不得迁移 transient cleanup implementation、clock helper、report record persistence 或 schema owner |
| `runtime_report_status_counts` | `&[RuntimeEvidenceReportRecord]` | `RuntimeReportLifecycleStatus` 枚举 | `RuntimeEvidenceReportStatusCounts` | 不得改变 status mapping、计数字段或 report lifecycle enum owner |

`RuntimeEvidenceHealthResponse`、`RuntimeEvidenceCleanupRequest` 与 `RuntimeEvidenceCleanupResponse` 当前仍由 `src/frontend_api_types.rs` 提供。`runtime_evidence_cleanup_policy`、`cleanup_transient_runtime_report_outputs` 与 `list_runtime_report_records` 当前仍由 `src/runtime_persistence.rs` 提供。本基线不迁移 schema owner、runtime persistence owner、storage lifecycle owner、metrics owner 或 `AppState`。

---

## 现有等价证据

当前已有专门自动化覆盖:

- `tests/api_evidence_contract.rs::runtime_evidence_health_tracks_metrics_and_cleanup_preserves_reports`
- `tests/api_evidence_contract.rs::runtime_evidence_contract_snapshot_matches_fixture`

补充广义回归覆盖:

- `tests/api_mutation.rs` 读取 `/api/runtime/evidence/health` 验证 mutation 指标联动。

因此 BE-001CJ-02 抽离方案不需要先补 endpoint smoke；但仍必须把 `api_evidence_contract` 作为实际抽离前后的硬门禁。

---

## 父级通信规则

`runtime.evidence_health` 后续若实际抽离，只能作为 `backend.runtime` 下的 runtime handler child。通信路径必须保持:

```text
backend.runtime.routes.evidence -> src/runtime/mod.rs re-export -> runtime.evidence_health
```

开发者未明确进入发布版本过渡前，不得让 route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner、metrics owner 或 `AppState` 横向直连该子叶。

---

## 明确排除

- 不处理 `backend.runtime.routes.evidence`，该 route child 已 closeout。
- 不处理 `runtime.report_ops`，该 sibling 已 closeout。
- 不创建 `src/runtime/evidence_health.rs`。
- 不迁移 `get_runtime_evidence_health`、`cleanup_runtime_evidence` 或 `runtime_report_status_counts`。
- 不迁移 `RuntimeEvidenceHealthResponse`、`RuntimeEvidenceCleanupRequest`、`RuntimeEvidenceCleanupResponse` 或 `RuntimeEvidenceReportStatusCounts`。
- 不迁移 `runtime_evidence_cleanup_policy`、`cleanup_transient_runtime_report_outputs`、`list_runtime_report_records`、`current_time_ms`。
- 不迁移 `AppState`、metrics owner、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner。
- 不处理 shared helpers 或 release transition guard。

---

## 验证要求

本批为 `no code movement` 等价基线，提交前仍需执行:

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
BE-001CJ-02 runtime.evidence_health 抽离方案
```

BE-001CJ-02 只能决定 planned child 文件、父级 re-export、允许迁移清单、验证命令和回退点；不得直接宣称 handler 已迁移。

---

## 幻觉检查点

AI 声称 BE-001CJ-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. planned child 文件 `src/runtime/evidence_health.rs` 尚未创建。
3. `get_runtime_evidence_health`、`cleanup_runtime_evidence` 与 `runtime_report_status_counts` 仍在 `src/runtime/mod.rs`。
4. `api_evidence_contract` 已覆盖 health / cleanup 等价基线，`api_mutation` 提供指标联动回归。
5. 下一步只能进入 BE-001CJ-02 抽离方案。
6. `backend.runtime.routes.evidence`、`runtime.report_ops`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、metrics owner、`AppState` 和 release transition guard 均未迁移。

不得宣称 handler 已抽离、backend 顶层完成、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `274-runtime.evidence_health单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.evidence_health` planned 子叶白箱节点。
3. 治理门禁能阻止跳过 BE-001CJ-02 直接创建 child 文件或迁移 handler。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
