# v4.16.0 runtime.experiment_limit 补测记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CT-03
> 基准: `300-runtime.experiment_limit抽离方案.md`、`299-runtime.experiment_limit单子叶等价基线.md`
> 目标子叶: `runtime.experiment_limit`
> 模块树坐标: `root.backend.runtime.runtime.experiment_limit`
> 代码动作: test-only

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CT-03 `runtime.experiment_limit` endpoint smoke 补测 | test-first 执行 |
| 规范矩阵 | experiment variant limit、bad_request contract、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.experiment_limit` | 等价证据增强 |
| 模块树 | `runtime.experiment_limit` | 测试证据登记 |

---

## 本批变更

本批只修改:

```text
tests/api_experiments.rs
```

新增测试:

```text
experiment_sweep_rejects_parameter_grid_above_variant_limit
```

测试覆盖:

- endpoint: `POST /api/runtime/experiments/backtest-sweep`
- replay source: `deterministic_mock`
- parameter grid: 4 个 fee 值 * 3 个 slippage 值 * 3 个 latency 值
- `variant_count = 36`
- 预期 HTTP status: `StatusCode::BAD_REQUEST`
- 预期 JSON `error`: `bad_request`
- 预期 message 包含 `参数扫描展开为 36 个变体`
- 预期 message 包含 `超出当前限制 27`

---

## 明确未变

- 未创建 `src/runtime/experiment_limit.rs`。
- 未迁移 `MAX_EXPERIMENT_VARIANTS`。
- 未修改 `src/runtime/mod.rs`。
- 未修改 `src/runtime/backtest/parameter_grid.rs`。
- 未修改 `src/runtime/backtest/start_orchestration.rs`。
- 未删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 未修改 route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 或 release transition guard。

---

## 等价保护

本批将 experiment limit 的拒绝路径从文档基线变成可执行 smoke:

```text
variant_count = 36
MAX_EXPERIMENT_VARIANTS = 27
variant_count > MAX_EXPERIMENT_VARIANTS -> bad_request
```

这使 BE-001CT-04 实际抽离时，常量值、strict greater-than 语义、error code 与用户可见文案都会被 `api_experiments` 捕获。

---

## 验证要求

本批提交前必须执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_experiments
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CT-04 runtime.experiment_limit 实际抽离
```

BE-001CT-04 才允许创建 `src/runtime/experiment_limit.rs` 并迁移 `MAX_EXPERIMENT_VARIANTS`。不得处理 parent include cleanup、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CT-03 完成时，必须说明:

1. 本批次是 test-only 补测。
2. 新增测试为 `experiment_sweep_rejects_parameter_grid_above_variant_limit`。
3. `src/runtime/experiment_limit.rs` 尚未创建。
4. `MAX_EXPERIMENT_VARIANTS` 仍在 `src/runtime/mod.rs`。
5. `src/runtime/backtest/parameter_grid.rs` 未修改。
6. 下一步只能进入 BE-001CT-04 实际抽离。

不得宣称 experiment limit 已抽离、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `301-runtime.experiment_limit补测记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `tests/api_experiments.rs` 新增超限负测并通过。
3. 全局递归下一步固定为 BE-001CT-04 `runtime.experiment_limit` 实际抽离。
4. 治理门禁、`api_experiments`、全量树覆盖和 `git diff --check` 均通过。
