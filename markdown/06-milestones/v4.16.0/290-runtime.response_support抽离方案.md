# v4.16.0 runtime.response_support 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CP-02  
> 基准: `289-runtime.response_support单子叶等价基线.md`、`288-backend.runtime第五轮父叶残余判断.md`  
> 目标子叶: `runtime.response_support`  
> 模块树坐标: `root.backend.runtime.runtime.response_support`  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CP-02 `runtime.response_support` 抽离方案 | 抽离方案 |
| 规范矩阵 | 父子通信、response DTO visibility、plain import、回退点 | 固定 |
| 引导矩阵 | `root.backend.runtime.runtime.response_support` | planned child 方案 |
| 模块树 | `runtime.response_support` | 白箱方案更新 |

---

## 目标文件与父级声明

BE-001CP-03 才允许创建 planned child 文件:

```text
src/runtime/response_support.rs
```

父级 `src/runtime/mod.rs` 在 BE-001CP-03 只允许新增:

```rust
mod response_support;
use response_support::{
    DiscardRuntimeArtifactResponse, MergeRecordEntry, MergeRecordsResponse,
};
```

必须使用 plain `use`。不得使用 `pub(crate) use response_support::{...};`，不得把 response DTO 升级成新的 crate public surface。

---

## 允许迁移清单

BE-001CP-03 只允许迁移以下 3 个 item:

| item | 当前文件 | 目标文件 | visibility 方案 |
| --- | --- | --- | --- |
| `DiscardRuntimeArtifactResponse` | `src/runtime/mod.rs` | `src/runtime/response_support.rs` | type `pub(crate)`；fields `pub(super)` |
| `MergeRecordsResponse` | `src/runtime/run.rs` | `src/runtime/response_support.rs` | type `pub(crate)`；fields `pub(super)` |
| `MergeRecordEntry` | `src/runtime/run.rs` | `src/runtime/response_support.rs` | type 优先 `pub(super)`；fields `pub(super)` |

若编译器要求 `MergeRecordEntry` 因 `MergeRecordsResponse` field exposure 提升到 `pub(crate)`，BE-001CP-03 必须在抽离记录中说明原因，并继续保持 fields `pub(super)`，不得升级成外部 public API。

---

## 允许修改的调用方

BE-001CP-03 允许保持以下调用方逻辑不变，只通过父级 controlled response surface 获得 DTO:

- `src/runtime/run/record_store.rs`
- `src/runtime/backtest/record_store.rs`
- `src/runtime/backtest/record_lifecycle.rs`
- `src/runtime/report_ops/merge_generation_health.rs`

这些文件不得新增横向 direct child import。既有 `use super::*` 可保持。

---

## run.rs drained include 规则

`src/runtime/run.rs` 当前只剩 `MergeRecordsResponse` 与 `MergeRecordEntry`。BE-001CP-03 迁移后，允许把 `src/runtime/run.rs` 降为 drained include 注释:

```rust
// Drained include retained until the runtime parent support closeout.
```

不得在 BE-001CP-03 删除 `include!("run.rs")`。parent include cleanup 必须等 response support、run guard、experiment limit 等父级残余继续收敛后，另起父叶残余判断。

---

## 明确排除

- 不迁移 `RunInProgressGuard`。
- 不迁移 `MAX_EXPERIMENT_VARIANTS`。
- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 不修改 `src/runtime/query_support.rs`。
- 不修改 response schema、schema owner、route facade、runtime persistence owner、storage lifecycle owner、frontend caller、`AppState`、lock order 或 release transition guard。
- 不新增 sibling child 横向连接，不启动发布版本过渡。

---

## 回退点

若 BE-001CP-03 编译或测试失败，回退方式固定为:

1. 删除 `src/runtime/response_support.rs`。
2. 将 `DiscardRuntimeArtifactResponse` 放回 `src/runtime/mod.rs` 原位置。
3. 将 `MergeRecordsResponse` 与 `MergeRecordEntry` 放回 `src/runtime/run.rs` 原位置。
4. 移除 `mod response_support;` 与 plain `use response_support::{...};`。
5. 保持其他 runtime child、route facade、schema、state owner 与 persistence owner 不变。

---

## 验证要求

BE-001CP-02 是 `no code movement` 方案提交，提交前仍需执行:

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

BE-001CP-03 实际抽离后也必须执行同一组命令。

---

## 下一步

下一步只允许进入:

```text
BE-001CP-03 runtime.response_support 实际抽离
```

BE-001CP-03 才能创建 `src/runtime/response_support.rs` 并迁移 3 个 response DTO。不得跳过 BE-001CP-03 直接做 closeout、parent include cleanup、run guard、experiment limit 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001CP-02 完成时，必须说明:

1. 本批次仍是 `no code movement` 抽离方案。
2. `src/runtime/response_support.rs` 尚未创建。
3. BE-001CP-03 的迁移清单仅限 `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`。
4. 父级只允许 `mod response_support;` 与 plain `use response_support::{...};`。
5. `src/runtime/run.rs` 迁移后只能降为 drained include 注释，不能删除 `include!("run.rs")`。
6. 下一步只能进入 BE-001CP-03 实际抽离。

不得宣称 response DTO 已迁移、`backend.runtime` 已完成、parent include 已删除、run guard 已处理、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `290-runtime.response_support抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案明确目标 child、父级声明、plain import、允许迁移清单、visibility 和回退点。
3. 治理门禁能阻止 BE-001CP-03 超范围迁移 run guard、experiment limit、parent include cleanup 或 release transition。
4. 治理门禁、全量树覆盖、Rust 等价测试和 `git diff --check` 均通过。
