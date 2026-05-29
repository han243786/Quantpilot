# v4.16.0 runtime.backtest.experiment_sweep.parameter_grid 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001W-01。  
> 前置: `110-runtime.backtest.experiment_sweep单叶closeout.md`、`109-runtime.backtest.experiment_sweep抽离记录.md`、`107-runtime.backtest.experiment_sweep单子叶等价基线.md`。  
> 判定: 建立 `runtime.backtest.experiment_sweep.parameter_grid` 单子叶等价基线；本批只冻结参数网格校验、轴归一化、base fallback、去重、variant count、展开顺序、父级调用和回归证据，不移动代码。  
> 代码动作: `no code movement`。下一步只能进入 BE-001W-02 抽离方案。

---

## 选择理由

`runtime.backtest.experiment_sweep` 已完成第一轮抽离和单叶 closeout，但 `stop_split: false`。内部最清晰的下一候选是 `parameter_grid`，因为它只围绕 3 个 helper 处理参数扫描输入到 `FrontendExecutionAssumptionOverrides` 列表的转换。

1. 它不拥有 route、state、persistence、response mapping、schema、audit 或 frontend caller。
2. 它是 `start_backtest_experiment` 创建 variants 之前的确定性纯转换边界。
3. 它的风险集中在 empty grid、负数校验、base fallback、去重、variant count 和展开顺序。
4. 它可由 `api_experiments` 保护，并由 `api_backtest` / `api_evidence_contract` 证明 sibling 未漂移。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001W 从 experiment_sweep 单叶 closeout 进入内部 parameter_grid 等价基线 | 推进 |
| 规范矩阵 | 参数网格校验、base fallback、去重、variant count、展开顺序、父级调用 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.parameter_grid` | 新增基线 |
| 模块树 | `runtime.backtest.experiment_sweep.parameter_grid` | 新增 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.parameter_grid` |
| 父模块 | `runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep.parameter_grid` |
| 当前真实文件 | `src/runtime/backtest/experiment_sweep.rs`、`src/frontend_api_types.rs`、`tests/api_experiments.rs`、`tests/api_backtest.rs`、`tests/api_evidence_contract.rs` |
| 当前 helper | `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides` |
| 保留父级方法 | `start_backtest_experiment`、`resolved_backtest_execution_assumptions`、`execute_backtest_request` |
| public 方法 | 本节点不新增 public 方法；3 个 helper 保持父级私有调用 |
| 保留常量 | `MAX_EXPERIMENT_VARIANTS` |
| 输入类型 | `FrontendExperimentRequest`、`FrontendExecutionAssumptionSweepGrid`、`RuntimeProtocolCoreConfig` |
| 输出类型 | `Vec<FrontendExecutionAssumptionOverrides>`、`StatusCode`、`String` |
| 代表测试 | `cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract` |
| 治理门禁 | `tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `request.parameter_grid.fee_bps` | `FrontendExecutionAssumptionSweepGrid` | 空轴回退 base；非空轴保持去重后的输入顺序；负数返回 `bad_request` |
| 输入 | `request.parameter_grid.slippage_bps` | `FrontendExecutionAssumptionSweepGrid` | 空轴回退 base；非空轴保持去重后的输入顺序；负数返回 `bad_request` |
| 输入 | `request.parameter_grid.latency_ms` | `FrontendExecutionAssumptionSweepGrid` | 空轴回退 base latency；非空轴保持去重后的输入顺序；类型为 `u64` |
| 输入 | base execution assumptions | `resolved_backtest_execution_assumptions` | 不改变 `taker_fee_bps`、`default_slippage_bps`、`latency_assumption_ms.unwrap_or(0)` 的 fallback |
| 输出 | `Vec<FrontendExecutionAssumptionOverrides>` | `start_backtest_experiment` | fee 外层、slippage 中层、latency 内层的展开顺序必须保持 |
| 输出 | `bad_request` | HTTP error body | empty grid、负数、variant 超限的错误 code 不变 |

---

## 当前真实边界

| helper | 当前输入 | 当前输出 | 当前约束 |
| --- | --- | --- | --- |
| `normalize_experiment_float_axis` | `values: &[f64]`、`base: f64`、`field: &str` | `Result<Vec<f64>, (StatusCode, String)>` | 空轴回退 base；负数返回 `parameter_grid.{field} 必须 >= 0`；重复值只保留第一次 |
| `normalize_experiment_latency_axis` | `values: &[u64]`、`base: u64` | `Vec<u64>` | 空轴回退 base；重复值只保留第一次 |
| `build_experiment_overrides` | `FrontendExperimentRequest`、`RuntimeProtocolCoreConfig` | `Result<Vec<FrontendExecutionAssumptionOverrides>, (StatusCode, String)>` | 空网格拒绝；展开数量受 `MAX_EXPERIMENT_VARIANTS` 限制；输出三项均为 `Some` |

---

## 等价冻结项

| 行为 | 当前语义 | 不得改变 |
| --- | --- | --- |
| empty grid | `fee_bps + slippage_bps + latency_ms == 0` 返回 `bad_request` | 不得自动生成 base-only variant |
| fee negative | 任意 `fee_bps < 0.0` 返回 `parameter_grid.fee_bps 必须 >= 0` | 不得静默夹取到 0 |
| slippage negative | 任意 `slippage_bps < 0.0` 返回 `parameter_grid.slippage_bps 必须 >= 0` | 不得静默夹取到 0 |
| latency axis | `latency_ms` 是 `u64`，无负数分支 | 不得引入浮点或 signed latency |
| base fallback | 空 fee/slippage/latency 轴分别回退 base fee、base slippage、base latency or 0 | 不得改成固定默认值 |
| dedupe | `Vec::contains` 保留首次出现顺序 | 不得排序或哈希重排 |
| variant count | 三轴长度乘积超过 `MAX_EXPERIMENT_VARIANTS` 时返回 `bad_request` | 不得部分截断 |
| expansion order | fee 外层、slippage 中层、latency 内层 | 不得改变 variant 顺序 |
| output shape | 每个 variant 输出 `FrontendExecutionAssumptionOverrides { fee_bps: Some, slippage_bps: Some, latency_ms: Some }` | 不得输出 `None` |

---

## 父子通信规则

```text
runtime.backtest.experiment_sweep::start_backtest_experiment
  -> runtime.backtest.experiment_sweep.parameter_grid
     -> normalize_experiment_float_axis
     -> normalize_experiment_latency_axis
     -> build_experiment_overrides
  -> runtime.backtest.execution_start::execute_backtest_request
