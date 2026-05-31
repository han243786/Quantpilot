# v4.16.0 runtime.root_support_import_pilot 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CX-03
> 基准: `310-runtime.parent_import_bridge抽离方案.md`
> 目标子叶: `runtime.root_support_import_pilot`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.root_support_import_pilot`
> 代码动作: actual Rust import rewrite

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CX-03 `runtime.root_support_import_pilot` 实际抽离 | 实际抽离 |
| 规范矩阵 | explicit import pass、minimum batch、parent import bridge、release transition guard | 等价收敛 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.root_support_import_pilot` | 白箱 import 收敛 |
| 模块树 | `runtime.root_support_import_pilot` | 抽离记录 |

---

## 实际改动

本批只执行 BE-001CX-02 指定的第一批 root support pilot:

```text
src/runtime/query_support.rs
src/runtime/response_support.rs
```

### `src/runtime/query_support.rs`

删除 parent wildcard:

```rust
use super::*;
```

改为显式 import:

```rust
use crate::{
    RuntimeAiProposalStatus, RuntimeEvidenceSourceKind, RuntimeReplayFilters, RuntimeReplayOptions,
};
use serde::Deserialize;
```

### `src/runtime/response_support.rs`

删除 parent wildcard:

```rust
use super::*;
```

改为显式 import:

```rust
use serde::Serialize;
```

---

## 等价结果

- `query_support` 仍只拥有 RuntimeReplayQuery、RuntimeParameterMutationListQuery、RuntimeAiProposalListQuery、RuntimeApprovalListQuery、OpsDailyQuery、AuditWeeklyQuery、ResearchMonthlyQuery 和 replay query normalization helper。
- `response_support` 仍只拥有 DiscardRuntimeArtifactResponse、MergeRecordsResponse、MergeRecordEntry。
- handler owner、route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、AppState、lock order 均未变更。
- `src/runtime/mod.rs` 父桥未删除，仍需后续 staged explicit import pass。
- 未新增 sibling horizontal link，未启动 release transition。

当前 `src/runtime/**.rs` 中存在 `use super::*` 或 `super::` 依赖的文件数从 46 降为 44。

---

## 排除项

- 不处理 `event_stream`、`evidence_health`、`report_ops`、`run`、`backtest` 或 `mutation`。
- 不删除 `src/runtime/mod.rs` 的 `use super::*`。
- 不改变 `pub(crate) use` route-facing surface。
- 不改变 response schema 字段、serde 命名、pagination、filter normalization 或 replay options 行为。
- 不启动 release transition，不引入性能旁路。

---

## 验证要求

本批实际 Rust import rewrite 提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CX-04 runtime.root_support_import_pilot 单叶 closeout
```

BE-001CX-04 需要判断 root support pilot 是否等价完成、是否值得继续拆分，以及下一批 explicit import pass 应进入 `runtime.root_entry_import_pass` 还是先补更细子叶。

---

## 幻觉检查点

AI 声称 BE-001CX-03 完成时，必须说明:

1. 本批只改写 `query_support` 与 `response_support` 两个文件的 parent wildcard import。
2. 当前 parent import bridge 尚未消除，`src/runtime/mod.rs` 未处理。
3. `use super::*` / `super::` 依赖文件数只从 46 降为 44。
4. 下一步只能进入 BE-001CX-04 单叶 closeout。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成、parent import bridge 已完全清除或 release transition 已启动。

---

## 验收标准

1. 两个目标文件不再通过 `use super::*` 获取父级白箱输入。
2. `cargo check -p quantpilot` 与指定 API 测试通过。
3. `311-runtime.root_support_import_pilot抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
4. 下一步固定为 BE-001CX-04 `runtime.root_support_import_pilot` 单叶 closeout。
