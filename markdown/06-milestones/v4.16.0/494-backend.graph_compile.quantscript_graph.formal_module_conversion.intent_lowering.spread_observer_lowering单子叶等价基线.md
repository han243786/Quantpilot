# v4.16.0 backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering 单子叶等价基线
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FX-01
> 基线: `493-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering父叶残余判断.md`
> 目标子叶: `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering`
> 判定: 单子叶等价基线冻结
> 模块树坐标: `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering`
> 代码动作: no code movement
> 下一步: BE-001FX-02 `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FX-01 `spread_observer_lowering` 单子叶等价基线 | 子叶基线 |
| 规范矩阵 | equivalence baseline / branch-level extraction guard / parent-child communication / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` | 新增下一层白箱节点 |
| 模块树 | `backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering` | baseline_frozen |

---

## 当前真实边界

当前 `spread_observer_lowering` 仍是 `intent_lowering.rs` 内的 match branch:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering.rs
builtin.intent.spread_observer
```

planned child:

```text
src/backend/graph_compile/quantscript_graph/formal_module_conversion/intent_lowering/spread_observer_lowering.rs
```

基线标记:

```text
spread_observer_lowering baseline_frozen
intent_lowering stop_split: false
spread_observer_lowering_selected
```

本批不创建 child file，不移动 Rust 代码。

---

## 白箱输入面

当前 branch 依赖父叶已准备好的输入:

| 输入 | 来源 | 语义 |
| --- | --- | --- |
| `node` | `nodes` intent iteration | 当前 intent node |
| `cfg` | `node.config` | spread observer 参数 |
| `node_id` | `node.id` | 生成 QS 变量前缀 |
| `edges` | graph edges array | 双上游 source 查找 |
| `source_var` | shared upstream fallback | 缺失第二 source 时回退 |
| `instrument` | intent config / default `BTCUSDT` | emit Intent 的交易标的 |
| `qs_lines` | 父级 QS line buffer | 追加 generated QuantScript lines |

候选 helper signature 后续 BE-001FX-02 必须围绕该输入面设计:

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

BE-001FX-02 可在不改变调用面语义的前提下微调返回类型，但不得新增 sibling caller 或跨过 `intent_lowering` 父叶。

---

## 等价语义冻结

后续实际抽离必须保持以下代码路径:

```text
upstream_sources = edges filtered by target_node_id == node_id
source_node_id -> replace dash and dot with underscore
left_source = upstream_sources.first() or source_var
right_source = upstream_sources.get(1) or left_source
max_time_diff_ms default 5000
spread_output_code Some(1) -> bps
spread_output_code other -> ratio
comparison_threshold default 0.0
comparison_op_code Some(3) -> >=
comparison_op_code other -> >
```

必须保持 QS 行顺序:

```text
let {node_id}_left = align_asof(field({left_source}, name="bid"), direction="backward", tolerance_ms={tolerance_ms})
let {node_id}_right = align_asof(field({right_source}, name="ask"), direction="backward", tolerance_ms={tolerance_ms})
let {node_id}_signal = spread({node_id}_left, {node_id}_right, output="{output}")
if {node_id}_signal {op} {threshold} {
emit Intent("BUY", instrument="{instrument}", quantity=1.0)
}
```

必须保留以下 literal / marker:

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

---

## 父子通信规则

后续实际抽离后唯一允许新增连接:

```text
intent_lowering -> spread_observer_lowering
```

现有上层连接保持:

```text
formal_module_conversion -> intent_lowering
```

禁止新增:

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

## 非目标

本基线不处理:

1. 不创建 `spread_observer_lowering.rs`。
2. 不修改 `intent_lowering.rs`。
3. 不抽 `shared_intent_context`。
4. 不抽其它 built-in intent branch。
5. 不改 unsupported intent `anyhow::bail!`。
6. 不改 `formal_module_conversion.rs`。
7. 不改 parser、route surface、artifact target projection、frontend caller 或 runtime caller。
8. 不启动 release transition。

---

## 回归门禁

后续实际抽离时至少补跑:

```text
cargo fmt
cargo check -p quantpilot
cargo test -p quantpilot compile_endpoint_lowers_graph_spread_bps_to_structured_threshold_condition
cargo test -p quantpilot spread_bps_condition_lowers_equivalently_across_graph_and_strategy_ir
cargo test -p quantpilot spread_bps_condition_lowers_equivalently_across_formal_graph_and_strategy_ir
```

本基线提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

---

## 下一步边界

下一步只能进入:

```text
BE-001FX-02
backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering
root.backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering
```

BE-001FX-02 只允许形成抽离方案，固定 planned child、父级声明、helper signature、迁移代码块、验证门禁和回退点；不得直接创建 child file 或移动 Rust。

---

## 幻觉检查点

AI 声称 BE-001FX-01 完成时，必须说明:

1. 本批是 `no code movement` 单子叶等价基线。
2. `spread_observer_lowering baseline_frozen` 成立，但 child file 尚未创建。
3. `builtin.intent.spread_observer` 仍在 `intent_lowering.rs`。
4. 下一步只能进入 BE-001FX-02 抽离方案。
5. 不得宣称 `spread_observer_lowering` 已抽离、`intent_lowering` 已收口或 Rust 重构完成。

---

## 验收标准

1. `494-backend.graph_compile.quantscript_graph.formal_module_conversion.intent_lowering.spread_observer_lowering单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `spread_observer_lowering baseline_frozen` 已记录。
3. planned child path 和父子通信规则已冻结。
4. 下一步固定为 BE-001FX-02 抽离方案。
5. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
