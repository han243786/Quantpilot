# v4.16.0 runtime.experiment_limit 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CT-04
> 基准: `301-runtime.experiment_limit补测记录.md`、`300-runtime.experiment_limit抽离方案.md`
> 目标子叶: `runtime.experiment_limit`
> 模块树坐标: `root.backend.runtime.runtime.experiment_limit`
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CT-04 `runtime.experiment_limit` 实际抽离 | 实际抽离 |
| 规范矩阵 | 父级支撑常量、父子通信、plain import、release transition guard | 执行 |
| 引导矩阵 | `root.backend.runtime.runtime.experiment_limit` | child 落地 |
| 模块树 | `runtime.experiment_limit` | 真实文件登记 |

---

## 本批变更

新增 child module:

```text
src/runtime/experiment_limit.rs
```

child 内容:

```rust
pub(super) const MAX_EXPERIMENT_VARIANTS: usize = 27;
```

父级 `src/runtime/mod.rs` 已增加:

```rust
mod experiment_limit;
use experiment_limit::MAX_EXPERIMENT_VARIANTS;
```

父级原内联常量已移除:

```rust
const MAX_EXPERIMENT_VARIANTS: usize = 27;
```

---

## 等价保持

`src/runtime/backtest/parameter_grid.rs` 未修改，仍通过父级 `use super::*` 访问 `MAX_EXPERIMENT_VARIANTS`。

保持不变的语义:

```text
variant_count = fee_values.len() * slippage_values.len() * latency_values.len()
variant_count > MAX_EXPERIMENT_VARIANTS -> bad_request
MAX_EXPERIMENT_VARIANTS = 27
```

BE-001CT-03 新增的 `experiment_sweep_rejects_parameter_grid_above_variant_limit` 覆盖 36 个变体超过 27 上限的拒绝路径。

---

## 明确未变

- 未修改 `tests/api_experiments.rs`。
- 未修改 `src/runtime/backtest/parameter_grid.rs`。
- 未修改 `src/runtime/backtest/start_orchestration.rs`。
- 未删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 未修改 `src/runtime/query_support.rs`、`src/runtime/response_support.rs` 或 `src/runtime/run_guard.rs`。
- 未修改 route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 或 release transition guard。

---

## 回退点

若 BE-001CT-04 失败，回退范围仅限:

1. 删除 `src/runtime/experiment_limit.rs`。
2. 从 `src/runtime/mod.rs` 移除 `mod experiment_limit`。
3. 从 `src/runtime/mod.rs` 移除 `use experiment_limit::MAX_EXPERIMENT_VARIANTS`。
4. 将 `const MAX_EXPERIMENT_VARIANTS: usize = 27;` 放回 `src/runtime/mod.rs`。

不得删除 BE-001CT-03 的超限负测。

---

## 验证要求

本批提交前必须执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot runtime_run_guard_resets_on_drop
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CT-05 runtime.experiment_limit 单叶 closeout
```

BE-001CT-05 只判断 `runtime.experiment_limit` 是否还值得继续细拆。不得处理 parent include cleanup、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CT-04 完成时，必须说明:

1. `src/runtime/experiment_limit.rs` 已创建。
2. `MAX_EXPERIMENT_VARIANTS` 已从 `src/runtime/mod.rs` 迁入 child，值仍为 `27`。
3. `src/runtime/mod.rs` 只保留 `mod experiment_limit` 与 plain import。
4. `src/runtime/backtest/parameter_grid.rs` 未修改，仍通过父级 `use super::*` 获得常量。
5. parent include deletion、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 与 release transition guard 均未处理。
6. 下一步只能进入 BE-001CT-05 单叶 closeout。

不得宣称 parent include 已删除、`backend.runtime` 已完成、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `302-runtime.experiment_limit抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/experiment_limit.rs` 进入模块树和全量树真实文件。
3. `api_experiments` 的 36 > 27 超限负测通过。
4. 全局递归下一步固定为 BE-001CT-05 `runtime.experiment_limit` 单叶 closeout。
5. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
