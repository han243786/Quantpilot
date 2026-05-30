# v4.16.0 runtime.response_support 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CP-03
> 基准: `290-runtime.response_support抽离方案.md`、`289-runtime.response_support单子叶等价基线.md`
> 目标子叶: `runtime.response_support`
> 模块树坐标: `root.backend.runtime.runtime.response_support`
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CP-03 `runtime.response_support` 实际抽离 | 单子叶抽离记录 |
| 规范矩阵 | 父子通信、response DTO visibility、plain import、release transition guard | 落地 |
| 引导矩阵 | `root.backend.runtime.runtime.response_support` | child 文件落地 |
| 模块树 | `runtime.response_support` | 白箱真实文件登记 |

---

## 代码变更

本批创建:

```text
src/runtime/response_support.rs
```

并更新:

```text
src/runtime/mod.rs
src/runtime/run.rs
```

`src/runtime/mod.rs` 新增 child 声明:

```rust
mod response_support;
```

父级以 plain import 回填 caller-facing response surface:

```rust
use response_support::{DiscardRuntimeArtifactResponse, MergeRecordEntry, MergeRecordsResponse};
```

未使用 `pub(crate) use response_support::{...};`，没有扩大 crate public surface。

`src/runtime/run.rs` 已降为 drained include:

```rust
// Drained include retained until the runtime parent support closeout.
```

`include!("run.rs")` 继续保留，parent include cleanup 只能由后续父叶残余判断处理。

---

## 已迁移 DTO

以下 3 个 item 已迁入 `src/runtime/response_support.rs`:

| item | 迁移后 type visibility | field visibility | 等价约束 |
| --- | --- | --- | --- |
| `DiscardRuntimeArtifactResponse` | `pub(crate)` | `pub(super)` | 保持 discard endpoints 的 `discarded_id` 与 `discarded_kind` JSON contract |
| `MergeRecordsResponse` | `pub(crate)` | `pub(super)` | 保持 `/api/v1/merge/records` 的 records / totals response contract |
| `MergeRecordEntry` | `pub(super)` | `pub(super)` | 保持 merge record item projection contract |

编译未要求把 `MergeRecordEntry` 提升为 `pub(crate)`，因此维持 BE-001CP-02 的优先方案: type `pub(super)`、fields `pub(super)`。

---

## 调用方等价

以下调用方文件未改动，继续通过 `use super::*` 访问父级受控 surface:

- `src/runtime/run/record_store.rs`
- `src/runtime/backtest/record_store.rs`
- `src/runtime/backtest/record_lifecycle.rs`
- `src/runtime/report_ops/merge_generation_health.rs`

父子通信路径保持:

```text
runtime child callers
  -> src/runtime/mod.rs controlled response surface
  -> runtime.response_support
```

开发者未明确进入发布版本过渡前，不得让 sibling child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner、`AppState` 或任何横向连接直接依赖 `runtime.response_support`。

---

## 明确未迁移

- run guard: `RunInProgressGuard`。
- experiment limit: `MAX_EXPERIMENT_VARIANTS`。
- parent include cleanup: `include!("run.rs")`、`include!("mutation.rs")`、`include!("backtest.rs")` 均保留。
- query support: `src/runtime/query_support.rs` 未改动。
- schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 与 release transition guard。
- `runtime.report_ops.merge_generation_health`、`runtime.run.record_store`、`runtime.backtest.record_store`、`runtime.backtest.record_lifecycle` closed child body 均未迁移。

---

## 验证要求

本批提交前必须执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_v1_reports
cargo test -p quantpilot --test api_v1_ops_health
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001CP-04 runtime.response_support 单叶 closeout
```

BE-001CP-04 只判断 response_support 本叶是否还值得继续拆分，不得跳过 closeout 处理 `RunInProgressGuard`、`MAX_EXPERIMENT_VARIANTS`、parent include cleanup 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CP-03 完成时，必须说明:

1. `src/runtime/response_support.rs` 已创建。
2. `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse` 与 `MergeRecordEntry` 已迁入 child。
3. `DiscardRuntimeArtifactResponse` 与 `MergeRecordsResponse` 类型本体为 `pub(crate)`，字段为 `pub(super)`。
4. `MergeRecordEntry` 类型本体与字段均为 `pub(super)`，未被编译器要求提升到 `pub(crate)`。
5. 父级只保留 `mod response_support;` 与普通 `use response_support::{...};`，没有 `pub(crate) use`。
6. `src/runtime/run.rs` 只剩 drained include 注释，但 `include!("run.rs")` 仍保留。
7. `RunInProgressGuard`、`MAX_EXPERIMENT_VARIANTS`、`include!("mutation.rs")`、`include!("backtest.rs")`、`src/runtime/query_support.rs` 均未迁移或删除。
8. schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 与 release transition guard 均未改变。
9. 下一步只能进入 BE-001CP-04 单叶 closeout。

不得宣称 `backend.runtime` 已完成、parent include 已删除、run guard 已处理、experiment limit 已处理、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `src/runtime/response_support.rs` 进入模块树和全量树。
2. `291-runtime.response_support抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
3. 3 个 response DTO 只从父级抽离到 child，不改变 discard endpoints 或 `/api/v1/merge/records` response contract。
4. Rust 等价测试、治理门禁、全量树覆盖和 `git diff --check` 均通过。