```

`runtime.backtest.experiment_sweep.parameter_grid` 只能被父级 `runtime.backtest.experiment_sweep` 私有调用。不得让 `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 本批次不做

- 不移动 `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides`。
- 不新增 `src/runtime/backtest/parameter_grid.rs`。
- 不修改 `FrontendExecutionAssumptionSweepGrid` 或 `FrontendExecutionAssumptionOverrides` schema。
- 不修改 `MAX_EXPERIMENT_VARIANTS`。
- 不新增、删除或改写 API 测试。
- 不迁移 `start_backtest_experiment`、`execute_backtest_request`、route aggregate、persistence、response mapping、schema、state、audit、frontend caller 或发布过渡连接。

---

## 后续队列

下一步只能进入 BE-001W-02 `runtime.backtest.experiment_sweep.parameter_grid` 抽离方案。方案阶段必须先说明是否新增 focused unit/API tests；不得直接移动 helper，也不得顺手删除 `src/runtime/backtest.rs` drained parent include 或迁移 shared owner。

---

## 回归保护

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test --no-run`
- `cargo test -p quantpilot --test api_experiments`
- `cargo test -p quantpilot --test api_backtest`
- `cargo test -p quantpilot --test api_evidence_contract`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `git diff --check`

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep.parameter_grid` 已建立基线时，必须说明本批是 `no code movement`，只冻结 3 个 helper 的参数网格等价边界。不得宣称 helper 已抽离、`src/runtime/backtest/parameter_grid.rs` 已存在、parameter_grid 已 closeout、`stop_split: true` 已设置、tests 已新增、route aggregate、execution_start、persistence、response mapping、schema、state、audit、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `111-runtime.backtest.experiment_sweep.parameter_grid单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树新增 `runtime.backtest.experiment_sweep.parameter_grid` 白箱节点。
3. 全量树覆盖本基线文档与下一步 BE-001W-02。
4. 治理门禁能发现本基线、`no code movement`、3 个 helper、`MAX_EXPERIMENT_VARIANTS`、禁止迁移边界和下一步。
5. 代表性治理门禁与 Rust 编译门禁继续通过。
