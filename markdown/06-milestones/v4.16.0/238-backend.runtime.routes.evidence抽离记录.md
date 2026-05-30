# v4.16.0 backend.runtime.routes.evidence 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BU-03  
> 基准: `237-backend.runtime.routes.evidence抽离方案.md`、`236-backend.runtime.routes.evidence单子叶等价基线.md`、`235-backend.runtime.routes第三轮父叶残余判断.md`  
> 判定: `backend.runtime.routes.evidence` route facade 实际抽离已完成。`src/backend/runtime/routes/evidence.rs` 已创建并承接 `/api/runtime/evidence/health` 与 `/api/runtime/evidence/cleanup` 两条 route registration；父级通过 `evidence::register_routes(router)` 委托并保持 `event_stream -> evidence -> mutation` 顺序。handler、schema owner、`AppState`、frontend caller、runtime persistence owner 和 release transition guard 均未迁移。  
> 代码动作: route facade extraction only

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BU-03 evidence route facade 实际抽离 | 实际抽离 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 验证 |
| 引导矩阵 | `root.backend.runtime.routes.evidence` | 更新真实文件 |
| 模块树 | `backend.runtime.routes.evidence` | `stop_split: pending` |

---

## 实际代码变更

| 文件 | 动作 | 说明 |
| --- | --- | --- |
| `src/backend/runtime/routes/evidence.rs` | 新增 | 承接 evidence route child facade |
| `src/backend/runtime/routes.rs` | 修改 | 新增 `pub mod evidence`，父级调用 `evidence::register_routes(router)` |

当前父级委托顺序:

```text
backtest -> run -> event_stream -> evidence -> mutation -> report_ops -> experiment -> ops
```

本批只迁移 route registration:

| route | method | handler |
| --- | --- | --- |
| `/api/runtime/evidence/health` | GET | `runtime_handlers::get_runtime_evidence_health` |
| `/api/runtime/evidence/cleanup` | POST | `runtime_handlers::cleanup_runtime_evidence` |

---

## 等价边界

保持不变:

- `get_runtime_evidence_health` handler body。
- `cleanup_runtime_evidence` handler body。
- `RuntimeEvidenceHealthResponse`、`RuntimeEvidenceCleanupRequest`、`RuntimeEvidenceCleanupResponse` schema owner。
- `runtime_evidence_cleanup_policy`。
- `cleanup_transient_runtime_report_outputs`。
- `list_runtime_report_records`。
- `AppState`、`state.report_store_dir`、`state.evidence_metrics`。
- frontend caller。
- runtime persistence owner。
- release transition guard。
- report_ops route group。
- event_stream handler 或 SSE 语义。

---

## 父子通信规则

实际形态固定为:

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.evidence
  -> crate::runtime::{get_runtime_evidence_health, cleanup_runtime_evidence}
```

`backend.runtime.routes.evidence` 只拥有 route facade，不拥有 handler、state、schema、frontend caller 或 runtime persistence owner。开发者未明确进入发布版本过渡前，AI 不得主动提出横向连接或性能旁路。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001BU-04 backend.runtime.routes.evidence 单叶 closeout
```

closeout 需要判断 `backend.runtime.routes.evidence` 是否值得继续细拆。不得跳过 closeout 直接处理 report_ops、event_stream 或 `backend.runtime.routes` 父叶残余判断。

---

## 幻觉检查点

AI 声称 BE-001BU-03 完成时，必须说明只完成 evidence route facade 实际抽离；`src/backend/runtime/routes/evidence.rs` 已创建，但 handler 与 state/persistence owner 均未迁移。不得宣称 cleanup implementation 已迁移、report persistence owner 已迁移、`backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `src/backend/runtime/routes/evidence.rs` 存在并只注册两条 evidence route。
2. `src/backend/runtime/routes.rs` 通过 `evidence::register_routes(router)` 委托，保留 `event_stream -> evidence -> mutation` 顺序。
3. `238-backend.runtime.routes.evidence抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
4. handler、schema owner、`AppState`、frontend caller、runtime persistence owner 和 release transition guard 均未迁移。
5. 下一步固定为 BE-001BU-04 单叶 closeout。
