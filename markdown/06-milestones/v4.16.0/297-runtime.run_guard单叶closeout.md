# v4.16.0 runtime.run_guard 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CR-04
> 基准: `296-runtime.run_guard抽离记录.md`、`295-runtime.run_guard抽离方案.md`
> 目标子叶: `runtime.run_guard`
> 模块树坐标: `root.backend.runtime.runtime.run_guard`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CR-04 `runtime.run_guard` 单叶 closeout | 单叶 closeout |
| 规范矩阵 | 父子通信、并发 guard、stop_split、后续父叶残余判断 | 收束 |
| 引导矩阵 | `root.backend.runtime.runtime.run_guard` | child closeout |
| 模块树 | `runtime.run_guard` | 白箱 closeout |

---

## closeout 判定

`runtime.run_guard stop_split: true`

理由:

1. 本叶只承接 `RunInProgressGuard` 与 Drop `store(false, Ordering::Release)` 复位语义，职责足够单一。
2. `swap(true, Ordering::AcqRel)`、busy response 与 handler orchestration 仍由 `runtime.run.session_start` 和 `runtime.run.v4_handoff` 拥有，不属于本叶内部可继续细拆的 owner。
3. `AppState.run_in_progress` owner 未迁移，继续拆成 `enter_check` / `drop_reset` / `unit_smoke` 微叶只会扩大父子接线面。
4. child-local unit smoke `runtime_run_guard_resets_on_drop` 已覆盖 Drop reset；再拆测试或 helper 不会产生新的稳定白箱。

---

## 当前白箱边界

| 项 | 当前 owner | 状态 |
| --- | --- | --- |
| `RunInProgressGuard` | `src/runtime/run_guard.rs` | closeout；输入为 `AtomicBool` 引用 |
| `Drop for RunInProgressGuard` | `src/runtime/run_guard.rs` | closeout |
| `runtime_run_guard_resets_on_drop` | `src/runtime/run_guard.rs` | child-local unit smoke |
| 父级 surface | `src/runtime/mod.rs` | `mod run_guard;` + plain `use run_guard::RunInProgressGuard;` |
| 调用方 | `src/runtime/run/session_start.rs`、`src/runtime/run/v4_handoff.rs` | 仍通过 `use super::*` |

---

## 明确未处理

- 未迁移 `MAX_EXPERIMENT_VARIANTS`。
- 未删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 未修改 `src/runtime/query_support.rs` 或 `src/runtime/response_support.rs`。
- 未修改 `AppState`、route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 或 release transition guard。
- 未新增 sibling child 横向连接，未启动发布版本过渡。

---

## 验证要求

本批为 `no code movement` closeout，提交前仍需执行:

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
BE-001CS-01 backend.runtime 第七轮父叶残余判断
```

BE-001CS-01 必须重新统计 `backend.runtime` 父级残余。当前已知残余包括 `MAX_EXPERIMENT_VARIANTS` 与 drained parent include cleanup；不得从 `runtime.run_guard` 继续细拆。

---

## 幻觉检查点

AI 声称 BE-001CR-04 完成时，必须说明:

1. 本批次是 `no code movement` closeout。
2. `runtime.run_guard stop_split: true`。
3. `src/runtime/run_guard.rs` 继续承接 `RunInProgressGuard`、Drop impl 和 child-local unit smoke。
4. 两个调用方仍通过父级受控 surface 访问 guard，没有 direct child import。
5. `MAX_EXPERIMENT_VARIANTS`、parent include deletion、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 与 release transition guard 均未处理。
6. 下一步只能进入 BE-001CS-01 `backend.runtime` 第七轮父叶残余判断。

不得宣称 `backend.runtime` 已完成、experiment limit 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `297-runtime.run_guard单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树登记 `runtime.run_guard stop_split: true`。
3. 全局递归下一步固定为 BE-001CS-01 `backend.runtime` 第七轮父叶残余判断。
4. 治理门禁、全量树覆盖、Rust 等价测试和 `git diff --check` 均通过。
