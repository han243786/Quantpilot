# v4.16.0 backend.runtime 第五轮父叶残余判断

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CO-01  
> 基准: `287-runtime.query_support单叶closeout.md`、`286-runtime.query_support抽离记录.md`、`13-递归模块化全局根流程.md`  
> 父叶: `backend.runtime`  
> 判定: `backend.runtime stop_split: false`  
> 下一步: BE-001CP-01 `runtime.response_support` 单子叶等价基线  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CO-01 `backend.runtime` 第五轮父叶残余判断 | 父叶残余判断 |
| 规范矩阵 | 父子通信、response DTO 边界、run guard、experiment limit、parent include、release transition guard | 候选排序 |
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

这些 closeout 只证明对应子叶已完成，不等于 `backend.runtime` 父叶完成。

其中 `runtime.query_support` 的真实 child 文件为 `src/runtime/query_support.rs`，本批只引用其 closeout 证据，不修改该 child。

---

## 当前真实残余

`backend.runtime` 仍不能设置 `stop_split: true`。当前父级 / parent support 残余如下:

| residual | 当前位置 | 调用方 | 判定 |
| --- | --- | --- | --- |
| `DiscardRuntimeArtifactResponse` | `src/runtime/mod.rs` | `src/runtime/run/record_store.rs`、`src/runtime/backtest/record_store.rs`、`src/runtime/backtest/record_lifecycle.rs` | response DTO residual |
| `MergeRecordsResponse` | `src/runtime/run.rs` | `src/runtime/report_ops/merge_generation_health.rs` | response DTO residual |
| `MergeRecordEntry` | `src/runtime/run.rs` | `src/runtime/report_ops/merge_generation_health.rs` | response DTO residual |
| `RunInProgressGuard` | `src/runtime/mod.rs` | `src/runtime/run/session_start.rs`、`src/runtime/run/v4_handoff.rs` | run guard residual |
| `MAX_EXPERIMENT_VARIANTS` | `src/runtime/mod.rs` | `src/runtime/backtest/parameter_grid.rs` | experiment limit residual |
| `include!("run.rs")` | `src/runtime/mod.rs` | parent include support | still needed until response DTO residual is moved |
| `include!("mutation.rs")` | `src/runtime/mod.rs` | drained parent include | cleanup deferred |
| `include!("backtest.rs")` | `src/runtime/mod.rs` | drained parent include | cleanup deferred |

---

## 下一候选选择

下一候选固定为:

```text
BE-001CP-01 runtime.response_support 单子叶等价基线
root.backend.runtime.runtime.response_support
```

理由:

1. response support 同时覆盖 `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse` 与 `MergeRecordEntry`，是当前最明确的父级 DTO 白箱残余。
2. `MergeRecordsResponse` 与 `MergeRecordEntry` 当前仍留在 `src/runtime/run.rs`，迁移 response support 能进一步把 `run.rs` 降为 drained include 或为后续 parent include cleanup 铺路。
3. response DTO 不拥有状态、锁顺序、持久化 owner、schema owner、frontend caller、runtime persistence owner 或 release transition guard，适合作为标准档单子叶基线。
4. 相比先动 `RunInProgressGuard`，response support 的迁移面更广，能削薄父级和 `run.rs` 双重残余。

---

## 暂不选择项

### `runtime.run_guard`

`RunInProgressGuard` 仍值得后续处理，但当前只覆盖一个 RAII guard 和两处调用。它涉及 `AppState.run_in_progress` 的 AcqRel / Release 语义，后续应单独冻结等价基线，不能混入 response DTO。

### `runtime.experiment_limit`

`MAX_EXPERIMENT_VARIANTS` 当前只服务 `src/runtime/backtest/parameter_grid.rs`。它更像 experiment sweep / parameter_grid 的常量 owner 回收，适合在 response support 与 run guard 后另起父叶判断，不应混入 response support。

### parent include cleanup

`include!("mutation.rs")` 与 `include!("backtest.rs")` 当前已是 drained include；`include!("run.rs")` 仍持有 merge response DTO。parent include cleanup 必须等 response support / run guard / experiment limit 边界继续削薄后再统一判断，不能在本批直接删除。

---

## 明确不变

- 不创建 `src/runtime/response_support.rs`。
- 不迁移 `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse` 或 `MergeRecordEntry`。
- 不迁移 `RunInProgressGuard`。
- 不迁移 `MAX_EXPERIMENT_VARIANTS`。
- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
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
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001CP-01 runtime.response_support 单子叶等价基线
```

BE-001CP-01 只能冻结 response DTO 的输入、输出、调用方、visibility、父级受控 surface 与硬门禁。不得直接创建 `src/runtime/response_support.rs`，不得迁移 DTO，不得处理 run guard、experiment limit、parent include deletion 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CO-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `backend.runtime stop_split: false`。
3. `runtime.query_support stop_split: true` 已成立，但父级仍有 response support / run guard / experiment limit / parent include residual。
4. 下一候选固定为 BE-001CP-01 `runtime.response_support` 单子叶等价基线。
5. 本批没有创建 `src/runtime/response_support.rs`，没有迁移 response DTO，没有处理 run guard、experiment limit、parent include deletion、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

不得宣称 `backend.runtime` 已完成、response DTO 已抽离、run guard 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `288-backend.runtime第五轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树保持 `backend.runtime stop_split: false`。
3. 全局递归下一步固定为 BE-001CP-01 `runtime.response_support` 单子叶等价基线。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
