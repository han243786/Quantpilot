# v4.16.0 runtime.run_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DB-02
> 基准: `322-runtime.run_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.run_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.run_import_pass`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DB-02 `runtime.run_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | explicit import pass、run child import、parent surface、release transition guard | 执行边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.run_import_pass` | run import plan |
| 模块树 | `runtime.run_import_pass` | 方案登记 |

---

## 方案判定

BE-001DB-03 采用同批 explicit import rewrite，处理 4 个 run child:

```text
src/runtime/run/v4_handoff.rs
src/runtime/run/session_start.rs
src/runtime/run/record_store.rs
src/runtime/run/replay_status.rs
```

这 4 个文件属于同一 run child import pocket，功能边界已经在前序递归中完成抽离。当前只移除 `use super::*`，不移动 handler、type、helper 或父级挂载。

---

## 允许动作

BE-001DB-03 只允许:

1. 将 4 个目标文件顶部的 `use super::*` 改为显式 import。
2. 优先从 crate-level owner 引入 `auth`、`AppState`、run persistence helper、runtime response builder、compile/runtime protocol helper、capability helper、graph audit helper、JSON helper 和 error helper。
3. 对仍属于父级白箱 surface 的局部符号使用显式 parent import，例如 `super::RunInProgressGuard`、`super::DiscardRuntimeArtifactResponse` 或 `super::RuntimeReplayQuery`。
4. 保留 `src/runtime/mod.rs` 中既有 `#[path]` child mount 与 `pub(crate) use run_*::{...}` facade。
5. 更新 `runtime.run_import_pass` 抽离记录、模块树、全量树与治理门禁。

---

## 禁止动作

BE-001DB-03 不允许:

1. 不修改 `src/runtime/mod.rs` 的 root parent bridge。
2. 不处理 `src/runtime/backtest/**`、`src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
3. 不改 run route path、handler name、visibility、response schema、status code 或 error code。
4. 不迁移 `RunInProgressGuard`、auth、`AppState`、run persistence、graph audit、capability guard、compile/runtime protocol、sandbox/session、evidence metrics 或 frontend caller owner。
5. 不新增 sibling horizontal link。
6. 不启动 release transition。

---

## 等价验收点

BE-001DB-03 完成后必须确认:

1. 4 个目标文件不再通过 `use super::*` 获取父级白箱输入。
2. `start_v4_runtime_run`、`start_test_run`、run list/detail/save/discard、run replay/status 行为等价。
3. `RunInProgressGuard` 的 run mutex 行为等价。
4. `RunStartResponse`、`RunDetailResponse`、`RunListItem`、`DiscardRuntimeArtifactResponse`、`RuntimeReplayResponse` 与 `RunStatusResponse` schema 等价。
5. runtime parent bridge 依赖文件数只允许从 38 降到 34，不得宣称 parent import bridge 完全消除。

---

## 回退点

若 BE-001DB-03 失败，回退范围仅限:

```text
src/runtime/run/v4_handoff.rs
src/runtime/run/session_start.rs
src/runtime/run/record_store.rs
src/runtime/run/replay_status.rs
markdown/06-milestones/v4.16.0/324-runtime.run_import_pass抽离记录.md
```

不得回退 BE-001DA-01、BE-001DB-01 或已经完成的 report_ops/root_entry/root_support import pass。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001DB-03 实际 import rewrite 后必须执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_sse
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001DB-03 runtime.run_import_pass 实际抽离
```

BE-001DB-03 只允许改写 4 个 run child 的 import，不得顺手处理 root parent bridge、backtest/mutation 子树或 release transition。

---

## 幻觉检查点

AI 声称 BE-001DB-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. 下一步 BE-001DB-03 只能处理 4 个 run child。
3. `src/runtime/mod.rs` root parent bridge 尚未处理。
4. runtime parent bridge 当前仍有 38 个依赖文件。
5. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.run_import_pass` 已完成、parent import bridge 已消除、`backend.runtime` 已完成或 Rust 重构已完成。

---

## 验收标准

1. `323-runtime.run_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定 BE-001DB-03 只处理 4 个 run child。
3. 下一步固定为 BE-001DB-03 `runtime.run_import_pass` 实际抽离。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
