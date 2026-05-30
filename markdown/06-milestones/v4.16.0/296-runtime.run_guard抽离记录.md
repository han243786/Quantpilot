# v4.16.0 runtime.run_guard 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CR-03
> 基准: `295-runtime.run_guard抽离方案.md`、`294-runtime.run_guard单子叶等价基线.md`
> 目标子叶: `runtime.run_guard`
> 模块树坐标: `root.backend.runtime.runtime.run_guard`
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CR-03 `runtime.run_guard` 实际抽离 | 抽离记录 |
| 规范矩阵 | 父子通信、并发 guard、visibility、unit smoke | 落地 |
| 引导矩阵 | `root.backend.runtime.runtime.run_guard` | child created |
| 模块树 | `runtime.run_guard` | 白箱实现更新 |

---

## 实际迁移结果

BE-001CR-03 已创建:

```text
src/runtime/run_guard.rs
```

已迁移 item:

| item | 来源 | 目标 | 结果 |
| --- | --- | --- | --- |
| `RunInProgressGuard` | `src/runtime/mod.rs` | `src/runtime/run_guard.rs` | type `pub(super)`；tuple field `pub(super)` |
| `Drop for RunInProgressGuard` | `src/runtime/mod.rs` | `src/runtime/run_guard.rs` | 继续 `store(false, Ordering::Release)` |

父级 `src/runtime/mod.rs` 只新增:

```rust
mod run_guard;
use run_guard::RunInProgressGuard;
```

该 import 是 plain import，不是 `pub(crate) use`。route facade、schema、frontend caller、runtime persistence owner、storage lifecycle owner 和 `AppState` 均未新增访问面。

---

## child-local unit smoke

`src/runtime/run_guard.rs` 新增最小 unit smoke:

```rust
runtime_run_guard_resets_on_drop
```

该测试只验证 `RunInProgressGuard` drop 后 `AtomicBool` 被复位，不进行 endpoint 并发编排，不改变 public API。

---

## 调用方等价

以下调用方未修改:

- `src/runtime/run/session_start.rs`
- `src/runtime/run/v4_handoff.rs`

仍保持:

1. 输入 owner 仍是 `AppState.run_in_progress`，调用方先执行 `state.run_in_progress.swap(true, std::sync::atomic::Ordering::AcqRel)`。
2. busy 时不构造 guard，并返回各自原有 409 response。
3. 成功进入后才构造 `RunInProgressGuard(&state.run_in_progress)`。
4. 所有成功、错误和 early return 路径继续依赖 Drop `Ordering::Release` 复位。

---

## 明确未处理

- 未迁移 `MAX_EXPERIMENT_VARIANTS`。
- 未删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 未修改 `src/runtime/query_support.rs` 或 `src/runtime/response_support.rs`。
- 未修改 `AppState`、route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 或 release transition guard。
- 未新增 sibling child 横向连接，未启动发布版本过渡。

---

## 验证记录

本批执行并通过:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot runtime_run_guard_resets_on_drop
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
BE-001CR-04 runtime.run_guard 单叶 closeout
```

BE-001CR-04 只判断 `runtime.run_guard` 是否继续细拆。不得跳过 closeout 直接处理 `MAX_EXPERIMENT_VARIANTS`、parent include cleanup 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001CR-03 完成时，必须说明:

1. `src/runtime/run_guard.rs` 已创建。
2. `RunInProgressGuard` 与 `Drop for RunInProgressGuard` 已迁入 child。
3. 父级 `src/runtime/mod.rs` 只保留 `mod run_guard;` 与 plain `use run_guard::RunInProgressGuard;`。
4. 两个调用方仍为 `src/runtime/run/session_start.rs` 与 `src/runtime/run/v4_handoff.rs`，且没有新增 direct child import。
5. `swap(true, Ordering::AcqRel)` 未移入 guard，busy response 未统一。
6. child-local unit smoke `runtime_run_guard_resets_on_drop` 已覆盖 Drop reset。
7. `MAX_EXPERIMENT_VARIANTS`、parent include deletion、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 与 release transition guard 均未处理。
8. 下一步只能进入 BE-001CR-04 单叶 closeout。

不得宣称 experiment limit 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `296-runtime.run_guard抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/run_guard.rs` 进入全量树。
3. 抽离只迁移 run guard，未改变两个调用方 busy response、进入检查或 Drop reset 语义。
4. 治理门禁、全量树覆盖、Rust 等价测试和 `git diff --check` 均通过。
