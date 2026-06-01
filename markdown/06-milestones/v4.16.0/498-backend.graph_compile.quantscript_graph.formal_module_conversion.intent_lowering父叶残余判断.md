# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering 父叶残余判断
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FY-01
> 基线: `497-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering单叶closeout.md`
> 目标父叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 判定: 父叶仍有残余，本轮选择 `macd_lowering`
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering`
> 代码动作: no code movement
> 下一步: BE-001FZ-01 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FY-01 `intent_lowering` 父叶残余判断 | 回到父叶 / 选择下一子叶 |
| 规范矩阵 | recursive residual judgment / child selection / stop_split false / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | 子叶队列继续推进 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering` | parent_residual_judgment |

---

## 已完成子叶确认

上一轮子叶已经 closeout:

```text
spread_observer_lowering baseline_frozen
spread_observer_lowering plan_frozen
spread_observer_lowering actual_extraction_done
spread_observer_lowering closeout_done
spread_observer_lowering stop_split: true
```

父级当前只通过受控父子通信调用:

```rust
mod spread_observer_lowering;
spread_observer_lowering::append_spread_observer_lowering_lines(
    node,
    edges,
    cfg,
    node_id,
    &source_var,
    instrument,
    qs_lines,
);
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
| `macd_lowering` | `fast_period` / `slow_period` / `signal_period` -> BUY/SELL | 本轮选择，双向输出且参数面更厚 |
| `momentum_lowering` | `lookback` / `threshold_ratio` -> momentum BUY | 稍后，可作为薄分支 |
| `zscore_lowering` | `window` / `entry_z` -> zscore BUY | 稍后，可作为薄分支 |
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
BE-001FZ-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering
macd_lowering_selected
```

优先选择 `macd_lowering` 的原因:

1. 它拥有 `fast_period`、`slow_period`、`signal_period` 三个参数输入。
2. 它输出 `macd_val`，并同时生成 BUY 与 SELL 两个方向的 intent。
3. 它比 double_ma / rsi / momentum / zscore / ma_deviation 薄分支更容易在迁移时产生等价缺口。
4. spread observer 已 closeout，下一步处理最厚的 remaining branch 更符合递归优先级。
5. 先抽 `macd_lowering` 后，剩余分支可以按薄叶节奏逐个处理。

---

## 保留不变量

后续 BE-001FZ-01 必须冻结:

```text
builtin.intent.macd
fast_period default 12
slow_period default 26
signal_period default 9
macd({}, {}, {}, {})
macd_val > 0
emit Intent("BUY", instrument="{}", quantity=1.0)
macd_val < 0
emit Intent("SELL", instrument="{}", quantity=1.0)
```

父叶仍必须保留:

```text
shared_intent_context
builtin.intent.double_ma
builtin.intent.rsi
builtin.intent.ma_deviation
builtin.intent.momentum
builtin.intent.zscore
builtin.intent.spread_observer
spread_observer_lowering::append_spread_observer_lowering_lines
unsupported intent
anyhow::bail!
```

父子通信规则仍是:

```text
formal_module_conversion -> intent_lowering
intent_lowering -> spread_observer_lowering
intent_lowering -> macd_lowering
```

禁止新增:

```text
formal_module_conversion -> macd_lowering
compile_api -> macd_lowering
graph_quantscript_api -> macd_lowering
graph_api -> macd_lowering
runtime sibling -> macd_lowering
frontend -> macd_lowering
sibling horizontal link
```

release transition guard: 当前仍未进入发布版本过渡，不得用性能理由绕过父子通信规则。

---

## 不进入范围

本批不处理:

1. 不修改 Rust 代码。
2. 不创建 `macd_lowering.rs`。
3. 不移动 `builtin.intent.macd` 分支。
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
BE-001FZ-01
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.macd_lowering
```

BE-001FZ-01 只能建立单子叶等价基线，冻结 `macd` 分支的输入、输出、fallback、QS line 生成顺序、父子通信面和回归门禁。不得直接创建 child file 或移动代码。

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

AI 声称 BE-001FY-01 完成时，必须说明:

1. 本批是 `no code movement` 父叶残余判断。
2. `intent_lowering stop_split: false`。
3. `macd_lowering_selected` 只代表下一基线选择。
4. 下一步只进入 BE-001FZ-01 `macd_lowering` 单子叶等价基线。
5. 不得宣称 `macd_lowering` 已抽离。
6. 不得宣称 `intent_lowering`、`formal_module_conversion`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `498-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `intent_lowering parent_residual_judgment` 已记录。
3. `macd_lowering_selected` 已记录。
4. 下一步固定为 BE-001FZ-01 `macd_lowering` 单子叶等价基线。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
