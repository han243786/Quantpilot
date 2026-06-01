# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering 抽离记录
> 版本类型: MINOR architecture / refactor
> 执行档位: 标准
> 批次: BE-001FX-03
> 基线: `495-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering抽离方案.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering`
> 判定: 实际抽离完成
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering`
> 代码动作: actual extraction
> 下一步: BE-001FX-04 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FX-03 `spread_observer_lowering` 实际抽离记录 | 方案落地 |
| 规范矩阵 | actual extraction / parent-child communication / equivalence preservation / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` | child file 落位 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` | actual_extraction_done |

---

## 实际变更

本批创建 child file:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/spread_observer_lowering.rs
```

父级 `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs` 新增:

```rust
mod spread_observer_lowering;
```

父级 branch 现在只保留受控父子调用:

```rust
"builtin.intent.spread_observer" => {
    spread_observer_lowering::append_spread_observer_lowering_lines(
        node,
        edges,
        cfg,
        node_id,
        &source_var,
        instrument,
        qs_lines,
    );
}
```

落地标记:

```text
spread_observer_lowering actual_extraction_done
spread_observer_lowering plan_frozen
spread_observer_lowering baseline_frozen
```

---

## Child Helper

child 承接 helper:

```rust
pub(super) fn append_spread_observer_lowering_lines(
    node: &Value,
    edges: &[Value],
    cfg: &Value,
    node_id: &str,
    source_var: &str,
    instrument: &str,
    qs_lines: &mut Vec<String>,
)
```

helper 保持 `pub(super)`，只能由父叶 `intent_lowering` 调用；helper 返回 `()`，没有新增 error path。父级 `append_intent_lowering_lines` 仍负责 `anyhow::Result<()>` 与 unsupported intent `anyhow::bail!`。

---

## 等价保持点

本批只移动 `builtin.intent.spread_observer` branch，保留以下语义:

```text
upstream_sources
left_source
right_source
source_var fallback
max_time_diff_ms default 5000
spread_output_code Some(1) -> bps
spread_output_code other -> ratio
comparison_threshold default 0.0
comparison_op_code Some(3) -> >=
comparison_op_code other -> >
align_asof
field({}, name="bid")
field({}, name="ask")
spread({}_left, {}_right, output="{}")
emit Intent("BUY", instrument="{}", quantity=1.0)
```

QS line 顺序保持:

```text
left align_asof bid
right align_asof ask
spread signal
if condition
emit BUY Intent
```

---

## 非目标确认

本批未移动:

```text
shared_intent_context
builtin.intent.double_ma
builtin.intent.rsi
builtin.intent.ma_deviation
builtin.intent.macd
builtin.intent.momentum
builtin.intent.zscore
unsupported intent
anyhow::bail!
formal_module_conversion -> intent_lowering
```

本批未新增:

```text
formal_module_conversion -> spread_observer_lowering
compile_api -> spread_observer_lowering
graph_quantscript_api -> spread_observer_lowering
graph_api -> spread_observer_lowering
runtime sibling -> spread_observer_lowering
frontend -> spread_observer_lowering
sibling horizontal link
release transition
```

---

## 父子通信规则

实际新增连接只有:

```text
intent_lowering -> spread_observer_lowering
```

上层连接仍是:

```text
formal_module_conversion -> intent_lowering
```

release transition guard: 当前没有开发者发布过渡决定，不允许以性能为理由绕过父子通信。

---

## 验证要求

本批提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot compile_endpoint_lowers_graph_spread_bps_to_structured_threshold_condition
cargo test -p quantpilot spread_bps_condition_lowers_equivalently_across_graph_and_strategy_ir
cargo test -p quantpilot spread_bps_condition_lowers_equivalently_across_formal_graph_and_strategy_ir
```

---

## 下一步边界

下一步只能进入:

```text
BE-001FX-04
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering
```

BE-001FX-04 只允许做单叶 closeout 与是否继续细拆判断，不得顺手移动其它 built-in intent branch、shared context、unsupported intent failure 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001FX-03 完成时，必须说明:

1. `spread_observer_lowering actual_extraction_done` 成立。
2. child file 已创建，父级只保留 `mod spread_observer_lowering;` 和受控调用。
3. 其它 built-in intent branch 与 unsupported intent hard fail 未移动。
4. 下一步只能进入 BE-001FX-04 单叶 closeout。
5. 不得宣称 `intent_lowering`、`formal_module_conversion`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `496-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/spread_observer_lowering.rs` 已创建并承接 spread observer branch。
3. 父级 `intent_lowering.rs` 只通过 `spread_observer_lowering::append_spread_observer_lowering_lines` 单向调用 child。
4. 下一步固定为 BE-001FX-04 单叶 closeout。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check、三条 spread observer targeted tests 和 `git diff --check` 均通过。
