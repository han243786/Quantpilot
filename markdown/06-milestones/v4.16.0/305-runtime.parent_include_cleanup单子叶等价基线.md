# v4.16.0 runtime.parent_include_cleanup 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CV-01
> 基准: `304-backend.runtime第八轮父叶残余判断.md`、`13-递归模块化全局根流程.md`
> 目标子叶: `runtime.parent_include_cleanup`
> 模块树坐标: `root.backend.runtime.runtime.parent_include_cleanup`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CV-01 `runtime.parent_include_cleanup` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | drained include 删除边界、父级支撑清理、回退点、release transition guard | 边界冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_include_cleanup` | cleanup leaf 建基线 |
| 模块树 | `runtime.parent_include_cleanup` | 白箱登记 |

---

## 当前实物状态

本叶只冻结 drained parent include cleanup，不迁移任何 handler、helper、schema、state 或 persistence owner。

| 当前文件 / 语句 | 当前状态 | 等价判定 |
| --- | --- | --- |
| `include!("run.rs")` | 位于 `src/runtime/mod.rs` | 展开 `src/runtime/run.rs` |
| `include!("mutation.rs")` | 位于 `src/runtime/mod.rs` | 展开 `src/runtime/mutation.rs` |
| `include!("backtest.rs")` | 位于 `src/runtime/mod.rs` | 展开 `src/runtime/backtest.rs` |
| `src/runtime/run.rs` | 只剩 `Drained include retained until the runtime parent support closeout.` 注释 | 无 Rust item |
| `src/runtime/mutation.rs` | 只剩 `Drained include retained until the runtime parent support closeout.` 注释 | 无 Rust item |
| `src/runtime/backtest.rs` | 只剩 `Drained parent include retained until the runtime.backtest parent residual closeout.` 注释 | 无 Rust item |

---

## 白箱输入输出

| 类别 | 输入 | 输出 | owner |
| --- | --- | --- | --- |
| cleanup input | `src/runtime/mod.rs` 三条 drained `include!(...)` | 待后续方案确认的删除清单 | `runtime.parent_include_cleanup` |
| cleanup input | `src/runtime/run.rs`、`src/runtime/mutation.rs`、`src/runtime/backtest.rs` | 待后续方案确认的 drained 文件删除清单 | `runtime.parent_include_cleanup` |
| retained surface | `src/runtime/mod.rs` child 声明、plain import、受控 re-export | 既有 public handler 出口继续可用 | 各已 closeout child |

---

## 关键 public 方法影响面

本叶不拥有 public handler，只拥有 parent include cleanup。后续实际删除时必须证明以下出口仍由既有 child re-export 保持:

| public surface | 当前 owner | 禁止事项 |
| --- | --- | --- |
| `start_test_run` | `src/runtime/run/session_start.rs` | 不得回迁到 parent include |
| `start_v4_runtime_run` | `src/runtime/run/v4_handoff.rs` | 不得新增 sibling child 横向连接 |
| `list_runs` / `get_run_detail` / `save_run_record` / `discard_run_record` | `src/runtime/run/record_store.rs` | 不得修改 persistence owner |
| `get_run_replay` / `get_run_status` | `src/runtime/run/replay_status.rs` | 不得修改 replay/status response mapping |
| `start_backtest_run` | `src/runtime/backtest/execution_start.rs` | 不得迁移 artifact/schema owner |
| `start_backtest_experiment` | `src/runtime/backtest/experiment_sweep.rs` | 不得修改 experiment limit 或 parameter grid |
| mutation / AI proposal public handlers | `src/runtime/mutation/parameter_mutation.rs`、`src/runtime/mutation/ai_proposal.rs` | 不得修改 approval、schema、frontend caller 或 lifecycle owner |

---

## 等价基线

后续实际 cleanup 的等价条件:

1. 删除 `include!("run.rs")` 后，`src/runtime/mod.rs` 的 run public re-export 仍全部来自 `run_*` child module。
2. 删除 `include!("mutation.rs")` 后，mutation public re-export 与 shared governance import 仍全部来自 `mutation_*` child module。
3. 删除 `include!("backtest.rs")` 后，backtest public re-export 与 experiment sweep re-export 仍全部来自 `backtest_*` child module。
4. 删除 `src/runtime/run.rs`、`src/runtime/mutation.rs`、`src/runtime/backtest.rs` 前，必须先确认父级不再引用对应 include。
5. `cargo check -p quantpilot` 必须证明删除后没有 missing item、duplicate item 或 visibility 漂移。
6. `api_run`、`api_backtest`、`api_mutation`、`api_experiments` 必须证明四条 runtime 行为面仍等价。

---

## 明确不变

- 不删除任何 `include!(...)`。
- 不删除 `src/runtime/run.rs`、`src/runtime/mutation.rs` 或 `src/runtime/backtest.rs`。
- 不修改 `src/runtime/mod.rs`。
- 不修改 `src/runtime/run/*`、`src/runtime/backtest/*`、`src/runtime/mutation/*` 或 `src/runtime/report_ops/*`。
- 不修改 `src/runtime/query_support.rs`、`src/runtime/response_support.rs`、`src/runtime/run_guard.rs` 或 `src/runtime/experiment_limit.rs`。
- 不修改 route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 或 release transition guard。

---

## 后续方案边界

下一步只能进入:

```text
BE-001CV-02 runtime.parent_include_cleanup 抽离方案
```

BE-001CV-02 只能设计 drained include cleanup 的最小删除方案、回退点和验证门禁。不得直接执行删除；不得把 cleanup 方案扩大成 handler 重排、module tree 重命名、state owner 迁移或 release transition。

---

## 验证要求

本批为 `no code movement` 等价基线，提交前仍需执行:

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

## 幻觉检查点

AI 声称 BE-001CV-01 完成时，必须说明:

1. 本批次是 `no code movement` 等价基线。
2. `runtime.parent_include_cleanup` 只拥有 drained include cleanup，不拥有 public handler。
3. 当前仍保留 `include!("run.rs")`、`include!("mutation.rs")`、`include!("backtest.rs")` 和三个 drained 文件。
4. 下一步只能进入 BE-001CV-02 抽离方案，不得直接删除。
5. route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 与 release transition guard 均未处理。

不得宣称 parent include 已删除、`backend.runtime` 已完成、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `305-runtime.parent_include_cleanup单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树登记 `runtime.parent_include_cleanup` 作为 cleanup leaf 基线，不创建 planned code file。
3. 全局递归下一步固定为 BE-001CV-02 `runtime.parent_include_cleanup` 抽离方案。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
