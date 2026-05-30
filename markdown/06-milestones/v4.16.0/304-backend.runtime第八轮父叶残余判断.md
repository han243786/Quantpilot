# v4.16.0 backend.runtime 第八轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CU-01
> 基准: `303-runtime.experiment_limit单叶closeout.md`、`302-runtime.experiment_limit抽离记录.md`、`13-递归模块化全局根流程.md`
> 父叶: `backend.runtime`
> 判定: `backend.runtime stop_split: false`
> 下一步: BE-001CV-01 `runtime.parent_include_cleanup` 单子叶等价基线
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CU-01 `backend.runtime` 第八轮父叶残余判断 | 父叶残余判断 |
| 规范矩阵 | parent include cleanup、父子通信、drained 文件删除边界、release transition guard | 候选排序 |
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
- `runtime.run_guard stop_split: true`
- `runtime.experiment_limit stop_split: true`

这些 closeout 只证明对应子叶已完成，不等于 `backend.runtime` 父叶完成。

---

## 当前真实残余

`backend.runtime` 仍不能设置 `stop_split: true`。当前父级真实残余已收缩为 drained parent include cleanup:

| residual | 当前位置 | 当前文件状态 | 判定 |
| --- | --- | --- | --- |
| `include!("run.rs")` | `src/runtime/mod.rs` | `src/runtime/run.rs` 只剩 drained include 注释 | cleanup deferred |
| `include!("mutation.rs")` | `src/runtime/mod.rs` | `src/runtime/mutation.rs` 只剩 drained include 注释 | cleanup deferred |
| `include!("backtest.rs")` | `src/runtime/mod.rs` | `src/runtime/backtest.rs` 只剩 drained parent include 注释 | cleanup deferred |

已清除或归位的父级支撑面:

- `src/runtime/query_support.rs` 已承接 Query DTO 与 normalization helper。
- `src/runtime/response_support.rs` 已承接 response DTO。
- `src/runtime/run_guard.rs` 已承接 `RunInProgressGuard` 与 Drop reset。
- `src/runtime/experiment_limit.rs` 已承接 `MAX_EXPERIMENT_VARIANTS = 27`。

---

## 下一候选选择

下一候选固定为:

```text
BE-001CV-01 runtime.parent_include_cleanup 单子叶等价基线
root.backend.runtime.runtime.parent_include_cleanup
```

理由:

1. `src/runtime/mod.rs` 当前只剩 child 声明、plain import、受控 re-export、三条 drained `include!(...)` 和父级必要 `use super::*` / `Query`。
2. `MAX_EXPERIMENT_VARIANTS` 已迁入 `src/runtime/experiment_limit.rs`，不再构成父级业务常量残余。
3. `src/runtime/run.rs`、`src/runtime/mutation.rs` 与 `src/runtime/backtest.rs` 当前均不再承接 handler / helper，只保留 drained 注释。
4. 删除 `include!("run.rs")`、`include!("mutation.rs")`、`include!("backtest.rs")` 与删除 drained 文件属于同一 parent support cleanup，不应混入已 closeout 的 run / mutation / backtest 子叶继续细拆。

---

## 暂不选择项

### reopen closed children

`runtime.run_guard stop_split: true`、`runtime.experiment_limit stop_split: true`、`runtime.query_support stop_split: true`、`runtime.response_support stop_split: true` 已成立。BE-001CV 不得回收这些 closed child，也不得把 parent include cleanup 伪装成这些叶子的内部继续细拆。

### release transition

本批不启动发布版本过渡，不新增 sibling child 横向连接，也不为性能目的绕过父子通信规则。只有开发者明确指出进入发布版本过渡时，后续才允许提出相关连接优化方案。

---

## 明确不变

- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 不删除 `src/runtime/run.rs`、`src/runtime/mutation.rs` 或 `src/runtime/backtest.rs`。
- 不修改 `src/runtime/mod.rs`。
- 不修改 `src/runtime/query_support.rs`、`src/runtime/response_support.rs`、`src/runtime/run_guard.rs` 或 `src/runtime/experiment_limit.rs`。
- 不修改 route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 或 release transition guard。

---

## 验证要求

本批为 `no code movement` 父叶残余判断，提交前仍需执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot runtime_run_guard_resets_on_drop
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_mutation
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001CV-01 runtime.parent_include_cleanup 单子叶等价基线
```

BE-001CV-01 只能冻结 drained parent include cleanup 的输入、输出、删除边界、回退点和硬门禁。不得直接删除 `include!(...)`，不得删除 drained 文件，不得迁移 schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或启动 release transition。

---

## 幻觉检查点

AI 声称 BE-001CU-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `backend.runtime stop_split: false`。
3. `runtime.experiment_limit stop_split: true` 已成立，父级真实残余只剩 drained parent include cleanup。
4. 当前仍保留 `include!("run.rs")`、`include!("mutation.rs")` 与 `include!("backtest.rs")`。
5. 下一候选固定为 BE-001CV-01 `runtime.parent_include_cleanup` 单子叶等价基线。
6. 本批没有删除 drained include、没有删除 drained 文件、没有处理 schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 或 release transition guard。

不得宣称 `backend.runtime` 已完成、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `304-backend.runtime第八轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树保持 `backend.runtime stop_split: false`。
3. 全局递归下一步固定为 BE-001CV-01 `runtime.parent_include_cleanup` 单子叶等价基线。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
