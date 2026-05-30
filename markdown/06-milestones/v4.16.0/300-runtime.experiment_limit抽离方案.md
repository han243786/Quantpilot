# v4.16.0 runtime.experiment_limit 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CT-02
> 基准: `299-runtime.experiment_limit单子叶等价基线.md`、`298-backend.runtime第七轮父叶残余判断.md`
> 目标子叶: `runtime.experiment_limit`
> 模块树坐标: `root.backend.runtime.runtime.experiment_limit`
> 判定: 采用 test-first。下一批 BE-001CT-03 只补 experiment variant limit 超限负测，不创建 child module；BE-001CT-04 才允许创建 planned child 并迁移 `MAX_EXPERIMENT_VARIANTS`。
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CT-02 `runtime.experiment_limit` 抽离方案 | 抽离方案 |
| 规范矩阵 | test-first、父级支撑常量、父子通信、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.experiment_limit` | planned child 抽离路径 |
| 模块树 | `runtime.experiment_limit` | 下一批先补 smoke |

---

## 方案判定

本子叶选择 test-first。

原因:

1. `runtime.experiment_limit` 的核心语义就是 `MAX_EXPERIMENT_VARIANTS = 27` 与 `variant_count > MAX_EXPERIMENT_VARIANTS` 拒绝路径。
2. 当前 `tests/api_experiments.rs` 已覆盖 parameter grid 主路径，但没有覆盖超限 bad_request。
3. 先补一个最小 endpoint 负测，可以在物理迁移前冻结 `StatusCode::BAD_REQUEST`、`error = "bad_request"` 与用户可见超限文案，避免实际抽离后才发现 limit contract 漂移。

因此执行顺序固定为:

1. BE-001CT-03: 只新增 `api_experiments` 超限负测，不创建 child module，不迁移常量。
2. BE-001CT-04: 在负测通过并提交后，创建 child module 并迁移 `MAX_EXPERIMENT_VARIANTS`。
3. BE-001CT-05: 做单叶 closeout，并判断本叶是否还值得继续拆分。

---

## BE-001CT-03 允许动作

BE-001CT-03 只允许修改:

```text
tests/api_experiments.rs
```

新增测试建议名:

```text
experiment_sweep_rejects_parameter_grid_above_variant_limit
```

测试输入:

- endpoint: `POST /api/runtime/experiments/backtest-sweep`
- `backtest_options.replay_source`: `deterministic_mock`
- `parameter_grid.fee_bps`: 4 个值
- `parameter_grid.slippage_bps`: 3 个值
- `parameter_grid.latency_ms`: 3 个值
- 预期 `variant_count = 36`，大于当前限制 `27`

允许断言:

- HTTP status 为 `StatusCode::BAD_REQUEST`。
- JSON `error` 为 `bad_request`。
- JSON `message` 包含 `参数扫描展开为 36 个变体`。
- JSON `message` 包含 `超出当前限制 27`。

BE-001CT-03 禁止创建 `src/runtime/experiment_limit.rs`、迁移 `MAX_EXPERIMENT_VARIANTS`、重写 parameter grid normalization、处理 parent include cleanup 或启动 release transition。

---

## BE-001CT-04 允许动作

只有 BE-001CT-03 通过并提交后，BE-001CT-04 才允许创建:

```text
src/runtime/experiment_limit.rs
```

目标 child 内容只允许承接父级支撑常量:

```rust
pub(super) const MAX_EXPERIMENT_VARIANTS: usize = 27;
```

父级 `src/runtime/mod.rs` 目标形态:

```rust
mod experiment_limit;
mod run_guard;

use experiment_limit::MAX_EXPERIMENT_VARIANTS;
```

迁移清单只限:

- 从 `src/runtime/mod.rs` 移出 `const MAX_EXPERIMENT_VARIANTS: usize = 27;`。
- 在 `src/runtime/mod.rs` 增加 `mod experiment_limit;`。
- 在 `src/runtime/mod.rs` 增加 plain import `use experiment_limit::MAX_EXPERIMENT_VARIANTS;`。

`src/runtime/backtest/parameter_grid.rs` 继续通过父级 `use super::*` 获得该常量，不新增 sibling child 横向 direct import。

---

## 明确保持不变

- 不修改 `src/runtime/backtest/parameter_grid.rs` 的 `variant_count` 计算、拒绝条件、error code、message 文案或 variant 生成顺序。
- 不修改 `src/runtime/backtest/start_orchestration.rs`。
- 不修改 `tests/api_experiments.rs` 既有两个主路径/删除路径测试的断言。
- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 不修改 `src/runtime/query_support.rs`、`src/runtime/response_support.rs` 或 `src/runtime/run_guard.rs`。
- 不修改 route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 或 release transition guard。

---

## 回退点

BE-001CT-03 若负测不稳定，停止在测试批次，不进入常量迁移。

BE-001CT-04 若抽离后编译或 smoke 失败，回退范围仅限:

- 删除 `src/runtime/experiment_limit.rs`。
- 从 `src/runtime/mod.rs` 移除 `mod experiment_limit`。
- 从 `src/runtime/mod.rs` 移除 `use experiment_limit::MAX_EXPERIMENT_VARIANTS`。
- 将 `const MAX_EXPERIMENT_VARIANTS: usize = 27;` 放回 `src/runtime/mod.rs` 原位置。

不得回改 BE-001CT-03 负测来掩盖抽离失败。

---

## 验证要求

BE-001CT-02 提交前执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_experiments
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001CT-03 补测后必须执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_experiments
```

BE-001CT-04 实际抽离后必须执行:

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
BE-001CT-03 runtime.experiment_limit endpoint smoke 补测
```

BE-001CT-03 不得创建 planned child，不得迁移 `MAX_EXPERIMENT_VARIANTS`，不得处理 parent include cleanup、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CT-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. 本叶选择 test-first。
3. 下一批 BE-001CT-03 只补 `tests/api_experiments.rs` 超限负测。
4. BE-001CT-04 才允许创建 `src/runtime/experiment_limit.rs` 并迁移 `MAX_EXPERIMENT_VARIANTS`。
5. `src/runtime/backtest/parameter_grid.rs` 当前不改，未来也不得新增 sibling child 横向 direct import。
6. parent include deletion、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 与 release transition guard 均未处理。

不得宣称 experiment limit 已抽离、负测已新增、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `300-runtime.experiment_limit抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树登记 `runtime.experiment_limit` 的 test-first 抽离路径，但不把 planned child 文件列入真实文件。
3. 全局递归下一步固定为 BE-001CT-03 `runtime.experiment_limit` endpoint smoke 补测。
4. 治理门禁、`api_experiments`、全量树覆盖和 `git diff --check` 均通过。
