# v4.16.0 backend.runtime.routes 第二轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001BR-01
> 基准: `127-backend.runtime.routes父叶残余判断.md`、`131-backend.runtime.routes.mutation单叶closeout.md`、`229-runtime.mutation.ai_proposal第九轮父叶残余判断.md`
> 判定: `backend.runtime.routes` 第二轮父叶残余判断完成。`backend.runtime.routes.run`、`backend.runtime.routes.backtest` 与 `backend.runtime.routes.mutation` 三个 route facade 均已完成当前递归范围内 closeout；`runtime.mutation.parameter_mutation` 与 `runtime.mutation.ai_proposal` handler 域也已回流到父级。但父 route aggregate 仍直接持有 experiment / evidence / report_ops / event_stream 等路线，因此父叶继续保持 `stop_split: false`。下一步只能进入 BE-001BS-01 `backend.runtime.routes.experiment` 单子叶等价基线。
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BR-01 route aggregate 第二轮父叶残余判断 | 队列回流 |
| 规范矩阵 | closed child 不回改、父子通信、发布过渡保护 | `stop_split: false` 固化 |
| 引导矩阵 | `root.backend.runtime.routes` | 父叶残余判断 |
| 模块树 | `backend.runtime.routes` | 保持打开并登记下一候选 |

---

## 当前父叶真实形态

`src/backend/runtime/routes.rs` 当前通过三个 child route facade 委托已关闭分支:

```rust
pub mod backtest;
pub mod mutation;
pub mod run;

let router = backtest::register_routes(router);
let router = run::register_routes(router);
let router = mutation::register_routes(router);
```

已关闭 route child:

| 子叶 | 文件 | closeout |
| --- | --- | --- |
| `backend.runtime.routes.run` | `src/backend/runtime/routes/run.rs` | `stop_split: true` |
| `backend.runtime.routes.backtest` | `src/backend/runtime/routes/backtest.rs` | `stop_split: true` |
| `backend.runtime.routes.mutation` | `src/backend/runtime/routes/mutation.rs` | `stop_split: true` |

已关闭 handler 回流:

| handler 域 | 结果 |
| --- | --- |
| `runtime.run.*` | run handler 子叶当前范围关闭 |
| `runtime.event_stream` | SSE handler 关闭，但 route facade cleanup 未做 |
| `runtime.backtest.*` | backtest handler 子树当前范围关闭 |
| `runtime.mutation.parameter_mutation` | 父叶当前范围关闭 |
| `runtime.mutation.ai_proposal` | 父叶当前范围关闭 |

---

## 剩余 route aggregate 候选

`src/backend/runtime/routes.rs` 仍直接持有以下路线，因此父叶不能 closeout:

| 候选 | 当前 route / handler | 判定 |
| --- | --- | --- |
| `backend.runtime.routes.experiment` | `/api/runtime/experiments/backtest-sweep`、`/api/runtime/experiments`、`/api/runtime/experiments/:experiment_id/save`、`/api/runtime/experiments/:experiment_id` -> `start_backtest_experiment`、`list_experiments`、`save_experiment_record`、`get_experiment_detail`、`discard_experiment_record` | 下一候选 |
| `backend.runtime.routes.evidence` | `/api/runtime/evidence/health`、`/api/runtime/evidence/cleanup` -> `get_runtime_evidence_health`、`cleanup_runtime_evidence` | 后续候选 |
| `backend.runtime.routes.report_ops` | `/api/runtime/reports*`、`/api/v1/reports/*`、`/api/v1/storage/health`、`/api/v1/runtime/generations`、`/api/v1/merge/records` | 后续候选 |
| `backend.runtime.routes.event_stream` | `/api/runtime/runs/:run_id/events` -> `stream_run_events` | 后续 cleanup 候选 |

选择 `backend.runtime.routes.experiment` 作为 BE-001BS-01 的原因: experiment route group 仍集中四条 route / 五个 handler，且 handler 域 `runtime.backtest.experiment_sweep` 已完成当前范围 closeout，先建立 route facade 等价基线可以让 route aggregate 继续收窄，同时不触碰 handler、schema、state 或 persistence owner。

---

## 非目标边界

BE-001BR-01 不迁移、不修改:

- `src/backend/runtime/routes.rs` 中任何 route。
- planned `src/backend/runtime/routes/experiment.rs`。
- `src/runtime/backtest/experiment_sweep.rs`。
- `src/runtime/event_stream.rs`。
- `src/runtime/mod.rs`。
- `AppState`。
- schema owner。
- frontend caller。
- runtime persistence owner。
- report/evidence/ops handler。
- release transition guard。

---

## 父子通信规则

当前仍固定:

```text
backend.interface_boundary
  -> backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.{run,backtest,mutation}
  -> runtime handlers
```

BE-001BS-01 之前不得创建 `src/backend/runtime/routes/experiment.rs`，不得把 experiment handler 从 `src/runtime/backtest/experiment_sweep.rs` 迁出，不得让 route facade 直接横向连接 handler child，也不得提出发布版本过渡。

---

## 回归保护

本批为治理收口批次，只运行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续 BE-001BS route facade 实际抽离时再补跑:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
```

---

## 下一步

下一步只能进入:

```text
BE-001BS-01 backend.runtime.routes.experiment 单子叶等价基线
```

该基线只允许冻结 experiment route group 的 path、method、handler owner、父级委托、回归证据和非目标边界；不得直接创建 route 子文件、不得移动 handler、不得改变 `AppState` / schema owner / frontend caller / runtime persistence owner / release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BR-01 完成时，必须说明 `backend.runtime.routes` 父叶仍是 `stop_split: false`，只是 `run`、`backtest`、`mutation` 三个 route child 及其当前 handler 回流已经完成。不得宣称 experiment/evidence/report_ops/event_stream route 已迁移，不得宣称 Rust backend 重构完成，不得宣称发布过渡已启动。

---

## 验收标准

1. `230-backend.runtime.routes第二轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树明确 `backend.runtime.routes` 仍为 `stop_split: false`。
3. 下一步固定为 BE-001BS-01 `backend.runtime.routes.experiment` 单子叶等价基线。
4. 本批保持 `no code movement`。
