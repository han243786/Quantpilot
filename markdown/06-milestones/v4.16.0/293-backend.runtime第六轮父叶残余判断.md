# v4.16.0 backend.runtime 第六轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CQ-01
> 基准: `292-runtime.response_support单叶closeout.md`、`291-runtime.response_support抽离记录.md`、`13-递归模块化全局根流程.md`
> 父叶: `backend.runtime`
> 判定: `backend.runtime stop_split: false`
> 下一步: BE-001CR-01 `runtime.run_guard` 单子叶等价基线
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CQ-01 `backend.runtime` 第六轮父叶残余判断 | 父叶残余判断 |
| 规范矩阵 | 父子通信、run guard、experiment limit、parent include、release transition guard | 候选排序 |
| 引导矩阵 | `root.backend.runtime` | 父叶队列更新 |
| 模块树 | `backend.runtime` | `stop_split: false` |

---

## 当前已 closeout 子叶

以下 runtime 子叶已完成当前递归范围内 closeout:

- `backend.runtime.routes stop_split: true`
- `runtime.report_ops stop_split: true`
- `runtime.evidence_health stop_split: true`
- `runtime.backtest stop_split: true`
- `runtime.mutation.parameter_mutation stop_split: true`
- `runtime.mutation.ai_proposal stop_split: true`
- `runtime.mutation.shared_governance stop_split: true`
- `runtime.query_support stop_split: true`
- `runtime.response_support stop_split: true`

这些 closeout 只证明对应子叶已完成，不等于 `backend.runtime` 父叶完成。

---

## 当前真实残余

`backend.runtime` 仍不能设置 `stop_split: true`。当前父级 / parent support 残余如下:

| residual | 当前位置 | 调用方 | 判定 |
| --- | --- | --- | --- |
| `RunInProgressGuard` | `src/runtime/mod.rs` | `src/runtime/run/session_start.rs`、`src/runtime/run/v4_handoff.rs` | run guard residual |
| `MAX_EXPERIMENT_VARIANTS` | `src/runtime/mod.rs` | `src/runtime/backtest/parameter_grid.rs` | experiment limit residual |
| `include!("run.rs")` | `src/runtime/mod.rs` | drained parent include | cleanup deferred |
| `include!("mutation.rs")` | `src/runtime/mod.rs` | drained parent include | cleanup deferred |
| `include!("backtest.rs")` | `src/runtime/mod.rs` | drained parent include | cleanup deferred |

已清除的父级支撑面:

- `runtime.query_support` 已承接 Query DTO 与 normalization helper。
- `runtime.response_support` 已承接 response DTO。
- `src/runtime/run.rs`、`src/runtime/mutation.rs` 与 `src/runtime/backtest.rs` 当前均为 drained include 注释文件。

---

## 下一候选选择

下一候选固定为:

```text
BE-001CR-01 runtime.run_guard 单子叶等价基线
root.backend.runtime.runtime.run_guard
```

理由:

1. `RunInProgressGuard` 仍在 `src/runtime/mod.rs`，但真实调用方是 `runtime.run.session_start` 与 `runtime.run.v4_handoff` 两个 run child。
2. 该 guard 承接 `AppState.run_in_progress` 的 run start CAS 与 drop-time Release reset 等价，属于并发生命周期支撑面，不能和 experiment limit 或 parent include cleanup 混处理。
3. 相比先动 `MAX_EXPERIMENT_VARIANTS`，run guard 更像跨 run child 的稳定白箱；先冻结它能让后续 parent include cleanup 的风险更小。
4. `runtime.run_guard` 预计只需要父级受控 child surface，不允许 sibling child 横向直连。

---

## 暂不选择项

### `runtime.experiment_limit`

`MAX_EXPERIMENT_VARIANTS` 当前只服务 `src/runtime/backtest/parameter_grid.rs`。它更像 experiment sweep / parameter_grid 的常量 owner 回收，适合在 run guard closeout 后另起父叶判断。不得混入 run guard。

### parent include cleanup

`src/runtime/run.rs`、`src/runtime/mutation.rs` 与 `src/runtime/backtest.rs` 当前均为 drained include 注释文件，但删除 `include!(...)` 与删除 drained 文件属于 parent support cleanup。必须等 run guard 与 experiment limit 边界继续削薄后再统一判断，不能在本批直接删除。

---

## 明确不变

- 不创建 `src/runtime/run_guard.rs`。
- 不迁移 `RunInProgressGuard`。
- 不迁移 `MAX_EXPERIMENT_VARIANTS`。
- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 不修改 `src/runtime/query_support.rs` 或 `src/runtime/response_support.rs`。
- 不修改 response schema、route facade、runtime persistence owner、storage lifecycle owner、frontend caller、`AppState`、lock order 或 release transition guard。

---

## 验证要求

本批为 `no code movement` 父叶残余判断，提交前仍需执行:

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
BE-001CR-01 runtime.run_guard 单子叶等价基线
```

BE-001CR-01 只能冻结 run guard 的输入、输出、CAS / Release semantics、调用方、visibility、父级受控 surface 与硬门禁。不得直接创建 `src/runtime/run_guard.rs`，不得迁移 guard，不得处理 experiment limit、parent include deletion 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CQ-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `backend.runtime stop_split: false`。
3. `runtime.response_support stop_split: true` 已成立，但父级仍有 run guard / experiment limit / parent include residual。
4. 下一候选固定为 BE-001CR-01 `runtime.run_guard` 单子叶等价基线。
5. 本批没有创建 `src/runtime/run_guard.rs`，没有迁移 `RunInProgressGuard`，没有处理 `MAX_EXPERIMENT_VARIANTS`、parent include deletion、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

不得宣称 `backend.runtime` 已完成、run guard 已抽离、experiment limit 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `293-backend.runtime第六轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树保持 `backend.runtime stop_split: false`。
3. 全局递归下一步固定为 BE-001CR-01 `runtime.run_guard` 单子叶等价基线。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
