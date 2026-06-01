# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering 父叶残余判断
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FW-01
> 基线: `492-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering单叶closeout.md`
> 目标父叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 判定: 父叶仍有残余，本轮选择 `spread_observer_lowering`
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 代码动作: no code movement
> 下一步: BE-001FX-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FW-01 `intent_lowering` 父叶残余判断 | 回到父叶 / 选择下一子叶 |
| 规范矩阵 | recursive residual judgment / stop_split false / child selection / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | 子叶队列继续推进 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | `stop_split: false` |

---

## 已完成子叶确认

上一轮子叶已经 closeout:

```text
intent_lowering actual_extraction_done
intent_lowering closeout_done
intent_lowering stop_split: false
```

当前真实 owner:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
append_intent_lowering_lines
```

父级只通过受控父子通信调用:

```rust
mod intent_lowering;
intent_lowering::append_intent_lowering_lines(nodes, edges, &mut qs_lines)?;
```

---

## 父叶当前残余

`intent_lowering.rs` 当前仍承载以下职责簇:

| 残余簇 | 代表行为 | 当前判断 |
| --- | --- | --- |
| `shared_intent_context` | `module_key`、`instrument`、`node_id`、upstream edge、`source_var` 派生 | 暂缓，所有分支共用，先保持父叶输入上下文 |
| `double_ma_lowering` | `fast_period` / `slow_period` -> SMA crossover BUY | 稍后，可作为薄分支 |
| `rsi_lowering` | `period` / `oversold_threshold` -> RSI BUY | 稍后，可作为薄分支 |
| `ma_deviation_lowering` | `lookback` / `baseline_period` -> MA ratio SELL | 稍后，可作为薄分支 |
| `macd_lowering` | `fast_period` / `slow_period` / `signal_period` -> BUY/SELL | 稍后，可作为中等分支 |
| `momentum_lowering` | `lookback` / `threshold_ratio` -> momentum BUY | 稍后，可作为薄分支 |
| `zscore_lowering` | `window` / `entry_z` -> zscore BUY | 稍后，可作为薄分支 |
| `spread_observer_lowering` | 双上游 source、`align_asof`、`spread_output_code`、`comparison_threshold`、`comparison_op_code` | 本轮选择，职责最重且最独立 |
| `unsupported_intent_failure` | unsupported intent `anyhow::bail!` | 暂缓，跟随父叶 hard failure 边界保留 |

因此父叶继续保持:

```text
intent_lowering parent_residual_judgment
intent_lowering stop_split: false
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering residual_exists
```

---

## 下一子叶选择

本轮选择:

```text
BE-001FX-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering
spread_observer_lowering_selected
```

优先选择 `spread_observer_lowering` 的原因:

1. 它是当前 built-in intent 分支中最长的独立代码块。
2. 它重新读取全部 upstream edges，和父叶 shared upstream single-source path 不完全相同。
3. 它拥有左右 source fallback、`max_time_diff_ms`、`spread_output_code`、`comparison_threshold`、`comparison_op_code` 等独立输入面。
4. 它生成 `align_asof(field(...))`、`spread(...)` 和 threshold condition 三段 QS 行，回归缺口更容易扩大。
5. 先抽该分支可以快速验证 branch-level extraction 模式，后续再决定是否抽 shared context 或薄分支。

---

## 保留不变量

后续 BE-001FX-01 必须冻结:

```text
builtin.intent.spread_observer
upstream_sources
left_source
right_source
source_var fallback
max_time_diff_ms
spread_output_code
comparison_threshold
comparison_op_code
align_asof
field({}, name="bid")
field({}, name="ask")
spread({}_left, {}_right, output="{}")
emit Intent("BUY", instrument="{}", quantity=1.0)
```

父叶仍必须保留:

```text
builtin.intent.double_ma
builtin.intent.rsi
builtin.intent.ma_deviation
builtin.intent.macd
builtin.intent.momentum
builtin.intent.zscore
unsupported intent
anyhow::bail!
```

父子通信规则仍是:

```text
formal_module_conversion -> intent_lowering
intent_lowering -> spread_observer_lowering
```

禁止新增:

```text
formal_module_conversion -> spread_observer_lowering
compile_api -> spread_observer_lowering
graph_api -> spread_observer_lowering
runtime sibling -> spread_observer_lowering
frontend -> spread_observer_lowering
sibling horizontal link
```

release transition guard: 当前仍未进入发布版本过渡，不得用性能理由绕过父子通信规则。

---

## 不进入范围

本批不处理:

1. 不修改 Rust 代码。
2. 不创建 `spread_observer_lowering.rs`。
3. 不移动 `builtin.intent.spread_observer` 分支。
4. 不抽 shared intent context。
5. 不抽其它 built-in intent branch。
6. 不改 unsupported intent hard failure。
7. 不改 `formal_module_conversion.rs` 父级调用方式。
8. 不新增 sibling horizontal link。
9. 不启动 release transition。
10. 不宣称 `intent_lowering`、`formal_module_conversion`、`backend.graph_compile`、`backend` 或 Rust 重构完成。

---

## 下一步边界

下一步只能进入:

```text
BE-001FX-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering
```

BE-001FX-01 只能建立单子叶等价基线，冻结 `spread_observer` 分支的输入、输出、fallback、QS line 生成顺序、父子通信面和回归门禁。不得直接创建 child file 或移动代码。

---

## 验证要求

本批是 `no code movement` 父叶残余判断，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

---

## 幻觉检查点

AI 声称 BE-001FW-01 完成时，必须说明:

1. 本批是 `no code movement` 父叶残余判断。
2. `intent_lowering stop_split: false`。
3. 下一步只进入 BE-001FX-01 `spread_observer_lowering` 单子叶等价基线。
4. 不得宣称 `spread_observer_lowering` 已抽离。
5. 不得宣称 `intent_lowering`、`formal_module_conversion`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `493-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `intent_lowering parent_residual_judgment` 已记录。
3. `spread_observer_lowering_selected` 已记录。
4. 下一步固定为 BE-001FX-01 `spread_observer_lowering` 单子叶等价基线。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
