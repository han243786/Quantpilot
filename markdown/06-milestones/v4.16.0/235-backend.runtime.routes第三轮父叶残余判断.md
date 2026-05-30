# v4.16.0 backend.runtime.routes 第三轮父叶残余判断

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BT-01  
> 基准: `230-backend.runtime.routes第二轮父叶残余判断.md`、`234-backend.runtime.routes.experiment单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: `backend.runtime.routes` 第三轮父叶残余判断完成。`run`、`backtest`、`mutation` 与 `experiment` 四个 route child 均已 closeout；父叶仍直接持有 evidence、report_ops 与 event_stream route，因此继续保持 `stop_split: false`。下一步只能进入 BE-001BU-01 `backend.runtime.routes.evidence` 单子叶等价基线。
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BT-01 route aggregate 第三轮父叶残余判断 | 队列回流 |
| 规范矩阵 | closed child 不回改、父子通信、发布过渡保护 | `stop_split: false` 固化 |
| 引导矩阵 | `root.backend.runtime.routes` | 父叶残余判断 |
| 模块树 | `backend.runtime.routes` | 保持打开并登记下一候选 |

---

## 当前父叶真实形态

`src/backend/runtime/routes.rs` 当前通过四个 child route facade 委托已关闭分支:

```rust
pub mod backtest;
pub mod experiment;
pub mod mutation;
pub mod run;

let router = backtest::register_routes(router);
let router = run::register_routes(router);
let router = mutation::register_routes(router);
let router = experiment::register_routes(router);
```

已关闭 route child:

| 子叶 | 文件 | closeout |
| --- | --- | --- |
| `backend.runtime.routes.run` | `src/backend/runtime/routes/run.rs` | `stop_split: true` |
| `backend.runtime.routes.backtest` | `src/backend/runtime/routes/backtest.rs` | `stop_split: true` |
| `backend.runtime.routes.mutation` | `src/backend/runtime/routes/mutation.rs` | `stop_split: true` |
| `backend.runtime.routes.experiment` | `src/backend/runtime/routes/experiment.rs` | `stop_split: true` |

---

## 剩余 route aggregate 候选

`src/backend/runtime/routes.rs` 仍直接持有以下路线，因此父叶不能 closeout:

| 候选 | 当前 route / handler | 判定 |
| --- | --- | --- |
| `backend.runtime.routes.evidence` | `/api/runtime/evidence/health`、`/api/runtime/evidence/cleanup` -> `get_runtime_evidence_health`、`cleanup_runtime_evidence` | 下一候选 |
| `backend.runtime.routes.report_ops` | `/api/runtime/reports*`、`/api/v1/merge/records`、`/api/v1/runtime/generations`、`/api/v1/storage/health`、`/api/v1/reports/*` | 后续候选 |
| `backend.runtime.routes.event_stream` | `/api/runtime/runs/:run_id/events` -> `stream_run_events` | 后续 cleanup 候选 |

选择 `backend.runtime.routes.evidence` 作为 BE-001BU-01 的原因:

- evidence health / cleanup 两条 route 同属 runtime evidence 运维边界，适合先建立小范围等价基线。
- `api_evidence_contract` 已能覆盖核心 response / cleanup 契约，回归证据清晰。
- report_ops 聚合面更宽，包含 runtime report、merge record、config generation、storage health 与 ops/audit/research reports，应在 evidence 收束后再处理。
- event_stream 仅单条 SSE route，handler `runtime.event_stream` 已 closeout；它适合作为后续 route cleanup，不应抢在 evidence pair 前面。

---

## 非目标边界

BE-001BT-01 不迁移、不修改:

- `src/backend/runtime/routes.rs` 中任何 route。
- planned `src/backend/runtime/routes/evidence.rs`。
- `get_runtime_evidence_health`。
- `cleanup_runtime_evidence`。
- `stream_run_events`。
- runtime report / merge / config / storage / ops report handler。
- `AppState`。
- schema owner。
- frontend caller。
- runtime persistence owner。
- release transition guard。

---

## 父子通信规则

当前固定:

```text
backend.interface_boundary
  -> backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.{run,backtest,mutation,experiment}
  -> runtime handlers
```

BE-001BU-01 之前不得创建 `src/backend/runtime/routes/evidence.rs`，不得迁移 evidence handler，不得改变 `AppState` / schema owner / frontend caller / runtime persistence owner，也不得提出发布版本过渡。

---

## 回归保护

本批为治理判断批次，只运行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续 BE-001BU route facade 实际抽离时再补跑:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
```

---

## 下一步

下一步只能进入:

```text
BE-001BU-01 backend.runtime.routes.evidence 单子叶等价基线
```

该基线只允许冻结 evidence route group 的 path、method、handler owner、父级委托、回归证据和非目标边界；不得直接创建 route 子文件、不得移动 handler、不得改变 `AppState` / schema owner / frontend caller / runtime persistence owner / release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BT-01 完成时，必须说明 `backend.runtime.routes` 父叶仍是 `stop_split: false`，只是 `run`、`backtest`、`mutation`、`experiment` 四个 route child 已 closeout。不得宣称 evidence/report_ops/event_stream route 已迁移，不得宣称 Rust backend 重构完成，不得宣称发布过渡已启动。

---

## 验收标准

1. `235-backend.runtime.routes第三轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树明确 `backend.runtime.routes` 仍为 `stop_split: false`。
3. 下一步固定为 BE-001BU-01 `backend.runtime.routes.evidence` 单子叶等价基线。
4. 本批保持 `no code movement`。
