# v4.16.0 runtime.experiment_limit 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CT-01
> 基准: `298-backend.runtime第七轮父叶残余判断.md`、`297-runtime.run_guard单叶closeout.md`
> 目标子叶: `runtime.experiment_limit`
> 模块树坐标: `root.backend.runtime.runtime.experiment_limit`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CT-01 `runtime.experiment_limit` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | 父子通信、experiment limit、visibility、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.experiment_limit` | planned child |
| 模块树 | `runtime.experiment_limit` | 白箱基线登记 |

---

## 当前真实边界

目标常量当前仍在 `src/runtime/mod.rs`:

```rust
const MAX_EXPERIMENT_VARIANTS: usize = 27;
```

唯一调用方当前在 `src/runtime/backtest/parameter_grid.rs`:

```rust
let variant_count = fee_values.len() * slippage_values.len() * latency_values.len();
if variant_count > MAX_EXPERIMENT_VARIANTS {
    return Err(json_bad_request(
        "bad_request",
        format!(
            "参数扫描展开为 {variant_count} 个变体，超出当前限制 {MAX_EXPERIMENT_VARIANTS}"
        ),
    ));
}

let mut variants = Vec::with_capacity(variant_count);
```

planned child 文件尚未创建:

```text
src/runtime/experiment_limit.rs
```

---

## 输入 / 输出 / 处理方

| 项 | 内容 | 约束 |
| --- | --- | --- |
| 输入 | `fee_values.len()`、`slippage_values.len()`、`latency_values.len()` | 输入轴已由 parameter_grid normalization 处理 |
| 限制值 | `MAX_EXPERIMENT_VARIANTS = 27` | 本基线不改变限制值 |
| 判定 | `variant_count > MAX_EXPERIMENT_VARIANTS` | 严格保留大于时拒绝，等于 27 时允许 |
| 拒绝输出 | `json_bad_request("bad_request", "...超出当前限制 {MAX_EXPERIMENT_VARIANTS}")` | 不改变错误 code 或用户可见文案 |
| 允许输出 | `Vec::with_capacity(variant_count)` 后生成 overrides | 不改变 variant 生成顺序 |
| 调用方 | `build_experiment_overrides` | 不改变 start orchestration、record lifecycle 或 backtest artifact |

---

## 关键 public 方法 / 类型

| 方法 / 类型 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `MAX_EXPERIMENT_VARIANTS` | 无 | `usize = 27` | `runtime.backtest.experiment_sweep.parameter_grid` | 不得改值、不得改变大于阈值拒绝语义 |
| `build_experiment_overrides` | `FrontendExperimentRequest`、`RuntimeProtocolCoreConfig` | `Vec<FrontendExecutionAssumptionOverrides>` 或 bad request | `runtime.backtest.experiment_sweep.start_orchestration` | 本叶不得接管 parameter_grid normalization 或 start orchestration |

`runtime.experiment_limit` 不是 HTTP route facade，不是 response schema，不是 runtime persistence owner，也不是 storage lifecycle owner。它只应承接 experiment sweep variant limit 这个父级支撑常量。

---

## 调用方等价

当前调用路径:

```text
start_backtest_experiment
  -> build_experiment_overrides
  -> normalize_experiment_float_axis / normalize_experiment_latency_axis
  -> variant_count = fee * slippage * latency
  -> variant_count > MAX_EXPERIMENT_VARIANTS: bad_request
  -> otherwise build FrontendExecutionAssumptionOverrides variants
```

BE-001CT-02/03 若进入抽离，`src/runtime/backtest/parameter_grid.rs` 不得直接横向 import sibling child。仍必须通过 `src/runtime/mod.rs` controlled experiment limit surface 暴露 `MAX_EXPERIMENT_VARIANTS`，以维持父子通信规则。

---

## 当前测试证据

当前等价证据来自:

- `cargo test -p quantpilot --test api_experiments`
- `cargo test -p quantpilot --test api_backtest`
- `cargo test -p quantpilot --test api_run`
- `cargo test -p quantpilot --test api_mutation`
- `cargo test -p quantpilot --test api_ai_proposal`
- `cargo test -p quantpilot --test api_v1_reports`
- `cargo test -p quantpilot --test api_v1_ops_health`

其中 `api_experiments` 覆盖 parameter grid 与 variant summaries 的主路径。当前没有新增超限拒绝的专门负测；BE-001CT-02 若认为需要，可先提出 test-first 方案，但不得在本基线直接新增测试或移动代码。

---

## 明确排除

- 不创建 `src/runtime/experiment_limit.rs`。
- 不迁移 `MAX_EXPERIMENT_VARIANTS`。
- 不修改 `src/runtime/backtest/parameter_grid.rs`。
- 不修改 `src/runtime/backtest/start_orchestration.rs` 或 experiment record lifecycle。
- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 不修改 `src/runtime/query_support.rs`、`src/runtime/response_support.rs` 或 `src/runtime/run_guard.rs`。
- 不修改 `AppState`、route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 或 release transition guard。

---

## 验证要求

本批为 `no code movement` 基线，提交前仍需执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot runtime_run_guard_resets_on_drop
cargo test -p quantpilot --test api_experiments
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
BE-001CT-02 runtime.experiment_limit 抽离方案
```

BE-001CT-02 必须先决定是否 test-first，再固定 `src/runtime/experiment_limit.rs`、父级声明、plain import、`pub(super)` visibility 与回退点。不得直接进入实际抽离，不得处理 parent include cleanup、schema owner、frontend caller 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CT-01 完成时，必须说明:

1. 本批次是 `no code movement` 等价基线。
2. `src/runtime/experiment_limit.rs` 尚未创建。
3. `MAX_EXPERIMENT_VARIANTS` 仍在 `src/runtime/mod.rs`，值仍为 `27`。
4. 唯一调用方仍为 `src/runtime/backtest/parameter_grid.rs` 的 `variant_count` guard。
5. 当前语义为 `variant_count > MAX_EXPERIMENT_VARIANTS` 时返回 `bad_request`，否则按既有顺序生成 `FrontendExecutionAssumptionOverrides`。
6. parent include deletion、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 与 release transition guard 均未处理。
7. 下一步只能进入 BE-001CT-02 抽离方案。

不得宣称 experiment limit 已抽离、parameter_grid 已重构、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `299-runtime.experiment_limit单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树登记 `root.backend.runtime.runtime.experiment_limit` planned child 基线，但不把 planned child 文件列入真实文件。
3. 全局递归下一步固定为 BE-001CT-02 `runtime.experiment_limit` 抽离方案。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
