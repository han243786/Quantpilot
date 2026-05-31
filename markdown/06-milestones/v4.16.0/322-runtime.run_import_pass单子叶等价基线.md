# v4.16.0 runtime.run_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DB-01
> 基准: `321-runtime.parent_import_bridge父叶残余判断.md`
> 目标子叶: `runtime.run_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.run_import_pass`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DB-01 `runtime.run_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | parent import bridge、explicit import pass、run child import、release transition guard | 等价边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.run_import_pass` | run import 白箱基线 |
| 模块树 | `runtime.run_import_pass` | 新残余子叶登记 |

---

## 候选范围

本基线只冻结 `runtime.run_import_pass` 的真实边界，不修改 Rust 代码。候选文件固定为:

```text
src/runtime/run/v4_handoff.rs
src/runtime/run/session_start.rs
src/runtime/run/record_store.rs
src/runtime/run/replay_status.rs
```

父级挂载仍位于 `src/runtime/mod.rs`:

```text
#[path = "run/record_store.rs"]
mod run_record_store;
#[path = "run/replay_status.rs"]
mod run_replay_status;
#[path = "run/session_start.rs"]
mod run_session_start;
#[path = "run/v4_handoff.rs"]
mod run_v4_handoff;
```

---

## 当前 import 事实

4 个候选文件当前都以 `use super::*` 获取父级白箱输入:

| 文件 | 当前职责 | 父级输入面 |
| --- | --- | --- |
| `src/runtime/run/v4_handoff.rs` | `/api/runtime/v4/run` handoff handler、v4 runtime static bundle、simulated capability matrix | `AppState`、`RunInProgressGuard`、`current_time_ms`、`json_bad_request_with_code`、`internal_error`、Axum extractors、Serde / JSON helpers、QRPC / QuantScript types |
| `src/runtime/run/session_start.rs` | legacy `/api/runtime/test-run` handler 与 run session bootstrap | auth user、`AppState`、`FrontendRunRequest`、`RunStartResponse`、capability guard、compile helpers、sandbox/session helpers、runtime event envelope helpers、collaboration/audit helpers |
| `src/runtime/run/record_store.rs` | run list/detail/save/discard handlers | pagination、run persistence、audit persistence、path sanitization、`DiscardRuntimeArtifactResponse`、`RunDetailResponse`、`RunListItem` |
| `src/runtime/run/replay_status.rs` | run replay/status handlers | `RuntimeReplayQuery`、replay normalization、run record loading、evidence metrics、replay/status response builders |

这些输入目前都来自既有 `src/runtime/mod.rs` 受控父级 surface 或 crate-level owner。BE-001DB-01 不改变任何 owner，只冻结后续显式 import 的依赖清单。

---

## 等价边界

后续实际抽离只允许把 4 个文件顶部的 `use super::*` 收敛为显式 import 或必要的父级限定输入，必须保持:

1. route path 与 handler function name 不变。
2. `pub(crate) use run_*::{...}` 对外 facade 不变。
3. run mutex / `RunInProgressGuard` 行为不变。
4. runtime record list/detail/save/discard schema 不变。
5. replay/status response schema 与 evidence metrics 写入不变。
6. legacy run 与 v4 handoff 的 error code、status code、JSON envelope 不变。
7. 不迁移 `AppState`、auth、run persistence、graph audit、capability guard、compile/runtime protocol、sandbox/session 或 frontend caller owner。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不处理 `src/runtime/mod.rs` 的 root parent bridge。
- 本批不处理 `src/runtime/backtest/**`、`src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 本批不拆分 run handler 行为体。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 基线，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续实际 import rewrite 至少补跑:

```powershell
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_sse
```

---

## 下一步

下一步只能进入:

```text
BE-001DB-02 runtime.run_import_pass 抽离方案
```

BE-001DB-02 只能设计 4 个 run child 的显式 import 改写范围、验证门禁和回退点；不得直接执行 Rust import rewrite。

---

## 幻觉检查点

AI 声称 BE-001DB-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. 候选范围只有 4 个 `src/runtime/run/**` child。
3. `src/runtime/mod.rs` root parent bridge 尚未处理。
4. 下一步只能进入 BE-001DB-02 `runtime.run_import_pass` 抽离方案。
5. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.run_import_pass` 已完成、parent import bridge 已消除、`backend.runtime` 已完成或 Rust 重构已完成。

---

## 验收标准

1. `322-runtime.run_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线冻结 4 个 run child 的真实 import 依赖面。
3. 下一步固定为 BE-001DB-02 `runtime.run_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
