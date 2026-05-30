# v4.16.0 runtime.experiment_limit 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CT-05
> 基准: `302-runtime.experiment_limit抽离记录.md`、`301-runtime.experiment_limit补测记录.md`
> 目标子叶: `runtime.experiment_limit`
> 模块树坐标: `root.backend.runtime.runtime.experiment_limit`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CT-05 `runtime.experiment_limit` 单叶 closeout | 单叶 closeout |
| 规范矩阵 | 父级支撑常量、stop_split、后续父叶残余判断 | 收束 |
| 引导矩阵 | `root.backend.runtime.runtime.experiment_limit` | child closeout |
| 模块树 | `runtime.experiment_limit` | 白箱 closeout |

---

## closeout 判定

`runtime.experiment_limit stop_split: true`

理由:

1. 本叶只承接 `MAX_EXPERIMENT_VARIANTS = 27` 这个 experiment sweep variant limit 常量，职责足够单一。
2. `variant_count` 计算、parameter grid normalization、bad_request 输出和 variant 生成顺序仍由 `runtime.backtest.experiment_sweep.parameter_grid` 拥有，不属于本叶内部可继续细拆的 owner。
3. `src/runtime/backtest/parameter_grid.rs` 仍通过父级 `use super::*` 访问常量，没有新增 sibling child 横向 direct import。
4. `experiment_sweep_rejects_parameter_grid_above_variant_limit` 已覆盖 36 个变体超过 27 上限的拒绝路径；继续拆 `limit_value` / `limit_guard` / `message_contract` 微叶只会扩大父子接线面。

---

## 当前白箱边界

| 项 | 当前 owner | 状态 |
| --- | --- | --- |
| `MAX_EXPERIMENT_VARIANTS` | `src/runtime/experiment_limit.rs` | closeout；值为 `27` |
| 父级 surface | `src/runtime/mod.rs` | `mod experiment_limit;` + plain `use experiment_limit::MAX_EXPERIMENT_VARIANTS;` |
| 调用方 | `src/runtime/backtest/parameter_grid.rs` | 仍通过 `use super::*` |
| 超限负测 | `tests/api_experiments.rs` | `experiment_sweep_rejects_parameter_grid_above_variant_limit` |

---

## 明确未处理

- 未修改 `src/runtime/backtest/parameter_grid.rs` 的 `variant_count` 计算、拒绝条件、error code、message 文案或 variant 生成顺序。
- 未删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 未修改 `src/runtime/query_support.rs`、`src/runtime/response_support.rs` 或 `src/runtime/run_guard.rs`。
- 未修改 route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 或 release transition guard。
- 未新增 sibling child 横向连接，未启动发布版本过渡。

---

## 验证要求

本批为 `no code movement` closeout，提交前仍需执行:

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
BE-001CU-01 backend.runtime 第八轮父叶残余判断
```

BE-001CU-01 必须重新统计 `backend.runtime` 父级残余。当前已知残余为 `include!("run.rs")`、`include!("mutation.rs")` 与 `include!("backtest.rs")` drained parent include cleanup；不得从 `runtime.experiment_limit` 继续细拆。

---

## 幻觉检查点

AI 声称 BE-001CT-05 完成时，必须说明:

1. 本批次是 `no code movement` closeout。
2. `runtime.experiment_limit stop_split: true`。
3. `src/runtime/experiment_limit.rs` 继续承接 `MAX_EXPERIMENT_VARIANTS = 27`。
4. `src/runtime/backtest/parameter_grid.rs` 仍通过父级受控 surface 访问常量，没有 direct child import。
5. parent include deletion、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 与 release transition guard 均未处理。
6. 下一步只能进入 BE-001CU-01 `backend.runtime` 第八轮父叶残余判断。

不得宣称 `backend.runtime` 已完成、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `303-runtime.experiment_limit单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树登记 `runtime.experiment_limit stop_split: true`。
3. 全局递归下一步固定为 BE-001CU-01 `backend.runtime` 第八轮父叶残余判断。
4. 治理门禁、全量树覆盖、Rust 等价测试和 `git diff --check` 均通过。
