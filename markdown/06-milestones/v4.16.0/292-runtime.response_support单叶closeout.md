# v4.16.0 runtime.response_support 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CP-04
> 基准: `291-runtime.response_support抽离记录.md`、`290-runtime.response_support抽离方案.md`、`13-递归模块化全局根流程.md`
> 目标子叶: `runtime.response_support`
> 判定: `runtime.response_support stop_split: true`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CP-04 `runtime.response_support` 单叶 closeout | closeout |
| 规范矩阵 | 单叶停止条件、父子通信、response DTO visibility、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.response_support` | 子叶收口 |
| 模块树 | `runtime.response_support` | `stop_split: true` |

---

## closeout 判定

`runtime.response_support` 当前不继续拆成 discard response / merge records response 微叶，设置:

```text
runtime.response_support stop_split: true
```

理由:

1. 当前 child 只有 3 个 response DTO，全部围绕 runtime API response projection，没有独立状态、锁顺序、持久化 owner 或 schema owner。
2. `DiscardRuntimeArtifactResponse` 与 `MergeRecordsResponse` 类型本体保持 `pub(crate)`，字段已收敛为 `pub(super)`；`MergeRecordEntry` 类型本体与字段均为 `pub(super)`。
3. 继续拆成 discard / merge 微叶会增加父级 import、治理登记和 caller surface，但不会形成新的稳定白箱。
4. `/api/v1/merge/records` 的专门 smoke 已存在，discard endpoints 仍由 run/backtest record_store 等价测试覆盖。

---

## 当前真实结构

已落地 child:

```text
src/runtime/response_support.rs
```

child 内部保持:

- `DiscardRuntimeArtifactResponse`: type `pub(crate)`，fields `pub(super)`。
- `MergeRecordsResponse`: type `pub(crate)`，fields `pub(super)`。
- `MergeRecordEntry`: type `pub(super)`，fields `pub(super)`。

父级 `src/runtime/mod.rs` 保留受控 child 声明:

```rust
mod response_support;
```

父级只用普通 import 回填 caller-facing response surface:

```rust
use response_support::{DiscardRuntimeArtifactResponse, MergeRecordEntry, MergeRecordsResponse};
```

不得改为 `pub(crate) use response_support::{...};`。

`src/runtime/run.rs` 只剩:

```rust
// Drained include retained until the runtime parent support closeout.
```

`include!("run.rs")` 继续保留。

---

## 调用方等价

以下调用方仍通过 `use super::*` 访问父级受控 surface:

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

开发者未明确进入发布版本过渡前，不得让 sibling child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner 或 `AppState` 横向直连该 child。

---

## 明确排除

- 不继续细拆 discard response / merge records response 微叶。
- 不修改 `src/runtime/response_support.rs`。
- 不修改调用方文件，不改变 `use super::*` 兼容路径。
- 不迁移 `RunInProgressGuard`。
- 不迁移 `MAX_EXPERIMENT_VARIANTS`。
- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 不修改 `src/runtime/query_support.rs`。
- 不迁移 schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 或 release transition guard。

---

## 验证要求

本批为 `no code movement` closeout，提交前仍需执行:

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

下一步只允许进入:

```text
BE-001CQ-01 backend.runtime 第六轮父叶残余判断
```

BE-001CQ-01 需要重新审视 `backend.runtime` 父级在 `runtime.response_support` closeout 后是否仍存在值得抽离的 run guard / experiment limit / parent include residual。不得从 `runtime.response_support` 继续细拆，不得启动 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CP-04 完成时，必须说明:

1. 本批次是 `no code movement` closeout。
2. `runtime.response_support stop_split: true`。
3. 不继续拆 discard response / merge records response 微叶。
4. `src/runtime/response_support.rs` 仍承接 3 个 response DTO。
5. `DiscardRuntimeArtifactResponse` 与 `MergeRecordsResponse` type 仍为 `pub(crate)`，fields 仍为 `pub(super)`。
6. `MergeRecordEntry` type 与 fields 仍为 `pub(super)`。
7. 调用方仍通过 `use super::*`，父级仍是普通 `use response_support::{...};`。
8. `src/runtime/run.rs` 仍是 drained include，`include!("run.rs")` 仍保留。
9. `RunInProgressGuard`、`MAX_EXPERIMENT_VARIANTS`、parent include deletion、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 与 release transition guard 均未处理。
10. 下一步只能进入 BE-001CQ-01 `backend.runtime` 第六轮父叶残余判断。

不得宣称 `backend.runtime` 已完成、run guard/experiment limit 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `292-runtime.response_support单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.response_support` 设置为 `stop_split: true`。
3. 全局递归下一步固定为 BE-001CQ-01 `backend.runtime` 第六轮父叶残余判断。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
