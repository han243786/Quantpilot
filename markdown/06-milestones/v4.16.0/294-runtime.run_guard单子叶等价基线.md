# v4.16.0 runtime.run_guard 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CR-01
> 基准: `293-backend.runtime第六轮父叶残余判断.md`、`292-runtime.response_support单叶closeout.md`
> 目标子叶: `runtime.run_guard`
> 模块树坐标: `root.backend.runtime.runtime.run_guard`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CR-01 `runtime.run_guard` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | 父子通信、并发 guard、visibility、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.run_guard` | planned child |
| 模块树 | `runtime.run_guard` | 白箱基线登记 |

---

## 当前真实边界

目标 item 当前仍在 `src/runtime/mod.rs`:

```rust
struct RunInProgressGuard<'a>(&'a std::sync::atomic::AtomicBool);
impl Drop for RunInProgressGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
    }
}
```

两个调用方当前均在 run child 中通过父级 `use super::*` 访问该 guard:

- `src/runtime/run/session_start.rs`
- `src/runtime/run/v4_handoff.rs`

planned child 文件尚未创建:

```text
src/runtime/run_guard.rs
```

---

## 输入 / 输出 / 处理方

| 项 | 内容 | 约束 |
| --- | --- | --- |
| 输入 | `&state.run_in_progress` / `&AtomicBool` | 不改变 `AppState.run_in_progress` owner |
| 预进入检查 | 调用方先执行 `swap(true, Ordering::AcqRel)` | 已经忙碌时必须保留 409 conflict |
| guard 构造 | `RunInProgressGuard(&state.run_in_progress)` | 只在成功进入运行后构造 |
| Drop 输出 | `store(false, Ordering::Release)` | 运行路径、错误路径和 early return 均必须复位 |
| 调用方 | `start_test_run`、`start_v4_runtime_run` | 不改变 handler response contract |

---

## 关键 public 方法 / 类型

| 方法 / 类型 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `RunInProgressGuard` | `&AtomicBool` | RAII guard | `runtime.run.session_start`、`runtime.run.v4_handoff` | 不得改变 constructor 时机、Drop 复位或 ordering |
| `Drop for RunInProgressGuard` | guard lifetime end | `run_in_progress = false` | Rust drop path | 不得改成手动复位、不得移除 Release ordering |

`RunInProgressGuard` 不是 HTTP response schema，不是 route facade，不是 runtime persistence owner。它只负责 run start 并发生命周期的最小复位语义。

---

## 调用方等价

`start_test_run` 当前流程:

```text
swap(true, Ordering::AcqRel)
  -> true: 409 "已有运行在进行中..."
  -> false: construct RunInProgressGuard
  -> capability/config/compile/sandbox/run/persist
  -> Drop resets run_in_progress with Ordering::Release
```

`start_v4_runtime_run` 当前流程:

```text
swap(true, Ordering::AcqRel)
  -> true: 409 runtime_busy JSON
  -> false: construct RunInProgressGuard
  -> resolve graph / run v4 runtime / build response
  -> Drop resets run_in_progress with Ordering::Release
```

BE-001CR-02/03 若进入抽离，调用方文件不得新增 sibling child 横向 direct import。仍必须通过 `src/runtime/mod.rs` controlled run guard surface。

---

## 当前测试证据

当前等价证据来自:

- `cargo test -p quantpilot --test api_run`
- `cargo test -p quantpilot --test api_backtest`
- `cargo test -p quantpilot --test api_mutation`
- `cargo test -p quantpilot --test api_ai_proposal`
- `cargo test -p quantpilot --test api_v1_reports`
- `cargo test -p quantpilot --test api_v1_ops_health`

其中 `api_run` 覆盖 legacy run 与 v4 run 的成功/拒绝路径；`runtime_write_rejects_missing_capability_without_creating_run` 覆盖 legacy run 在 guard 后进入错误路径时不创建 run record。当前没有新增专门的并发冲突测试；BE-001CR-02 若认为需要，可先提出 test-first 方案，但不得在本基线直接新增测试或移动代码。

---

## 明确排除

- 不创建 `src/runtime/run_guard.rs`。
- 不迁移 `RunInProgressGuard`。
- 不修改 `src/runtime/run/session_start.rs` 或 `src/runtime/run/v4_handoff.rs`。
- 不迁移 `MAX_EXPERIMENT_VARIANTS`。
- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 不修改 `src/runtime/query_support.rs` 或 `src/runtime/response_support.rs`。
- 不修改 `AppState`、route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 或 release transition guard。

---

## 验证要求

本批为 `no code movement` 基线，提交前仍需执行:

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
BE-001CR-02 runtime.run_guard 抽离方案
```

BE-001CR-02 必须先决定是否 test-first，再固定 `src/runtime/run_guard.rs`、父级声明、plain import、visibility 与回退点。不得直接进入实际抽离，不得处理 experiment limit、parent include cleanup 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CR-01 完成时，必须说明:

1. 本批次是 `no code movement` 等价基线。
2. `src/runtime/run_guard.rs` 尚未创建。
3. `RunInProgressGuard` 仍在 `src/runtime/mod.rs`。
4. 两个调用方仍为 `src/runtime/run/session_start.rs` 与 `src/runtime/run/v4_handoff.rs`。
5. 当前语义为调用方 `swap(true, Ordering::AcqRel)` 进入，guard Drop `store(false, Ordering::Release)` 复位。
6. `MAX_EXPERIMENT_VARIANTS`、parent include deletion、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 与 release transition guard 均未处理。
7. 下一步只能进入 BE-001CR-02 抽离方案。

不得宣称 run guard 已抽离、并发语义已改造、experiment limit 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `294-runtime.run_guard单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树登记 `root.backend.runtime.runtime.run_guard` planned child 基线。
3. 全局递归下一步固定为 BE-001CR-02 `runtime.run_guard` 抽离方案。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
