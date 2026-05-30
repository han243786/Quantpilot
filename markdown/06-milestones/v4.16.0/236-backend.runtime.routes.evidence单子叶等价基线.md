# v4.16.0 backend.runtime.routes.evidence 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BU-01  
> 基准: `235-backend.runtime.routes第三轮父叶残余判断.md`、`234-backend.runtime.routes.experiment单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: `backend.runtime.routes.evidence` 单子叶等价基线已建立。当前 `no code movement`，只冻结 evidence route group 的 path、method、handler owner、父级委托、状态/持久化读取边界和回归证据。下一步只能进入 BE-001BU-02 抽离方案，不得创建 `src/backend/runtime/routes/evidence.rs` 或迁移 handler。
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BU-01 evidence route facade 单子叶基线 | 新建基线 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.evidence` | 新增单子叶基线 |
| 模块树 | `backend.runtime.routes.evidence` | `stop_split: pending` |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.evidence` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `src/backend/runtime/routes.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `root.backend.runtime.routes.evidence` |
| 真实文件 | `src/backend/runtime/routes.rs`、`src/runtime/mod.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`src/lib.rs`、`tests/api_evidence_contract.rs`、`tests/api_run.rs` |
| public 方法 | `backend.runtime.routes::register_routes`、`get_runtime_evidence_health`、`cleanup_runtime_evidence` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1`、`git diff --check` |

---

## 当前 route owner 基线

| route | method | handler | 当前 owner |
| --- | --- | --- | --- |
| `/api/runtime/evidence/health` | GET | `runtime_handlers::get_runtime_evidence_health` | `src/backend/runtime/routes.rs` |
| `/api/runtime/evidence/cleanup` | POST | `runtime_handlers::cleanup_runtime_evidence` | `src/backend/runtime/routes.rs` |

父级注册片段:

```rust
.route(
    "/api/runtime/evidence/health",
    get(runtime_handlers::get_runtime_evidence_health),
)
.route(
    "/api/runtime/evidence/cleanup",
    post(runtime_handlers::cleanup_runtime_evidence),
)
```

---

## Handler 等价边界

| handler | 输入 | 输出 | 关键依赖 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `get_runtime_evidence_health` | `State<AppState>` | `Json<RuntimeEvidenceHealthResponse>` | `list_runtime_report_records`、`state.evidence_metrics.snapshot()`、`runtime_report_status_counts`、`runtime_evidence_cleanup_policy` | 不得迁移 report store、metrics owner 或 response schema |
| `cleanup_runtime_evidence` | `State<AppState>`、`Json<RuntimeEvidenceCleanupRequest>` | `Json<RuntimeEvidenceCleanupResponse>` | `runtime_evidence_cleanup_policy`、`cleanup_transient_runtime_report_outputs`、`current_time_ms`、`list_runtime_report_records` | 不得迁移 cleanup implementation、report persistence owner 或 clock helper |

---

## 保留边界

BE-001BU-01 不迁移、不修改:

- `src/backend/runtime/routes.rs` 中任何 route registration。
- planned `src/backend/runtime/routes/evidence.rs`。
- `get_runtime_evidence_health`。
- `cleanup_runtime_evidence`。
- `runtime_evidence_cleanup_policy`。
- `cleanup_transient_runtime_report_outputs`。
- `list_runtime_report_records`。
- `RuntimeEvidenceHealthResponse` / `RuntimeEvidenceCleanupRequest` / `RuntimeEvidenceCleanupResponse` schema owner。
- `AppState`、`state.report_store_dir`、`state.evidence_metrics`。
- frontend caller。
- runtime persistence owner。
- release transition guard。

---

## 父子通信规则

当前固定:

```text
backend.runtime
  -> backend.runtime.routes
  -> crate::runtime::{get_runtime_evidence_health, cleanup_runtime_evidence}
```

BE-001BU-01 只登记计划中的 route child 坐标。下一步若进入抽离方案，也只能规划 route registration facade，不得迁移 evidence handler、report persistence owner、schema owner、AppState、frontend caller 或发布过渡连接。

---

## 回归证据

| 证据 | 覆盖 |
| --- | --- |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence health snapshot、cleanup response、report retention / transient output cleanup |
| `cargo test -p quantpilot --test api_run` | runtime run 与 evidence 侧效应兼容 |
| `cargo test --no-run` | 编译所有 Rust 测试目标 |
| `tools/check-matrix-governance.ps1` | 模块树 / 里程碑 / gate token 覆盖 |
| `tools/check-full-feature-tree.ps1` | 全量树路径覆盖 |

---

## 验证计划

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001BU-02 backend.runtime.routes.evidence 抽离方案
```

该方案只允许规划 `src/backend/runtime/routes/evidence.rs` route facade 与父级 `evidence::register_routes(router)` 委托；不得直接创建目标文件、不得迁移 handler、不得改变 `AppState` / schema owner / frontend caller / runtime persistence owner / release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BU-01 完成时，必须说明当前只是 `backend.runtime.routes.evidence` 等价基线，`src/backend/runtime/routes/evidence.rs` 尚未创建，route registration 与 handler 仍在原 owner。不得宣称 evidence route 已迁移、cleanup implementation 已迁移、report persistence owner 已迁移、`backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `236-backend.runtime.routes.evidence单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `backend.runtime.routes.evidence` 白箱节点，并标记 `stop_split: pending`。
3. 基线明确 health / cleanup route path、method、handler owner 和非目标边界。
4. 下一步固定为 BE-001BU-02 抽离方案。
5. 本批保持 `no code movement`。
