# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering 抽离方案
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FX-02
> 基线: `494-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering单子叶等价基线.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering`
> 判定: 抽离方案
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering`
> 代码动作: no code movement
> 下一步: BE-001FX-03 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FX-02 `spread_observer_lowering` 抽离方案 | 方案冻结 |
| 规范矩阵 | extraction plan / branch-level helper / parent-child communication / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` | planned child 接口设计 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` | plan_frozen |

---

## Planned Child

下一批 BE-001FX-03 只允许创建:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/spread_observer_lowering.rs
```

父级 `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs` 只允许新增:

```rust
mod spread_observer_lowering;
```

并将当前 match branch 改为父到子的受控调用:

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

方案标记:

```text
spread_observer_lowering plan_frozen
spread_observer_lowering baseline_frozen
```

---

## Planned Helper Signature

planned helper 固定为:

```rust
use serde_json::Value;

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

约束:

1. `pub(super)` 只允许父叶 `intent_lowering` 调用。
2. helper 返回 `()`，不得新增 error path；父级 `append_intent_lowering_lines` 继续持有 `anyhow::Result<()>` 与 unsupported intent hard fail。
3. `node`、`edges`、`cfg`、`node_id`、`source_var`、`instrument`、`qs_lines` 均沿用 BE-001FX-01 白箱输入面。
4. helper 只追加 spread observer QuantScript lines，不得读取其它 built-in intent branch。

---

## 允许移动的代码

BE-001FX-03 只允许移动 `src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs` 中的当前 branch:

```text
builtin.intent.spread_observer
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

实际迁移后必须保持 QS line 顺序:

```text
left align_asof bid
right align_asof ask
spread signal
if condition
emit BUY Intent
```

---

## 不允许移动的代码

BE-001FX-03 不得移动:

1. `shared_intent_context`，包括 `module_key`、`instrument`、`node_id` 与单上游 `source_var` 派生。
2. `builtin.intent.double_ma`。
3. `builtin.intent.rsi`。
4. `builtin.intent.ma_deviation`。
5. `builtin.intent.macd`。
6. `builtin.intent.momentum`。
7. `builtin.intent.zscore`。
8. unsupported intent `anyhow::bail!`。
9. `formal_module_conversion.rs` 父级调用。
10. parser、route surface、artifact target projection、frontend caller 或 runtime caller。

---

## 父子通信规则

BE-001FX-03 后唯一允许新增连接是:

```text
intent_lowering -> spread_observer_lowering
```

现有上层连接保持:

```text
formal_module_conversion -> intent_lowering
```

不得新增:

```text
formal_module_conversion -> spread_observer_lowering
compile_api -> spread_observer_lowering
graph_quantscript_api -> spread_observer_lowering
graph_api -> spread_observer_lowering
runtime sibling -> spread_observer_lowering
frontend -> spread_observer_lowering
sibling horizontal link
```

release transition guard: 当前没有开发者发布过渡决定，不允许以性能为理由绕过父子通信。

---

## 回退方案

若 BE-001FX-03 验证失败，回退只允许:

1. 将 `append_spread_observer_lowering_lines` 函数体恢复回父级 `builtin.intent.spread_observer` branch。
2. 移除父级 `mod spread_observer_lowering;`。
3. 删除 planned child file。
4. 保留 BE-001FX-01 / BE-001FX-02 治理记录，新增失败 closeout 或修正记录，不改写历史语义。

---

## 下一步边界

下一步只能进入:

```text
BE-001FX-03
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering
```

BE-001FX-03 只允许执行上述 planned child 创建、父级 `mod spread_observer_lowering;` 声明和单 branch 移动。不得顺手处理其它 built-in intent branch、shared context、unsupported intent failure 或 release transition。

---

## 验证要求

本批是 `no code movement` 抽离方案，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

下一批 BE-001FX-03 实际抽离必须补跑:

```powershell
cargo test -p quantpilot compile_endpoint_lowers_graph_spread_bps_to_structured_threshold_condition
cargo test -p quantpilot spread_bps_condition_lowers_equivalently_across_graph_and_strategy_ir
cargo test -p quantpilot spread_bps_condition_lowers_equivalently_across_formal_graph_and_strategy_ir
```

---

## 幻觉检查点

AI 声称 BE-001FX-02 完成时，必须说明:

1. 当前只是 `no code movement` 抽离方案。
2. `spread_observer_lowering plan_frozen` 成立，但 child file 尚未创建。
3. 下一步 BE-001FX-03 只能移动 `builtin.intent.spread_observer` branch。
4. 不得宣称 `spread_observer_lowering` 已抽离。
5. 不得宣称 `intent_lowering`、`formal_module_conversion`、`backend.graph_compile`、`backend` 或 Rust 重构已经收口。

---

## 验收标准

1. `495-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `spread_observer_lowering plan_frozen` 已记录。
3. planned child、helper signature、父级调用方式、允许移动代码、不允许移动代码、回退方案和下一批测试门禁均已冻结。
4. 下一步固定为 BE-001FX-03 实际抽离记录。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
