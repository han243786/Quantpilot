# v4.16.0 runtime.run_guard 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CR-02
> 基准: `294-runtime.run_guard单子叶等价基线.md`、`293-backend.runtime第六轮父叶残余判断.md`
> 目标子叶: `runtime.run_guard`
> 模块树坐标: `root.backend.runtime.runtime.run_guard`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CR-02 `runtime.run_guard` 抽离方案 | 抽离方案 |
| 规范矩阵 | 父子通信、并发 guard、plain import、visibility、回退点 | 固定 |
| 引导矩阵 | `root.backend.runtime.runtime.run_guard` | planned child 方案 |
| 模块树 | `runtime.run_guard` | 白箱方案更新 |

---

## test-first 判定

BE-001CR-02 不单独开 test-first 批次。

理由:

1. 本叶只迁移 `RunInProgressGuard` 与 `Drop for RunInProgressGuard`，不改变 route、schema、response body、persistence、`AppState.run_in_progress` owner 或 handler orchestration。
2. 当前进入语义仍由两个调用方执行 `swap(true, Ordering::AcqRel)`；BE-001CR-03 不得把进入检查移入 guard，因此 endpoint 行为不应产生新分支。
3. 专门 endpoint 并发 busy 测试需要人为拉长 run 生命周期，容易引入时间竞态；本批用现有 `api_run` 成功/拒绝路径和 child-local unit smoke 覆盖更稳定。

BE-001CR-03 实际抽离时，允许在新 child 内新增最小 unit smoke，例如验证 guard drop 后 `AtomicBool` 被 `Ordering::Release` 复位；该测试不得扩展为 endpoint 并发编排，不得修改 public API。

---

## 目标文件与父级声明

BE-001CR-03 才允许创建 planned child 文件:

```text
src/runtime/run_guard.rs
```

父级 `src/runtime/mod.rs` 在 BE-001CR-03 只允许新增:

```rust
mod run_guard;
use run_guard::RunInProgressGuard;
```

必须使用 plain `use`。不得使用 `pub(crate) use run_guard::RunInProgressGuard;`，不得把 run guard 升级成 crate public surface。

---

## 允许迁移清单

BE-001CR-03 只允许迁移以下 2 个 item:

| item | 当前文件 | 目标文件 | visibility 方案 |
| --- | --- | --- | --- |
| `RunInProgressGuard` | `src/runtime/mod.rs` | `src/runtime/run_guard.rs` | type `pub(super)`；tuple field `pub(super)` |
| `Drop for RunInProgressGuard` | `src/runtime/mod.rs` | `src/runtime/run_guard.rs` | impl 保持 child 内部 |

目标 child 的代码形状固定为:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) struct RunInProgressGuard<'a>(pub(super) &'a AtomicBool);

impl Drop for RunInProgressGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
```

tuple field 需要 `pub(super)`，因为两个 run child 仍通过父级 controlled surface 构造 `RunInProgressGuard(&state.run_in_progress)`。这不是横向连接；调用方仍不得 direct import `run_guard` child。

---

## 允许修改的调用方

BE-001CR-03 原则上不修改以下调用方:

- `src/runtime/run/session_start.rs`
- `src/runtime/run/v4_handoff.rs`

两个调用方必须继续通过父级 `use super::*` 获得 `RunInProgressGuard`，并继续保留各自的 busy response:

- legacy `start_test_run`: busy 时返回 409 中文提示。
- v4 `start_v4_runtime_run`: busy 时返回 409 `runtime_busy` JSON。

不得把 `swap(true, Ordering::AcqRel)` 移入 guard，不得统一两个 response body，不得把 guard 改成手动 reset helper。

---

## 明确排除

- 不迁移 `MAX_EXPERIMENT_VARIANTS`。
- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 不修改 `src/runtime/query_support.rs` 或 `src/runtime/response_support.rs`。
- 不修改 `AppState`、route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 或 release transition guard。
- 不新增 sibling child 横向连接，不启动发布版本过渡。

---

## 回退点

若 BE-001CR-03 编译或测试失败，回退方式固定为:

1. 删除 `src/runtime/run_guard.rs`。
2. 将 `RunInProgressGuard` 与 `Drop for RunInProgressGuard` 放回 `src/runtime/mod.rs` 原位置。
3. 移除 `mod run_guard;` 与 plain `use run_guard::RunInProgressGuard;`。
4. 保持 `src/runtime/run/session_start.rs` 与 `src/runtime/run/v4_handoff.rs` 的 `swap(true, Ordering::AcqRel)`、busy response 和 guard 构造时机不变。
5. 保持 experiment limit、parent include、query/response support、state owner、schema owner、persistence owner 与 release transition guard 不变。

---

## 验证要求

BE-001CR-02 是 `no code movement` 方案提交，提交前仍需执行:

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

BE-001CR-03 实际抽离后也必须执行同一组命令。

---

## 下一步

下一步只允许进入:

```text
BE-001CR-03 runtime.run_guard 实际抽离
```

BE-001CR-03 才能创建 `src/runtime/run_guard.rs` 并迁移 `RunInProgressGuard` 与 Drop impl。不得跳过 BE-001CR-03 直接做 closeout、experiment limit、parent include cleanup 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001CR-02 完成时，必须说明:

1. 本批次仍是 `no code movement` 抽离方案。
2. 方案选择不单独开 test-first 批次，但允许 BE-001CR-03 在 child 内新增最小 unit smoke。
3. `src/runtime/run_guard.rs` 尚未创建。
4. BE-001CR-03 的迁移清单仅限 `RunInProgressGuard` 与 `Drop for RunInProgressGuard`。
5. 父级只允许 `mod run_guard;` 与 plain `use run_guard::RunInProgressGuard;`。
6. `swap(true, Ordering::AcqRel)` 仍留在 `src/runtime/run/session_start.rs` 与 `src/runtime/run/v4_handoff.rs`。
7. `MAX_EXPERIMENT_VARIANTS`、parent include deletion、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 与 release transition guard 均未处理。
8. 下一步只能进入 BE-001CR-03 实际抽离。

不得宣称 run guard 已迁移、并发语义已改造、experiment limit 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `295-runtime.run_guard抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案明确目标 child、父级声明、plain import、允许迁移清单、visibility、test-first 判定和回退点。
3. 治理门禁能阻止 BE-001CR-03 超范围迁移 experiment limit、parent include cleanup、`AppState` 或 release transition。
4. 治理门禁、全量树覆盖、Rust 等价测试和 `git diff --check` 均通过。
